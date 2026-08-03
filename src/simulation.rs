use macroquad::prelude::*;

use crate::growth_section::{GrowthSection, GrowthSectionState, should_fork};
use crate::vector::{ANGLE_UP, Vector2D};

/// Below this length a forked child is no longer viable and the axis terminates.
/// Kept low relative to `internode_length` so a length_decay of 0.7 gets several
/// fork generations before hitting the floor, rather than terminating after only
/// two or three (which reads as a sparse, "handful of branches" tree).
const MIN_VIABLE_INTERNODE_LENGTH: f32 = 2.0;

/// How many hops up `parent_index` to walk when excluding a tip's own recent
/// wood from collision/shadow checks. With short internodes, thickness stays
/// constant across a continuation run (decay only applies at forks), so a tip
/// can end up within clearance distance of its own grandparent/great-grandparent
/// internode along the same axis — only excluding the immediate parent isn't
/// enough. Comfortably covers a `fork_interval + fork_jitter` run plus margin.
const ANCESTOR_SKIP_DEPTH: usize = 10;

/// Fractional jitter applied to a newly created internode's target length, e.g.
/// 0.08 = +/-8%. Proportional rather than flat so it stays a minor variation at
/// large lengths but doesn't dominate/destabilize fork outcomes once lengths
/// have decayed down near `MIN_VIABLE_INTERNODE_LENGTH`.
const LENGTH_JITTER_FRACTION: f32 = 0.08;

/// Applies `LENGTH_JITTER_FRACTION` variation to a target internode length.
fn jittered_length(length: f32) -> f32 {
    length * (1.0 + rand::gen_range(-LENGTH_JITTER_FRACTION, LENGTH_JITTER_FRACTION))
}

/// How strongly a fork's leader-side coin flip is biased toward correcting
/// the axis back toward vertical, vs. a pure 50/50 flip. 0.0 = unbiased
/// (random walk); higher values self-correct faster but look less naturally
/// random. Kept well short of 1.0 so the leader still visibly zigzags rather
/// than snapping straight — real sympodial zigzag is statistical, not exact.
const LEADER_CORRECTION_STRENGTH: f32 = 0.35;

pub struct TreeSimulation {
    pub growth_sections: Vec<GrowthSection>,
    pub branch_angle_spread: f32, // Angle separation for child growth sections (radians)
    pub length_decay: f32,        // Multiplier for child section max_length
    pub thickness_decay: f32,     // Multiplier for child section thickness
    /// Share of the fork's combined thickness decay given to the leader child,
    /// vs. 1.0 - dominance_ratio for the lateral child (0.5 = symmetric, matching
    /// the old fork behavior). Also shrinks the leader's angle deviation relative
    /// to `branch_angle_spread` so it continues closer to the parent's trajectory
    /// while the lateral branches off at the full spread — modeling sympodial
    /// apical dominance (e.g. elm), where the bud best-connected to the old axis
    /// wins the resource/vascular race and becomes the new leader.
    pub dominance_ratio: f32,
    /// How much of `dominance_ratio`'s asymmetry carries over to *length*
    /// specifically (0.0 = length always splits symmetrically regardless of
    /// dominance_ratio, 1.0 = length uses the same full asymmetry as thickness).
    /// Length is what determines how many more generations a lateral can fork
    /// before hitting `MIN_VIABLE_INTERNODE_LENGTH`, so applying the full
    /// dominance_ratio to it compounds fast and starves laterals out after only
    /// one or two generations — a visibly thick/straight leader doesn't need an
    /// equally starved lateral length to read as dominant.
    pub length_dominance_blend: f32,
    pub sun_azimuth: f32,      // Angle pointing TO the sun in radians
    pub internode_length: f32, // Target length of each internode before it freezes
    pub fork_interval: u32,    // Baseline # of internodes between forks along an axis
    pub fork_jitter: u32,      // +/- range applied to fork_interval so forking isn't metronomic
}

