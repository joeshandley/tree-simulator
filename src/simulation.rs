use crate::branch::{Branch, BranchState};
use crate::vector::{ANGLE_UP, Vector2D};

pub struct TreeSimulation {
    pub branches: Vec<Branch>,
    pub branch_angle_spread: f32, // Angle separation for child branches (radians)
    pub length_decay: f32,        // Multiplier for child branch max_length (e.g., 0.7)
    pub sun_azimuth: f32,         // Angle pointing TO the sun in radians
}

impl TreeSimulation {
    pub fn new(trunk_start: Vector2D, trunk_length: f32, trunk_thickness: f32) -> Self {
        let trunk = Branch::new(trunk_start, ANGLE_UP, trunk_length, trunk_thickness, None);

        TreeSimulation {
            branches: vec![trunk],
            branch_angle_spread: 40.0_f32.to_radians(),
            length_decay: 0.75,
            sun_azimuth: ANGLE_UP,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Convert the azimuth angle into a normalized directional vector
        let global_light_dir = Vector2D {
            x: self.sun_azimuth.cos(),
            y: self.sun_azimuth.sin(),
        };
        let num_branches = self.branches.len();
        let mut new_branches = Vec::new();

        for i in 0..num_branches {
            // Only growing branches need collision & light sampling
            if !matches!(self.branches[i].state, BranchState::Growing) {
                continue;
            }

            let tip = self.branches[i].end_point();
            let mut shadow_count = 0;
            let mut is_blocked = false;

            let parent_i = self.branches[i].parent_index;

            'inner: for j in 0..num_branches {
                // Skip checking against self
                if i == j {
                    continue;
                }

                let parent_j = self.branches[j].parent_index;

                // Skip direct parent, direct children, and siblings
                let is_parent = parent_i == Some(j);
                let is_child = parent_j == Some(i);
                let is_sibling = parent_i.is_some() && parent_i == parent_j;

                if is_parent || is_child || is_sibling {
                    continue 'inner;
                }

                // Skip branches that just spawned and have no physical length
                if self.branches[j].current_length <= f32::EPSILON {
                    continue 'inner;
                }

                let other_start = self.branches[j].start_point;
                let other_end = self.branches[j].end_point();

                // 1. Physical Collision Guard:
                // Stop growth if the tip comes within safety clearance of another branch
                let dist = tip.distance_to_segment(other_start, other_end);
                let min_clearance =
                    (self.branches[i].thickness + self.branches[j].thickness) * 0.75;

                if dist < min_clearance {
                    is_blocked = true;
                    break;
                }

                // 2. Canopy Occlusion (Raycast):
                // Check if another branch is directly above this tip blocking skylight
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

            // Now create a mutable version of the branch of index i
            let branch = &mut self.branches[i];

            // Apply physics with the dynamically calculated local light
            branch.update_physics(delta_time, global_light_dir, local_light_intensity);

            // Handle split logic when fully grown
            if branch.current_length >= branch.max_length {
                branch.current_length = branch.max_length; // Clamp

                let end_pos = branch.end_point();
                let child_length = branch.max_length * self.length_decay;
                let child_thickness = branch.thickness * 0.75;

                // Asymmetric spread: Primary leader shoot stays closer to parent angle, side shoot branches off
                let left_angle = branch.angle - self.branch_angle_spread;
                let right_angle = branch.angle + self.branch_angle_spread;

                if child_length > 5.0 {
                    let left_child =
                        Branch::new(end_pos, left_angle, child_length, child_thickness, Some(i));
                    let right_child =
                        Branch::new(end_pos, right_angle, child_length, child_thickness, Some(i));

                    let left_idx = num_branches + new_branches.len();
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
            } else if is_blocked {
                // The branch is physically blocked and permanently stunted.
                branch.state = BranchState::FullyGrown {
                    child_indices: Vec::new(),
                };
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
        let trunk = Branch::new(trunk_start, ANGLE_UP, trunk_length, trunk_thickness, None);
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
