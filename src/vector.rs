use std::f32::consts::FRAC_PI_2;

// -PI/2 radians points straight up in standard 2D graphics space
pub const ANGLE_UP: f32 = -FRAC_PI_2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }
}
