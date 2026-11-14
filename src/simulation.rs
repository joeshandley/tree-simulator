use macroquad::rand::RandGenerator;

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
fn jittered_length(length: f32, rng: &RandGenerator) -> f32 {
    length * (1.0 + rng.gen_range(-LENGTH_JITTER_FRACTION, LENGTH_JITTER_FRACTION))
}

/// Simulated seconds per step. Every step is the same size so a tree's shape
/// depends only on its seed and step count, never on frame rate or speed
/// (plan 0001).
pub const SIM_DT: f32 = 1.0 / 60.0;

/// Longest real frame time fed to the simulation. A stall (window drag, sleep)
/// counts as this much time instead of its full length.
pub const MAX_FRAME_TIME: f32 = 0.1;

/// Most steps run in one frame. Past this the simulation slows down rather
/// than piling up a backlog to catch up on later.
pub const MAX_STEPS_PER_FRAME: u32 = 64;

/// Turns one frame's real time into a number of fixed steps.
/// `speed` is simulated seconds per real second (0 when paused). Returns the
/// steps to run and the accumulator to carry into the next frame.
pub fn plan_steps(accumulator: f32, frame_time: f32, speed: f32) -> (u32, f32) {
    let total = accumulator + frame_time.min(MAX_FRAME_TIME) * speed;
    // The epsilon stops f32 rounding (e.g. 0.1 / (1/60) = 5.9999995) losing a step.
    let available = ((total / SIM_DT) + 1e-4).floor() as u32;
    let steps = available.min(MAX_STEPS_PER_FRAME);
    // Capping at SIM_DT drops any backlog when the cap was hit (slow down).
    let remaining = (total - steps as f32 * SIM_DT).clamp(0.0, SIM_DT);
    (steps, remaining)
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
    /// Seed the current tree was grown from; `reset` with the same seed regrows it.
    pub seed: u64,
    /// The simulation's only source of randomness. Owned rather than global so
    /// parallel tests and future headless runs can't disturb each other.
    rng: RandGenerator,
}

impl TreeSimulation {
    pub fn new(trunk_start: Vector2D, trunk_thickness: f32, seed: u64) -> Self {
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
            seed,
            rng: seeded_rng(seed),
        }
    }

    /// Advances the simulation by one fixed step of `SIM_DT`.
    pub fn step(&mut self) {
        self.update(SIM_DT);
    }

    fn update(&mut self, delta_time: f32) {
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
                let jitter_roll = self.rng.gen_range(0.0, 1.0);
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
                    let leader_on_left = self.rng.gen_range(0.0, 1.0) < leader_on_left_probability;
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
                            jittered_length(leader_length, &self.rng),
                            leader_thickness,
                            Some(i),
                        ));
                    }
                    if lateral_length > MIN_VIABLE_INTERNODE_LENGTH {
                        child_indices.push(num_sections + new_sections.len());
                        new_sections.push(GrowthSection::new(
                            end_pos,
                            lateral_angle,
                            jittered_length(lateral_length, &self.rng),
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
                        jittered_length(section.max_length, &self.rng),
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

    pub fn reset(&mut self, trunk_start: Vector2D, trunk_thickness: f32, seed: u64) {
        self.seed = seed;
        self.rng = seeded_rng(seed);
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

fn seeded_rng(seed: u64) -> RandGenerator {
    let rng = RandGenerator::new();
    rng.srand(seed);
    rng
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: Vector2D = Vector2D { x: 0.0, y: 0.0 };

    fn grow(seed: u64, steps: usize) -> TreeSimulation {
        let mut sim = TreeSimulation::new(START, 20.0, seed);
        for _ in 0..steps {
            sim.step();
        }
        sim
    }

    fn snapshot(sim: &TreeSimulation) -> String {
        format!("{:?}", sim.growth_sections)
    }

    #[test]
    fn same_seed_grows_identical_tree() {
        assert_eq!(snapshot(&grow(42, 800)), snapshot(&grow(42, 800)));
    }

    #[test]
    fn reset_with_same_seed_regrows_identical_tree() {
        let mut sim = grow(7, 800);
        let first = snapshot(&sim);
        sim.reset(START, 20.0, 7);
        for _ in 0..800 {
            sim.step();
        }
        assert_eq!(snapshot(&sim), first);
    }

    #[test]
    fn different_seeds_grow_different_trees() {
        assert_ne!(snapshot(&grow(1, 800)), snapshot(&grow(2, 800)));
    }

    #[test]
    fn plan_steps_cases() {
        // (case, accumulator, frame_time, speed, expected steps, expected accumulator)
        let cases = [
            ("normal frame", 0.0, SIM_DT, 1.0, 1, 0.0),
            ("paused keeps accumulator", 0.5 * SIM_DT, SIM_DT, 0.0, 0, 0.5 * SIM_DT),
            ("stall clamped to MAX_FRAME_TIME", 0.0, 2.0, 1.0, 6, 0.0),
            ("cap hit drops the backlog", 0.0, 0.1, 20.0, MAX_STEPS_PER_FRAME, SIM_DT),
        ];
        for (case, acc, frame, speed, steps, remaining) in cases {
            let (got_steps, got_acc) = plan_steps(acc, frame, speed);
            assert_eq!(got_steps, steps, "{case}: steps");
            assert!(got_acc <= remaining + 1e-6, "{case}: accumulator {got_acc} > {remaining}");
            assert!(got_acc >= 0.0, "{case}: negative accumulator");
        }
    }

    #[test]
    fn simulated_time_is_independent_of_frame_rate() {
        let total_steps = |fps: u32| {
            let (mut acc, mut total) = (0.0, 0u32);
            for _ in 0..fps * 10 {
                let (steps, next) = plan_steps(acc, 1.0 / fps as f32, 1.0);
                acc = next;
                total += steps;
            }
            total
        };
        for fps in [30, 60, 144] {
            let steps = total_steps(fps);
            assert!(steps.abs_diff(600) <= 1, "{fps} fps ran {steps} steps for 10 s");
        }
    }
}
