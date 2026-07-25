use crate::branch::{Branch, BranchState};
use crate::vector::{ANGLE_UP, Vector2D};

pub struct TreeSimulation {
    pub branches: Vec<Branch>,
    pub branch_angle_spread: f32, // Angle separation for child branches (radians)
    pub length_decay: f32,        // Multiplier for child branch max_length (e.g., 0.7)
    pub global_sun_direction: Vector2D, // E.g., Vector2D { x: 0.2, y: -1.0 } pointing up and slightly right
}

impl TreeSimulation {
    pub fn new(trunk_start: Vector2D, trunk_length: f32, trunk_thickness: f32) -> Self {
        let trunk = Branch::new(trunk_start, ANGLE_UP, trunk_length, trunk_thickness);

        TreeSimulation {
            branches: vec![trunk],
            branch_angle_spread: 0.5,
            length_decay: 0.75,
            global_sun_direction: Vector2D { x: 0.0, y: -1.0 },
            // global_sun_direction: Vector2D { x: 0.1, y: -1.0 }, // Overhead sun with slight tilt
        }
    }

    /// Evaluates environmental conditions at a given point in space.
    /// In a simple setup, this returns global sun. In an advanced setup,
    /// this samples a light grid to account for self-shading.
    fn sample_light_at(&self, position: Vector2D) -> (Vector2D, f32) {
        // Placeholder for future light-grid/shadow calculation:
        // Returns (Light Direction Vector, Light Intensity [0.0 - 1.0])
        (self.global_sun_direction, 1.0)
    }

    pub fn update(&mut self, delta_time: f32) {
        let initial_count = self.branches.len();

        // -----------------------------------------------------------------
        // PASS 1: Read-Only Environmental Phase (Immutable borrow of `self`)
        // -----------------------------------------------------------------
        let light_data: Vec<(Vector2D, f32)> = (0..initial_count)
            .map(|i| {
                if let BranchState::Growing = self.branches[i].state {
                    self.sample_light_at(self.branches[i].end_point())
                } else {
                    (Vector2D { x: 0.0, y: 0.0 }, 0.0)
                }
            })
            .collect();

        // -----------------------------------------------------------------
        // PASS 2: Physics Update & Forking Phase (Mutable borrow of `self.branches`)
        // -----------------------------------------------------------------
        let mut new_branches = Vec::new();

        // Pass 1: Physical Updates & Spawning Children
        for index in 0..initial_count {
            let branch = &mut self.branches[index];

            if let BranchState::Growing = branch.state {
                // Apply light context calculated in Pass 1
                let (light_dir, light_intensity) = light_data[index];
                branch.update_physics(delta_time, light_dir, light_intensity);

                // Handle split logic when fully grown
                if branch.current_length >= branch.max_length {
                    branch.current_length = branch.max_length; // Clamp

                    let end_pos = branch.end_point();
                    let child_length = branch.max_length * self.length_decay;
                    let child_thickness = branch.thickness * 0.75;

                    // Asymmetric spread: Primary leader shoot stays closer to parent angle, side shoot branches off
                    let left_angle = branch.angle - (self.branch_angle_spread * 0.7);
                    let right_angle = branch.angle + (self.branch_angle_spread * 1.1);

                    if child_length > 5.0 {
                        let left_child =
                            Branch::new(end_pos, left_angle, child_length, child_thickness);
                        let right_child =
                            Branch::new(end_pos, right_angle, child_length, child_thickness);

                        let left_idx = initial_count + new_branches.len();
                        let right_idx = left_idx + 1;

                        branch.state = BranchState::FullyGrown {
                            child_indices: vec![left_idx, right_idx],
                        };

                        new_branches.push(left_child);
                        new_branches.push(right_child);
                    } else {
                        // Terminal branches (smallest branch threshold reached)
                        // also need to transition out of Growing state!
                        branch.state = BranchState::FullyGrown {
                            child_indices: Vec::new(),
                        };
                    }
                }
            }
        }

        // Pass 2: Append newly created branches
        self.branches.extend(new_branches);

        // -----------------------------------------------------------------
        // PASS 3: Position Propagation Phase
        // -----------------------------------------------------------------
        self.propagate_positions();
    }

    pub fn is_finished(&self) -> bool {
        // `matches!` checks the variant type and ignores `child_indices`
        self.branches
            .iter()
            .all(|branch| matches!(branch.state, BranchState::FullyGrown { .. }))
    }

    pub fn reset(&mut self, trunk_start: Vector2D, trunk_length: f32, trunk_thickness: f32) {
        let trunk = Branch::new(trunk_start, ANGLE_UP, trunk_length, trunk_thickness);
        self.branches = vec![trunk];
    }

    /// Ensures child branches remain attached to their parent's moving end points.
    fn propagate_positions(&mut self) {
        for i in 0..self.branches.len() {
            if let BranchState::FullyGrown { ref child_indices } = self.branches[i].state.clone() {
                let parent_end = self.branches[i].end_point();
                for &child_idx in child_indices {
                    self.branches[child_idx].start_point = parent_end;
                }
            }
        }
    }
}
