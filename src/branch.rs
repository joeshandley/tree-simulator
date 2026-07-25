use crate::vector::{ANGLE_UP, Vector2D};

#[derive(Debug, Clone)]
pub enum BranchState {
    Growing,
    FullyGrown { child_indices: Vec<usize> },
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub start_point: Vector2D,
    pub angle: f32,
    pub current_length: f32,
    pub max_length: f32,
    pub thickness: f32,
    pub growth_rate: f32,
    pub state: BranchState,
    pub parent_index: Option<usize>,
    pub gravitropism_weight: f32, // Sensitivity to gravity (upward pull)
    pub phototropism_weight: f32, // Sensitivity to light direction
}

impl Branch {
    pub fn new(
        start_point: Vector2D,
        angle: f32,
        max_length: f32,
        thickness: f32,
        parent_index: Option<usize>,
    ) -> Self {
        Branch {
            start_point,
            angle,
            current_length: 0.0,
            max_length,
            thickness,
            growth_rate: 50.0,
            state: BranchState::Growing,
            parent_index,
            gravitropism_weight: 1.0,
            phototropism_weight: 0.5, // Phototropism often dominates young tips
        }
    }

    pub fn end_point(&self) -> Vector2D {
        Vector2D {
            x: self.start_point.x + self.current_length * self.angle.cos(),
            y: self.start_point.y + self.current_length * self.angle.sin(),
        }
    }

    /// Primary growth & tropism physics update
    pub fn update_physics(
        &mut self,
        delta_time: f32,
        light_direction: Vector2D,
        light_intensity: f32,
    ) {
        if let BranchState::Growing = self.state {
            // 1. Advance length (light intensity speeds up or slows down growth)
            let effective_growth_rate = self.growth_rate * light_intensity;
            self.current_length += effective_growth_rate * delta_time;

            // 2. Gravitropism Force (-PI/2 is UP in 2D screen space where +Y is Down)
            let gravity_torque = (ANGLE_UP - self.angle).sin() * self.gravitropism_weight;

            // 3. Phototropism Force (Bend toward local light source)
            let light_angle = light_direction.angle();
            // Scale phototropism by light intensity (leaves in deep shadow don't bend toward light as strongly)
            let photo_torque =
                (light_angle - self.angle).sin() * self.phototropism_weight * light_intensity;

            // 4. Structural Flexibility (Thinner, younger tips bend much more easily than thick wood)
            let flexibility = (1.5 / self.thickness.max(0.5)).clamp(0.1, 2.0);

            // 5. Combine Forces & Apply Smooth Rotational Torque
            let net_torque = gravity_torque + photo_torque;
            self.angle += net_torque * flexibility * delta_time;
        }
    }
}
