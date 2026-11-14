// vector.rs
use std::f32::consts::FRAC_PI_2;

/// -PI/2 radians points straight up in standard 2D graphics space (where +Y points down).
pub const ANGLE_UP: f32 = -FRAC_PI_2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    /// Creates a new 2D vector with the specified X and Y coordinates.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Calculates the angle of the vector in radians relative to the positive X-axis (`atan2(y, x)`).
    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Calculates the dot product between `self` and `rhs` (`x1*x2 + y1*y2`).
    ///
    /// Indicates alignment: positive if pointing generally together, zero if perpendicular, negative if opposing.
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    /// Returns the squared length ($x^2 + y^2$) of the vector.
    ///
    /// Cheaper than standard length because it avoids an expensive square root (`sqrt()`).
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Calculates the shortest distance from this point (`self`) to a line segment defined by `a` and `b`.
    ///
    /// Projects the point onto segment `[a, b]` using dot product projection, clamped between `0.0` and `1.0`.
    pub fn distance_to_segment(self, a: Vector2D, b: Vector2D) -> f32 {
        let ab = b - a;
        let ap = self - a;
        let ab_len_sq = ab.length_squared();

        if ab_len_sq == 0.0 {
            return (self - a).length_squared().sqrt();
        }

        let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
        let closest_point = Vector2D {
            x: a.x + t * ab.x,
            y: a.y + t * ab.y,
        };

        (self - closest_point).length_squared().sqrt()
    }

    /// Tests if an infinite ray starting at `origin` in direction `dir` intersects line segment `[seg_start, seg_end]`.
    ///
    /// Returns `true` if the ray hits the segment ahead of the ray origin.
    pub fn ray_intersects_segment(
        origin: Vector2D,
        dir: Vector2D,
        seg_start: Vector2D,
        seg_end: Vector2D,
    ) -> bool {
        let v1 = origin - seg_start;
        let v2 = seg_end - seg_start;
        let v3 = Vector2D {
            x: -dir.y,
            y: dir.x,
        };

        let dot = v2.dot(v3);
        if dot.abs() < 1e-6 {
            return false;
        }

        let t1 = (v2.x * v1.y - v2.y * v1.x) / dot;
        let t2 = v1.dot(v3) / dot;

        t1 >= 0.0 && (0.0..=1.0).contains(&t2)
    }
}

/// `a - b` is the vector pointing from `b` to `a`.
impl std::ops::Sub for Vector2D {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    // Helper constant for floating-point tolerance (epsilon)
    const EPSILON: f32 = 1e-5;

    /// Helper assertion function to compare floats safely.
    /// In TS, you might use `expect(val).toBeCloseTo(expected)`.
    fn assert_float_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "Expected {}, got {}",
            expected,
            actual
        );
    }

    #[test]
    fn test_new_and_fields() {
        let v = Vector2D::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn test_angle() {
        assert_float_eq(Vector2D::new(1.0, 0.0).angle(), 0.0);
        assert_float_eq(Vector2D::new(0.0, 1.0).angle(), FRAC_PI_2);
        assert_float_eq(Vector2D::new(0.0, -1.0).angle(), ANGLE_UP);
        assert_float_eq(Vector2D::new(-1.0, 0.0).angle(), PI);
    }

    #[test]
    fn test_sub() {
        let v1 = Vector2D::new(5.0, 7.0);
        let v2 = Vector2D::new(2.0, 3.0);

        // Exact integer math represented as floats is exact, so assert_eq! works fine here
        assert_eq!(v1 - v2, Vector2D::new(3.0, 4.0));
    }

    #[test]
    fn test_dot_product() {
        let v1 = Vector2D::new(2.0, 3.0);
        let v2 = Vector2D::new(4.0, -1.0);

        // Perpendicular vectors -> 0.0
        let perp1 = Vector2D::new(1.0, 0.0);
        let perp2 = Vector2D::new(0.0, 1.0);

        assert_eq!(v1.dot(v2), 5.0); // (2*4) + (3*-1) = 8 - 3 = 5
        assert_eq!(perp1.dot(perp2), 0.0);
    }

    #[test]
    fn test_length_squared() {
        let v = Vector2D::new(3.0, 4.0);
        assert_eq!(v.length_squared(), 25.0);
    }

    mod distance_to_segment_tests {
        use super::*;

        #[test]
        fn test_point_projects_onto_middle_of_segment() {
            let p = Vector2D::new(2.0, 2.0);
            let a = Vector2D::new(0.0, 0.0);
            let b = Vector2D::new(4.0, 0.0);

            // Point (2,2) is directly 2 units above the horizontal segment [(0,0), (4,0)]
            assert_float_eq(p.distance_to_segment(a, b), 2.0);
        }

        #[test]
        fn test_point_closest_to_endpoint_a() {
            let p = Vector2D::new(-2.0, 3.0);
            let a = Vector2D::new(0.0, 0.0);
            let b = Vector2D::new(4.0, 0.0);

            // Distance should be directly to 'a' (0,0), which forms a 3-4-5 triangle
            assert_float_eq(
                p.distance_to_segment(a, b),
                ((-2.0_f32).powi(2) + 3.0_f32.powi(2)).sqrt(),
            );
        }

        #[test]
        fn test_point_closest_to_endpoint_b() {
            let p = Vector2D::new(6.0, 0.0);
            let a = Vector2D::new(0.0, 0.0);
            let b = Vector2D::new(4.0, 0.0);

            // Distance should be directly to 'b' (4,0)
            assert_float_eq(p.distance_to_segment(a, b), 2.0);
        }

        #[test]
        fn test_degrade_to_point_when_segment_zero_length() {
            let p = Vector2D::new(3.0, 4.0);
            let a = Vector2D::new(0.0, 0.0);
            let b = Vector2D::new(0.0, 0.0); // Degenerate segment (point)

            assert_float_eq(p.distance_to_segment(a, b), 5.0);
        }
    }

    mod ray_intersects_segment_tests {
        use super::*;

        #[test]
        fn test_direct_hit() {
            let origin = Vector2D::new(0.0, 0.0);
            let dir = Vector2D::new(1.0, 0.0); // Pointing right

            let seg_start = Vector2D::new(5.0, -2.0);
            let seg_end = Vector2D::new(5.0, 2.0); // Vertical wall at X = 5

            assert!(Vector2D::ray_intersects_segment(
                origin, dir, seg_start, seg_end
            ));
        }

        #[test]
        fn test_miss_pointing_away() {
            let origin = Vector2D::new(0.0, 0.0);
            let dir = Vector2D::new(-1.0, 0.0); // Pointing left (away from wall)

            let seg_start = Vector2D::new(5.0, -2.0);
            let seg_end = Vector2D::new(5.0, 2.0);

            assert!(!Vector2D::ray_intersects_segment(
                origin, dir, seg_start, seg_end
            ));
        }

        #[test]
        fn test_miss_parallel_ray_and_segment() {
            let origin = Vector2D::new(0.0, 0.0);
            let dir = Vector2D::new(1.0, 0.0); // Pointing right

            let seg_start = Vector2D::new(2.0, 1.0);
            let seg_end = Vector2D::new(6.0, 1.0); // Horizontal line parallel to ray

            assert!(!Vector2D::ray_intersects_segment(
                origin, dir, seg_start, seg_end
            ));
        }
    }
}
