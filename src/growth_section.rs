use crate::params::GrowthParams;
use crate::vector::{ANGLE_UP, Vector2D};

#[derive(Debug, Clone)]
pub enum GrowthSectionState {
    Growing,
    FullyGrown { child_indices: Vec<usize> },
}

#[derive(Debug, Clone)]
pub struct GrowthSection {
    pub start_point: Vector2D,
    pub angle: f32,
    pub current_length: f32,
    pub max_length: f32,
    pub thickness: f32,
    pub state: GrowthSectionState,
    pub parent_index: Option<usize>,
    /// Authoritative end point once `state` is `FullyGrown`. Frozen wood never
    /// moves again, so callers should rely on `end_point()` rather than reading
    /// this field directly.
    pub cached_end_point: Vector2D,
    /// Count of internodes since the last real fork along this axis (0 at the
    /// trunk and reset to 0 on each forked child). Drives fork-vs-continuation.
    pub internodes_since_fork: u32,
}

impl GrowthSection {
    pub fn new(
        start_point: Vector2D,
        angle: f32,
        max_length: f32,
        thickness: f32,
        parent_index: Option<usize>,
    ) -> Self {
        GrowthSection {
            start_point,
            angle,
            current_length: 0.0,
            max_length,
            thickness,
            state: GrowthSectionState::Growing,
            parent_index,
            cached_end_point: start_point,
            internodes_since_fork: 0,
        }
    }

    pub fn end_point(&self) -> Vector2D {
        match self.state {
            GrowthSectionState::FullyGrown { .. } => self.cached_end_point,
            GrowthSectionState::Growing => Vector2D {
                x: self.start_point.x + self.current_length * self.angle.cos(),
                y: self.start_point.y + self.current_length * self.angle.sin(),
            },
        }
    }

    /// Locks in this internode's final geometry and transitions it out of `Growing`.
    /// Once frozen, angle/thickness/length/end_point never change again — a future
    /// dormant-bud-wake feature must set a new child's `start_point` from
    /// `cached_end_point` at creation time rather than resurrecting a per-frame
    /// position-sync pass.
    pub fn freeze(&mut self, child_indices: Vec<usize>) {
        self.freeze_at(self.end_point(), child_indices);
    }

    /// `freeze` with an end point the caller already computed and handed to the
    /// children as their start. Computing it twice can differ in the last bit in
    /// release builds, which would detach children from their parent.
    pub fn freeze_at(&mut self, end_point: Vector2D, child_indices: Vec<usize>) {
        self.cached_end_point = end_point;
        self.state = GrowthSectionState::FullyGrown { child_indices };
    }

    /// Growth and tropism for one step of `delta_time` hours.
    pub fn update_physics(
        &mut self,
        delta_time: f32,
        light_direction: Vector2D,
        light_intensity: f32,
        params: &GrowthParams,
    ) {
        if let GrowthSectionState::Growing = self.state {
            // 1. Advance length (light intensity speeds up or slows down growth)
            self.current_length += params.growth_rate * light_intensity * delta_time;

            // 2. Gravitropism torque (-PI/2 is UP in 2D screen space where +Y is down)
            let gravity_torque = (ANGLE_UP - self.angle).sin() * params.gravitropism_rate;

            // 3. Phototropism torque, weaker in shade
            let light_angle = light_direction.angle();
            let photo_torque =
                (light_angle - self.angle).sin() * params.phototropism_rate * light_intensity;

            // 4. Thinner, younger tips bend more easily than thick wood
            let flexibility = (params.flexibility_reference_thickness
                / self.thickness.max(params.min_flexibility_thickness))
            .clamp(0.1, 2.0);

            self.angle += (gravity_torque + photo_torque) * flexibility * delta_time;
        }
    }
}

/// Decides whether an internode reaching full length should fork (2 children)
/// or continue (1 child), given how many internodes have elapsed since the
/// last fork on this axis. `jitter_roll` is an external U(0,1) sample so this
/// stays a pure, unit-testable function. Deterministic threshold+jitter
/// guarantees an axis eventually forks (or hits the length floor) rather than
/// continuing forever on unlucky rolls.
pub fn should_fork(
    internodes_since_fork: u32,
    fork_interval: u32,
    fork_jitter: u32,
    jitter_roll: f32,
) -> bool {
    let offset = ((jitter_roll * 2.0 - 1.0) * fork_jitter as f32).round() as i32;
    let effective_interval = (fork_interval as i32 + offset).max(1);
    internodes_since_fork as i32 >= effective_interval
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freeze_caches_end_point_and_ignores_later_mutation() {
        let mut section = GrowthSection::new(Vector2D::new(0.0, 0.0), 0.0, 10.0, 20.0, None);
        section.current_length = 10.0;
        let expected_end = section.end_point();

        section.freeze(Vec::new());
        assert_eq!(section.end_point(), expected_end);

        // Mutating geometry after freezing must not affect the cached end point.
        section.current_length = 999.0;
        section.angle = 3.0;
        assert_eq!(section.end_point(), expected_end);
    }

    #[test]
    fn should_fork_never_fires_below_jittered_threshold() {
        for roll in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert!(!should_fork(0, 4, 1, roll));
            assert!(!should_fork(2, 4, 1, roll));
        }
    }

    #[test]
    fn should_fork_always_fires_at_or_above_jittered_threshold() {
        for roll in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert!(should_fork(5, 4, 1, roll));
            assert!(should_fork(10, 4, 1, roll));
        }
    }

    #[test]
    fn should_fork_respects_jitter_within_band() {
        // roll = 0.0 -> offset = -1 -> effective_interval = 3
        assert!(should_fork(3, 4, 1, 0.0));
        // roll = 1.0 -> offset = +1 -> effective_interval = 5
        assert!(!should_fork(3, 4, 1, 1.0));
    }
}
