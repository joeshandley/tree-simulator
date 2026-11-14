//! Tree metrics for the measurement harness (plan 0002, D11). Pure functions of
//! the section arena, so they can be tested on hand-built trees.

use crate::growth_section::{GrowthSection, GrowthSectionState};

/// Shape of one tree. Definitions are fixed by plan 0002 (D11); changing one
/// breaks comparison with older baselines, so note it in `docs/metrics/`.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeMetrics {
    pub sections: usize,
    /// Fully grown sections with no children.
    pub tips: usize,
    /// Base of the trunk to the highest end point (screen +Y is down).
    pub height: f32,
    /// Horizontal span of all section end points.
    pub crown_width: f32,
    /// Mean angle between the two children at each fork, in degrees.
    pub mean_fork_angle_deg: Option<f32>,
    /// Mean of parent thickness² ÷ sum of children's thickness² at each fork.
    /// 1.0 means cross-section area is conserved (Leonardo's rule).
    pub mean_leonardo_ratio: Option<f32>,
}

pub fn measure(sections: &[GrowthSection]) -> TreeMetrics {
    let base = sections.first().map(|s| s.start_point);
    let ends: Vec<_> = sections.iter().map(|s| s.end_point()).collect();
    let height = base.map_or(0.0, |b| ends.iter().map(|e| b.y - e.y).fold(0.0, f32::max));
    let crown_width = if ends.is_empty() {
        0.0
    } else {
        let min_x = ends.iter().map(|e| e.x).fold(f32::INFINITY, f32::min);
        let max_x = ends.iter().map(|e| e.x).fold(f32::NEG_INFINITY, f32::max);
        max_x - min_x
    };

    let mut tips = 0;
    let (mut angles, mut ratios) = (Vec::new(), Vec::new());
    for section in sections {
        if let GrowthSectionState::FullyGrown { child_indices } = &section.state {
            match child_indices.as_slice() {
                [] => tips += 1,
                [a, b] => {
                    let (a, b) = (&sections[*a], &sections[*b]);
                    angles.push(angle_between(a.angle, b.angle).to_degrees());
                    let children = a.thickness.powi(2) + b.thickness.powi(2);
                    ratios.push(section.thickness.powi(2) / children);
                }
                _ => {}
            }
        }
    }

    TreeMetrics {
        sections: sections.len(),
        tips,
        height,
        crown_width,
        mean_fork_angle_deg: mean(&angles),
        mean_leonardo_ratio: mean(&ratios),
    }
}

/// Smallest angle between two headings, in radians, within [0, π].
fn angle_between(a: f32, b: f32) -> f32 {
    let diff = (a - b).rem_euclid(std::f32::consts::TAU);
    diff.min(std::f32::consts::TAU - diff)
}

fn mean(values: &[f32]) -> Option<f32> {
    (!values.is_empty()).then(|| values.iter().sum::<f32>() / values.len() as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::growth_section::GrowthSection;
    use crate::vector::{ANGLE_UP, Vector2D};

    fn frozen(start: Vector2D, angle: f32, length: f32, thickness: f32, parent: Option<usize>, children: Vec<usize>) -> GrowthSection {
        let mut s = GrowthSection::new(start, angle, length, thickness, parent);
        s.current_length = length;
        s.freeze(children);
        s
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn straight_trunk() {
        let tree = vec![frozen(Vector2D::new(0.0, 0.0), ANGLE_UP, 10.0, 2.0, None, vec![])];
        let m = measure(&tree);
        assert_eq!((m.sections, m.tips), (1, 1));
        assert!(close(m.height, 10.0), "height {}", m.height);
        assert!(close(m.crown_width, 0.0), "width {}", m.crown_width);
        assert_eq!(m.mean_fork_angle_deg, None);
        assert_eq!(m.mean_leonardo_ratio, None);
    }

    #[test]
    fn single_fork() {
        let base = Vector2D::new(0.0, 0.0);
        let fork_point = Vector2D::new(0.0, -10.0);
        let tree = vec![
            frozen(base, ANGLE_UP, 10.0, 2.0, None, vec![1, 2]),
            frozen(fork_point, ANGLE_UP - 0.5, 5.0, 1.0, Some(0), vec![]),
            frozen(fork_point, ANGLE_UP + 0.5, 5.0, 1.0, Some(0), vec![]),
        ];
        let m = measure(&tree);
        assert_eq!((m.sections, m.tips), (3, 2));
        assert!(close(m.height, 10.0 + 5.0 * 0.5_f32.cos()), "height {}", m.height);
        assert!(close(m.crown_width, 10.0 * 0.5_f32.sin()), "width {}", m.crown_width);
        assert!(close(m.mean_fork_angle_deg.unwrap(), 1.0_f32.to_degrees()));
        // Parent area 2² = 4 over children 1² + 1² = 2.
        assert!(close(m.mean_leonardo_ratio.unwrap(), 2.0));
    }

    #[test]
    fn fork_angle_wraps_round_the_circle() {
        assert!(close(angle_between(0.1, std::f32::consts::TAU - 0.1), 0.2));
        assert!(close(angle_between(-3.0, 3.0), std::f32::consts::TAU - 6.0));
    }
}
