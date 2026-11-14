use macroquad::rand::RandGenerator;

use crate::growth_section::{GrowthSection, GrowthSectionState, should_fork};
use crate::params::GrowthParams;
use crate::vector::{ANGLE_UP, Vector2D};

/// How many hops up `parent_index` to walk when excluding a tip's own recent
/// wood from collision/shadow checks. With short internodes, thickness stays
/// constant across a continuation run (decay only applies at forks), so a tip
/// can end up within clearance distance of its own grandparent/great-grandparent
/// internode along the same axis — only excluding the immediate parent isn't
/// enough. Covers a `fork_interval + fork_jitter` run (12 + 3) plus margin.
const ANCESTOR_SKIP_DEPTH: usize = 20;

/// Fractional jitter applied to a newly created internode's target length, e.g.
/// 0.08 = +/-8%. Proportional rather than flat so it stays a minor variation at
/// large lengths but doesn't dominate/destabilize fork outcomes once lengths
/// have decayed down near `GrowthParams::min_viable_internode_length`.
const LENGTH_JITTER_FRACTION: f32 = 0.08;

/// Applies `LENGTH_JITTER_FRACTION` variation to a target internode length.
fn jittered_length(length: f32, rng: &RandGenerator) -> f32 {
    length * (1.0 + rng.gen_range(-LENGTH_JITTER_FRACTION, LENGTH_JITTER_FRACTION))
}

/// Simulated hours per step (plan 0003, D12). Every step is the same size so a
/// tree's shape depends only on its seed and step count, never on frame rate
/// or speed (plan 0001).
pub const SIM_DT: f32 = 1.0;

/// Longest real frame time fed to the simulation. A stall (window drag, sleep)
/// counts as this much time instead of its full length.
pub const MAX_FRAME_TIME: f32 = 0.1;

/// Most steps run in one frame. Past this the simulation slows down rather
/// than piling up a backlog to catch up on later.
pub const MAX_STEPS_PER_FRAME: u32 = 64;

/// Turns one frame's real time into a number of fixed steps.
/// `speed` is simulated hours per real second (0 when paused). Returns the
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

/// Light at a growing tip for one step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightSample {
    /// Wood within clearance of the tip: no light, and the section stops growing.
    pub blocked: bool,
    /// Sections crossing the ray from the tip towards the sun.
    pub shading_sections: i32,
}

impl LightSample {
    /// 0 when blocked; otherwise each shading section passes `SHADE_TRANSMISSION`.
    pub fn intensity(self) -> f32 {
        if self.blocked {
            0.0
        } else {
            SHADE_TRANSMISSION.powi(self.shading_sections)
        }
    }
}

/// Fraction of light passing each shading section. Estimate; replaced by the
/// shade-map design question at roadmap step 4.
const SHADE_TRANSMISSION: f32 = 0.85;

pub struct TreeSimulation {
    pub growth_sections: Vec<GrowthSection>,
    /// Every tunable, in metres and hours (plan 0003).
    pub params: GrowthParams,
    /// Seed the current tree was grown from; `reset` with the same seed regrows it.
    pub seed: u64,
    /// Simulated hours since the tree was planted (one per step).
    pub hours: u64,
    /// For each section, its ancestors nearest first, up to `ANCESTOR_SKIP_DEPTH`.
    /// Parallel to `growth_sections`; built once when a section is created.
    ancestor_chains: Vec<Vec<usize>>,
    /// The simulation's only source of randomness. Owned rather than global so
    /// parallel tests and future headless runs can't disturb each other.
    rng: RandGenerator,
}

impl TreeSimulation {
    /// A new tree at `trunk_start` (metres) with default parameters.
    pub fn new(trunk_start: Vector2D, seed: u64) -> Self {
        Self::with_params(trunk_start, seed, GrowthParams::default())
    }