impl TreeSimulation {
    pub fn new(trunk_start: Vector2D, trunk_thickness: f32) -> Self {
        let internode_length = 12.0;
        let trunk = GrowthSection::new(
            trunk_start,
            ANGLE_UP,
            internode_length,
            trunk_thickness,
            None,
        );

        TreeSimulation {
            growth_sections: vec![trunk],
            branch_angle_spread: 40.0_f32.to_radians(),
            // Combined leader+lateral length budget per fork. Raised slightly from
            // 0.7 so the leader (which keeps the larger share, see dominance_ratio)
            // survives more fork generations before hitting
            // MIN_VIABLE_INTERNODE_LENGTH — that floor is what was capping the
            // whole tree's final size, since once the leader axis hits it the
            // entire tree stops growing.
            length_decay: 0.75,
            thickness_decay: 0.707, // approx 1/sqrt(2) (ref. Da Vinci - splits in two)
            dominance_ratio: 0.6,   // Leader favored over lateral; 0.5 would be symmetric
            length_dominance_blend: 0.5, // Length asymmetry softened relative to thickness/angle
            sun_azimuth: ANGLE_UP,
            internode_length,
            // Longer straight runs between forks add height/volume linearly
            // (more continuation internodes per axis) rather than exponentially
            // (more branch points), so this is a cheap way to make the overall
            // tree bigger without a combinatorial blowup in section count.
            fork_interval: 9,
            fork_jitter: 3, // Keeps the same +/-33% relative jitter as before
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Convert the azimuth angle into a normalized directional vector
        let global_light_dir = Vector2D {
            x: self.sun_azimuth.cos(),
            y: self.sun_azimuth.sin(),
        };
        let num_sections = self.growth_sections.len();
        let mut new_sections = Vec::new();

        // Precompute each section's bounded ancestor chain once per frame (shared by
        // every growing tip below) rather than per (tip, candidate) pair.
        let ancestor_chains: Vec<Vec<usize>> = (0..num_sections)
            .map(|idx| {
                let mut chain = Vec::with_capacity(ANCESTOR_SKIP_DEPTH);
                let mut walk = self.growth_sections[idx].parent_index;
                while let Some(a) = walk {
                    if chain.len() >= ANCESTOR_SKIP_DEPTH {
                        break;
                    }
                    chain.push(a);
                    walk = self.growth_sections[a].parent_index;
                }
                chain
            })
            .collect();

        for i in 0..num_sections {
            // Only growing sections need collision & light sampling
            if !matches!(self.growth_sections[i].state, GrowthSectionState::Growing) {
                continue;
            }

            let tip = self.growth_sections[i].end_point();
            let mut shadow_count = 0;
            let mut is_blocked = false;

            'inner: for j in 0..num_sections {
                // Skip checking against self
                if i == j {
                    continue;
                }

                // Skip anything in the same recent lineage: a direct ancestor/descendant
                // of i, or a "cousin" that shares a common ancestor with i within the
                // bounded depth (e.g. the other fork child's own continuations) — with
                // short internodes, unrelated-looking nodes can be geometrically close
                // to a very recent relative, so this must catch more than just parent/
                // child/sibling.
                let is_recent_relative = ancestor_chains[i].contains(&j)
                    || ancestor_chains[j].contains(&i)
                    || ancestor_chains[i]
                        .iter()
                        .any(|a| ancestor_chains[j].contains(a));

                if is_recent_relative {
                    continue 'inner;
                }

                // Skip sections that just spawned and have no physical length
                if self.growth_sections[j].current_length <= f32::EPSILON {
                    continue 'inner;
                }

                let other_start = self.growth_sections[j].start_point;
                let other_end = self.growth_sections[j].end_point();

                // 1. Physical Collision Guard:
                // Stop growth if the tip comes within safety clearance of another section
                let dist = tip.distance_to_segment(other_start, other_end);
                let min_clearance = (self.growth_sections[i].thickness
                    + self.growth_sections[j].thickness)
                    * self.thickness_decay;

                if dist < min_clearance {
                    is_blocked = true;
                    break;
                }

                // 2. Canopy Occlusion (Raycast):
                // Check if another section is directly above this tip blocking skylight
                if Vector2D::ray_intersects_segment(tip, global_light_dir, other_start, other_end) {
                    shadow_count += 1;
                }
            }

            // Calculate effective light intensity:
            // 0.0 if blocked physically, exponentially reduced by overhead canopy layers
            let local_light_intensity = if is_blocked {
                0.0
            } else {
                0.85_f32.powi(shadow_count)
            };

            // Now create a mutable version of the section at index i
            let section = &mut self.growth_sections[i];

            // Apply physics with the dynamically calculated local light
            section.update_physics(delta_time, global_light_dir, local_light_intensity);

            // Handle internode completion: fork into two children, continue as one, or terminate.
            if section.current_length >= section.max_length {
                section.current_length = section.max_length; // Clamp

                // Unconditional size floor: an axis whose own target length has
                // already shrunk to the floor stops for good here, regardless of
                // the fork-vs-continue cadence below. Without this, continuation
                // never re-checks size at all (only a fork attempt does), so an
                // axis could keep spawning several more undersized internodes
                // before its next scheduled fork check ever looked at length.
                if section.max_length <= MIN_VIABLE_INTERNODE_LENGTH {
                    section.freeze(Vec::new());
                    continue;
                }

                let end_pos = section.end_point();
                let jitter_roll = rand::gen_range(0.0, 1.0);
                let fork_due = should_fork(
                    section.internodes_since_fork,
                    self.fork_interval,
                    self.fork_jitter,
                    jitter_roll,
                );

                if fork_due {
                    // Split the fork's combined decay budget asymmetrically: the
                    // leader keeps most of it (longer, thicker, straighter), the
                    // lateral gets the remainder (shorter, thinner, wider angle) —
                    // see `dominance_ratio` doc comment for the biological basis.
                    //
                    // Length uses a softened ratio (see `length_dominance_blend`):
                    // at the full dominance_ratio, the lateral's length shrinks so
                    // fast across generations that it hits MIN_VIABLE_INTERNODE_LENGTH
                    // after only one or two forks and can never build out a
                    // sub-canopy. Thickness/angle keep the full ratio since those
                    // don't compound the same way — a thinner, more-angled lateral
                    // still reads as visually subordinate without being starved out.
                    let length_ratio =
                        0.5 + (self.dominance_ratio - 0.5) * self.length_dominance_blend;
                    let leader_length = section.max_length * self.length_decay * 2.0 * length_ratio;
                    let lateral_length =
                        section.max_length * self.length_decay * 2.0 * (1.0 - length_ratio);
                    let leader_thickness =
                        section.thickness * self.thickness_decay * 2.0 * self.dominance_ratio;
                    let lateral_thickness = section.thickness
                        * self.thickness_decay
                        * 2.0
                        * (1.0 - self.dominance_ratio);

                    let leader_angle_offset =
                        self.branch_angle_spread * (1.0 - self.dominance_ratio);
                    // Bias which side the leader kinks toward correcting the axis
                    // back toward vertical, instead of a pure coin flip. A thick,
                    // low-flexibility leader (see `dominance_ratio`) barely bends
                    // via gravitropism before it freezes, so an unbiased flip is a
                    // true random walk in heading that never self-corrects and
                    // compounds into a wandering, snake-like trunk over many forks.
                    // Reuses gravitropism's own signed sin() convention (see
                    // `update_physics`): positive means increasing angle is the
                    // corrective direction, so it lowers the odds of the "left"
                    // (angle-decreasing) option in that case, and vice versa.
                    let restoring_direction = (ANGLE_UP - section.angle).sin();
                    let leader_on_left_probability =
                        (0.5 - restoring_direction * LEADER_CORRECTION_STRENGTH).clamp(0.1, 0.9);
                    let leader_on_left = rand::gen_range(0.0, 1.0) < leader_on_left_probability;
                    let (leader_angle, lateral_angle) = if leader_on_left {
                        (
                            section.angle - leader_angle_offset,
                            section.angle + self.branch_angle_spread,
                        )
                    } else {
                        (
                            section.angle + leader_angle_offset,
                            section.angle - self.branch_angle_spread,
                        )
                    };

                    let mut child_indices = Vec::new();

                    if leader_length > MIN_VIABLE_INTERNODE_LENGTH {
                        child_indices.push(num_sections + new_sections.len());
                        new_sections.push(GrowthSection::new(
                            end_pos,
                            leader_angle,
                            jittered_length(leader_length),
                            leader_thickness,
                            Some(i),
                        ));
                    }
                    if lateral_length > MIN_VIABLE_INTERNODE_LENGTH {
                        child_indices.push(num_sections + new_sections.len());
                        new_sections.push(GrowthSection::new(
                            end_pos,
                            lateral_angle,
                            jittered_length(lateral_length),
                            lateral_thickness,
                            Some(i),
                        ));
                    }

                    // Both may fail the viability floor (rare at dominance_ratio
                    // extremes) — freezing with an empty child list terminates
                    // this axis for good, same as the old symmetric behavior.
                    section.freeze(child_indices);
                } else {
                    // Continuation: same angle/thickness/target length, no decay applied.
                    let mut child = GrowthSection::new(
                        end_pos,
                        section.angle,
                        jittered_length(section.max_length),
                        section.thickness,
                        Some(i),
                    );
                    child.internodes_since_fork = section.internodes_since_fork + 1;

                    let child_idx = num_sections + new_sections.len();

                    section.freeze(vec![child_idx]);

                    new_sections.push(child);
                }
            } else if is_blocked {
                // The section is physically blocked and permanently stunted.
                section.freeze(Vec::new());
            }
        }

        // Pass 2: Append newly created sections
        self.growth_sections.extend(new_sections);
    }

    pub fn is_finished(&self) -> bool {
        // `matches!` checks the variant type and ignores `child_indices`
        self.growth_sections
            .iter()
            .all(|section| matches!(section.state, GrowthSectionState::FullyGrown { .. }))
    }

    pub fn reset(&mut self, trunk_start: Vector2D, trunk_thickness: f32) {
        let trunk = GrowthSection::new(
            trunk_start,
            ANGLE_UP,
            self.internode_length,
            trunk_thickness,
            None,
        );
        self.growth_sections = vec![trunk];
    }
}
