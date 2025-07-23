use glam::Vec3;

// Physics Constants
pub const GRAVITY: f32 = 9.81 * 2.8; // m/s^2, doubled for more "gamey" feel
pub const JUMP_FORCE: f32 = 8.0; // m/s
pub const WALK_SPEED: f32 = 4.0; // m/s
pub const FRICTION_COEFFICIENT: f32 = 0.8; // Dimensionless, used to scale velocity down

// Player Dimensions
pub const PLAYER_WIDTH: f32 = 0.6; // meters
pub const PLAYER_HEIGHT: f32 = 1.8; // meters
pub const PLAYER_EYE_HEIGHT: f32 = 1.6; // meters, from feet
pub const PLAYER_HALF_WIDTH: f32 = PLAYER_WIDTH / 2.0;

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    // Method to check if this AABB intersects with another one.
    pub fn intersects(&self, other: &AABB) -> bool {
        (self.min.x < other.max.x && self.max.x > other.min.x)
            && (self.min.y < other.max.y && self.max.y > other.min.y)
            && (self.min.z < other.max.z && self.max.z > other.min.z)
    }
}