    pub fn with_params(trunk_start: Vector2D, seed: u64, params: GrowthParams) -> Self {
        let mut sim = TreeSimulation {
            growth_sections: Vec::new(),
            params,
            seed,
            hours: 0,
            ancestor_chains: Vec::new(),
            rng: seeded_rng(seed),
        };
        sim.reset(trunk_start, seed);
        sim
    }

    /// Advances the simulation by one fixed step of `SIM_DT`. Does nothing once
    /// every section has stopped growing, so `hours` is the tree's final age.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }
        self.update(SIM_DT);
        self.hours += 1;
    }

    fn update(&mut self, delta_time: f32) {
        let light_dir = Vector2D {
            x: self.params.sun_azimuth.cos(),
            y: self.params.sun_azimuth.sin(),
        };
        // Children made this step are appended after the loop, so indices stay valid.
        let num_sections = self.growth_sections.len();
        let mut new_sections = Vec::new();

        for i in 0..num_sections {
            if !matches!(self.growth_sections[i].state, GrowthSectionState::Growing) {
                continue;
            }
            let light = self.sample_light(i, light_dir);
            let section = &mut self.growth_sections[i];
            section.update_physics(delta_time, light_dir, light.intensity(), &self.params);

            if section.current_length >= section.max_length {
                section.current_length = section.max_length;
                self.complete_internode(i, num_sections, &mut new_sections);
            } else if light.blocked {
                // Physically blocked: permanently stunted.
                section.freeze(Vec::new());
            }
        }

        self.append_sections(new_sections);
    }

    /// Light reaching growing section `i`'s tip: blocked by wood within clearance,
    /// or shaded by each section crossing the ray towards the sun. Recent
    /// relatives never count (see `is_recent_relative`).
    fn sample_light(&self, i: usize, light_dir: Vector2D) -> LightSample {
        let tip = self.growth_sections[i].end_point();
        let mut shading_sections = 0;

        for (j, other) in self.growth_sections.iter().enumerate() {
            // Skip self, and sections that just spawned and have no length yet
            if i == j || other.current_length <= f32::EPSILON {
                continue;
            }

            // Cheap geometry first: most sections are nowhere near this tip,
            // so the relative check runs only for the rare hits (plan 0004).
            let (other_start, other_end) = (other.start_point, other.end_point());
            let min_clearance =
                (self.growth_sections[i].thickness + other.thickness) * self.params.thickness_decay;
            let too_close = tip.distance_to_segment(other_start, other_end) < min_clearance;
            let shades = !too_close
                && Vector2D::ray_intersects_segment(tip, light_dir, other_start, other_end);

            if !(too_close || shades) || self.is_recent_relative(i, j) {
                continue;
            }
            if too_close {
                return LightSample {
                    blocked: true,
                    shading_sections,
                };
            }
            shading_sections += 1;
        }
        LightSample {
            blocked: false,
            shading_sections,
        }
    }

    /// A direct ancestor or descendant of `i`, or a "cousin" sharing an ancestor
    /// within `ANCESTOR_SKIP_DEPTH` (e.g. the other fork child's continuations).
    /// With short internodes a tip is always geometrically close to its own
    /// recent wood, so these are excluded from collision and shade.
    fn is_recent_relative(&self, i: usize, j: usize) -> bool {
        let chains = &self.ancestor_chains;
        chains[i].contains(&j)
            || chains[j].contains(&i)
            || chains[i].iter().any(|a| chains[j].contains(a))
    }

    /// Section `i` has reached full length: stop the axis, fork into a leader
    /// and a lateral, or continue with one child. New children go to
    /// `new_sections`; `num_sections` is where their indices start.
    fn complete_internode(
        &mut self,
        i: usize,
        num_sections: usize,
        new_sections: &mut Vec<GrowthSection>,
    ) {
        let section = &mut self.growth_sections[i];

        // Size floor: an axis whose target length has shrunk to the floor stops
        // here, whatever the fork cadence. Continuations don't re-check size, so
        // without this an axis could keep spawning undersized internodes.
        if section.max_length <= self.params.min_viable_internode_length {
            section.freeze(Vec::new());
            return;
        }

        let end_pos = section.end_point();
        let jitter_roll = self.rng.gen_range(0.0, 1.0);
        let fork_due = should_fork(
            section.internodes_since_fork,
            self.params.fork_interval,
            self.params.fork_jitter,
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
            // fast across generations that it hits min_viable_internode_length
            // after only one or two forks and can never build out a
            // sub-canopy. Thickness/angle keep the full ratio since those
            // don't compound the same way — a thinner, more-angled lateral
            // still reads as visually subordinate without being starved out.
            let length_ratio =
                0.5 + (self.params.dominance_ratio - 0.5) * self.params.length_dominance_blend;
            let leader_length = section.max_length * self.params.length_decay * 2.0 * length_ratio;
            let lateral_length =
                section.max_length * self.params.length_decay * 2.0 * (1.0 - length_ratio);
            let leader_thickness =
                section.thickness * self.params.thickness_decay * 2.0 * self.params.dominance_ratio;
            let lateral_thickness = section.thickness
                * self.params.thickness_decay
                * 2.0
                * (1.0 - self.params.dominance_ratio);

            let leader_angle_offset =
                self.params.branch_angle_spread * (1.0 - self.params.dominance_ratio);
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
                    section.angle + self.params.branch_angle_spread,
                )
            } else {
                (
                    section.angle + leader_angle_offset,
                    section.angle - self.params.branch_angle_spread,
                )
            };

            let mut child_indices = Vec::new();

            if leader_length > self.params.min_viable_internode_length {
                child_indices.push(num_sections + new_sections.len());
                new_sections.push(GrowthSection::new(
                    end_pos,
                    leader_angle,
                    jittered_length(leader_length, &self.rng),
                    leader_thickness,
                    Some(i),
                ));
            }
            if lateral_length > self.params.min_viable_internode_length {
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
            section.freeze_at(end_pos, child_indices);
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

            section.freeze_at(end_pos, vec![child_idx]);

            new_sections.push(child);
        }
    }

    /// Appends this step's new sections with their ancestor chains. A section's
    /// ancestry never changes, so each chain is built once, here.
    fn append_sections(&mut self, new_sections: Vec<GrowthSection>) {
        for section in &new_sections {
            let parent = section
                .parent_index
                .expect("new sections always have a parent");
            let mut chain = Vec::with_capacity(ANCESTOR_SKIP_DEPTH);
            chain.push(parent);
            chain.extend(
                self.ancestor_chains[parent]
                    .iter()
                    .take(ANCESTOR_SKIP_DEPTH - 1),
            );
            self.ancestor_chains.push(chain);
        }
        self.growth_sections.extend(new_sections);
    }

    pub fn is_finished(&self) -> bool {
        // `matches!` checks the variant type and ignores `child_indices`
        self.growth_sections
            .iter()
            .all(|section| matches!(section.state, GrowthSectionState::FullyGrown { .. }))
    }

    pub fn reset(&mut self, trunk_start: Vector2D, seed: u64) {
        self.seed = seed;
        self.rng = seeded_rng(seed);
        self.hours = 0;
        let trunk = GrowthSection::new(
            trunk_start,
            ANGLE_UP,
            self.params.internode_length,
            self.params.trunk_thickness,
            None,
        );
        self.growth_sections = vec![trunk];
        self.ancestor_chains = vec![Vec::new()];
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
    /// About two simulated years: enough internodes for several forks.
    const STEPS: usize = 18_000;

    fn grow(seed: u64, steps: usize) -> TreeSimulation {
        let mut sim = TreeSimulation::new(START, seed);
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
        assert_eq!(snapshot(&grow(42, STEPS)), snapshot(&grow(42, STEPS)));
    }

    #[test]
    fn reset_with_same_seed_regrows_identical_tree() {
        let mut sim = grow(7, STEPS);
        let first = snapshot(&sim);
        sim.reset(START, 7);
        for _ in 0..STEPS {
            sim.step();
        }
        assert_eq!(snapshot(&sim), first);
    }

    #[test]
    fn different_seeds_grow_different_trees() {
        assert_ne!(snapshot(&grow(1, STEPS)), snapshot(&grow(2, STEPS)));
    }

    #[test]
    fn first_internode_takes_length_over_rate_hours() {
        let params = GrowthParams::default();
        let expected = (params.internode_length / params.growth_rate).ceil() as u64;
        let mut sim = TreeSimulation::new(START, 1);
        while sim.growth_sections.len() == 1 {
            sim.step();
        }
        assert!(
            sim.hours.abs_diff(expected) <= 1,
            "took {} h, expected ~{expected} h",
            sim.hours
        );
    }

    /// Hash of a seeded tree's geometry and structure (positions, angles,
    /// lengths, thickness, links). Refactors and performance work must not
    /// change it; only a deliberate behaviour change may, with a re-recorded value.
    fn fingerprint(sim: &TreeSimulation) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for s in &sim.growth_sections {
            let children = match &s.state {
                GrowthSectionState::Growing => None,
                GrowthSectionState::FullyGrown { child_indices } => Some(child_indices.clone()),
            };
            let end = s.end_point();
            for value in [
                s.start_point.x,
                s.start_point.y,
                end.x,
                end.y,
                s.angle,
                s.current_length,
                s.max_length,
                s.thickness,
            ] {
                value.to_bits().hash(&mut hasher);
            }
            (s.parent_index, children, s.internodes_since_fork).hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Recorded at `3ae50a4` (after plan 0004), before the code tidy-up.
    const FINGERPRINT_SEED_3_AT_30K: u64 = 14_743_237_593_890_381_805;

    #[test]
    fn optimisations_leave_the_tree_unchanged() {
        let got = fingerprint(&grow(3, 30_000));
        assert_eq!(
            got, FINGERPRINT_SEED_3_AT_30K,
            "tree changed: fingerprint {got}"
        );
    }

    #[test]
    fn fork_splits_thickness_and_angle_by_dominance() {
        // Fork at the second internode with no jitter.
        let params = GrowthParams {
            fork_interval: 1,
            fork_jitter: 0,
            ..GrowthParams::default()
        };
        let mut sim = TreeSimulation::with_params(START, 1, params.clone());
        let fork = loop {
            sim.step();
            let found = sim.growth_sections.iter().position(|s| {
                matches!(&s.state, GrowthSectionState::FullyGrown { child_indices } if child_indices.len() == 2)
            });
            if let Some(i) = found {
                break i;
            }
        };
        let parent = &sim.growth_sections[fork];
        let GrowthSectionState::FullyGrown { child_indices } = &parent.state else {
            unreachable!()
        };
        let (a, b) = (
            &sim.growth_sections[child_indices[0]],
            &sim.growth_sections[child_indices[1]],
        );
        let (leader, lateral) = if a.thickness > b.thickness {
            (a, b)
        } else {
            (b, a)
        };

        let d = params.dominance_ratio;
        assert!((leader.thickness / lateral.thickness - d / (1.0 - d)).abs() < 1e-4);
        // Children have grown one step at most, so their angles are still near the split.
        let leader_offset = (leader.angle - parent.angle).abs();
        let lateral_offset = (lateral.angle - parent.angle).abs();
        assert!(
            (leader_offset - params.branch_angle_spread * (1.0 - d)).abs() < 0.01,
            "leader {leader_offset}"
        );
        assert!(
            (lateral_offset - params.branch_angle_spread).abs() < 0.01,
            "lateral {lateral_offset}"
        );
    }

    #[test]
    fn tilted_shoot_bends_upwards() {
        let params = GrowthParams::default();
        let up = Vector2D::new(0.0, -1.0);
        let mut shoot = GrowthSection::new(START, ANGLE_UP + 0.5, 1.0, 0.002, None);
        for _ in 0..200 {
            shoot.update_physics(SIM_DT, up, 1.0, &params);
        }
        assert!(
            (shoot.angle - ANGLE_UP).abs() < 0.5,
            "angle moved {} rad from upright",
            shoot.angle - ANGLE_UP
        );
    }

    /// A frozen section from `a` to `b` with no relatives, for light tests.
    fn add_obstacle(sim: &mut TreeSimulation, a: Vector2D, b: Vector2D) {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let length = (dx * dx + dy * dy).sqrt();
        let mut s = GrowthSection::new(a, dy.atan2(dx), length, 0.05, None);
        s.current_length = length;
        s.freeze(Vec::new());
        sim.growth_sections.push(s);
        sim.ancestor_chains.push(Vec::new());
    }

    #[test]
    fn branch_overhead_shades_the_tip() {
        let mut sim = TreeSimulation::new(START, 1);
        add_obstacle(
            &mut sim,
            Vector2D::new(-0.1, -0.5),
            Vector2D::new(0.1, -0.5),
        );
        let light = sim.sample_light(0, Vector2D::new(0.0, -1.0));
        assert_eq!(
            light,
            LightSample {
                blocked: false,
                shading_sections: 1
            }
        );
        assert!((light.intensity() - 0.85).abs() < 1e-6);
    }

    #[test]
    fn branch_too_close_blocks_the_tip() {
        let mut sim = TreeSimulation::new(START, 1);
        add_obstacle(
            &mut sim,
            Vector2D::new(-0.1, -0.02),
            Vector2D::new(0.1, -0.02),
        );
        let light = sim.sample_light(0, Vector2D::new(0.0, -1.0));
        assert!(light.blocked);
        assert_eq!(light.intensity(), 0.0);
    }

    #[test]
    fn clock_stops_when_tree_finishes() {
        // A floor equal to the internode length makes the trunk its only section.
        let params = GrowthParams {
            min_viable_internode_length: GrowthParams::default().internode_length,
            ..GrowthParams::default()
        };
        let mut sim = TreeSimulation::with_params(START, 1, params);
        while !sim.is_finished() {
            sim.step();
        }
        let finished_at = sim.hours;
        for _ in 0..100 {
            sim.step();
        }
        assert_eq!(sim.hours, finished_at);
    }

    #[test]
    fn plan_steps_cases() {
        // Speeds are simulated hours per real second; SIM_DT is one hour.
        // (case, accumulator, frame_time, speed, expected steps, max accumulator)
        let cases = [
            ("normal frame", 0.0, 1.0 / 60.0, 60.0, 1, 0.0),
            ("paused keeps accumulator", 0.5, 1.0 / 60.0, 0.0, 0, 0.5),
            ("stall clamped to MAX_FRAME_TIME", 0.0, 2.0, 60.0, 6, 0.0),
            (
                "cap hit drops the backlog",
                0.0,
                0.1,
                1200.0,
                MAX_STEPS_PER_FRAME,
                SIM_DT,
            ),
        ];
        for (case, acc, frame, speed, steps, remaining) in cases {
            let (got_steps, got_acc) = plan_steps(acc, frame, speed);
            assert_eq!(got_steps, steps, "{case}: steps");
            assert!(
                got_acc <= remaining + 1e-4,
                "{case}: accumulator {got_acc} > {remaining}"
            );
            assert!(got_acc >= 0.0, "{case}: negative accumulator");
        }
    }

    #[test]
    fn simulated_time_is_independent_of_frame_rate() {
        let total_steps = |fps: u32| {
            let (mut acc, mut total) = (0.0, 0u32);
            for _ in 0..fps * 10 {
                let (steps, next) = plan_steps(acc, 1.0 / fps as f32, 60.0);
                acc = next;
                total += steps;
            }
            total
        };
        for fps in [30, 60, 144] {
            let steps = total_steps(fps);
            assert!(
                steps.abs_diff(600) <= 1,
                "{fps} fps ran {steps} steps for 10 s"
            );
        }
    }
}
