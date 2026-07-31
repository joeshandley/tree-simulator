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

pub struct TreeSimulation {
    pub growth_sections: Vec<GrowthSection>,
    pub branch_angle_spread: f32, // Angle separation for child growth sections (radians)
    pub length_decay: f32,        // Multiplier for child section max_length
    pub thickness_decay: f32,     // Multiplier for child section thickness
    pub sun_azimuth: f32,         // Angle pointing TO the sun in radians
    pub internode_length: f32,    // Target length of each internode before it freezes
    pub fork_interval: u32,       // Baseline # of internodes between forks along an axis
    pub fork_jitter: u32,         // +/- range applied to fork_interval so forking isn't metronomic
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
            branch_angle_spread: 25.0_f32.to_radians(),
            length_decay: 0.7,
            thickness_decay: 0.707, // approx 1/sqrt(2) (ref. Da Vinci - splits in two)
            sun_azimuth: ANGLE_UP,
            internode_length,
            fork_interval: 6,
            fork_jitter: 2,
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
                    let child_length = section.max_length * self.length_decay;
                    let child_thickness = section.thickness * self.thickness_decay;

                    // Asymmetric spread: Primary leader shoot stays closer to parent angle, side shoot branches off
                    let left_angle = section.angle - self.branch_angle_spread;
                    let right_angle = section.angle + self.branch_angle_spread;

                    if child_length > MIN_VIABLE_INTERNODE_LENGTH {
                        let left_child = GrowthSection::new(
                            end_pos,
                            left_angle,
                            jittered_length(child_length),
                            child_thickness,
                            Some(i),
                        );
                        let right_child = GrowthSection::new(
                            end_pos,
                            right_angle,
                            jittered_length(child_length),
                            child_thickness,
                            Some(i),
                        );

                        let left_idx = num_sections + new_sections.len();
                        let right_idx = left_idx + 1;

                        section.freeze(vec![left_idx, right_idx]);

                        new_sections.push(left_child);
                        new_sections.push(right_child);
                    } else {
                        // Terminal branches (smallest section threshold reached)
                        // also need to transition out of Growing state!
                        section.freeze(Vec::new());
                    }
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
