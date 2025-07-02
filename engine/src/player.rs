use glam::Vec3;
use crate::physics::{AABB, PLAYER_HEIGHT, PLAYER_HALF_WIDTH, GRAVITY, JUMP_FORCE, WALK_SPEED, FRICTION_COEFFICIENT};
// Remove direct dependency on Chunk, will use World instead
// use crate::chunk::Chunk;
use crate::world::World; // Import World

// Input state for player movement intentions
// This will be populated by the input handling system
#[derive(Debug, Default, Clone, Copy)]
pub struct PlayerMovementIntention {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    // Potentially add sprint, crouch flags here later
}

pub struct Player {
    pub position: Vec3, // Position of the player's feet, centered horizontally
    pub velocity: Vec3,
    pub local_bounding_box: AABB, // Relative to player's position (feet)
    pub on_ground: bool,

    // Camera orientation fields
    pub yaw: f32,   // Radians. Rotation around the Y axis (vertical)
    pub pitch: f32, // Radians. Rotation around the X axis (horizontal)
    pub mouse_sensitivity: f32, // Added sensitivity here

    // Movement intention state
    pub movement_intention: PlayerMovementIntention,
}

impl Player {
    pub fn new(initial_position: Vec3, initial_yaw: f32, initial_pitch: f32, mouse_sensitivity: f32) -> Self {
        Self {
            position: initial_position,
            velocity: Vec3::ZERO,
            local_bounding_box: AABB {
                min: Vec3::new(-PLAYER_HALF_WIDTH, 0.0, -PLAYER_HALF_WIDTH),
                max: Vec3::new(PLAYER_HALF_WIDTH, PLAYER_HEIGHT, PLAYER_HALF_WIDTH),
            },
            on_ground: false,
            yaw: initial_yaw,
            pitch: initial_pitch,
            mouse_sensitivity,
            movement_intention: PlayerMovementIntention::default(),
        }
    }

    // Placeholder for processing mouse movement, logic will be moved from CameraController
    pub fn process_mouse_movement(&mut self, delta_x: f64, delta_y: f64) {
        let delta_x = delta_x as f32 * self.mouse_sensitivity;
        let delta_y = delta_y as f32 * self.mouse_sensitivity;

        self.yaw += delta_x;
        self.pitch -= delta_y; // Inverted because y-coordinates usually go from top to bottom in window systems

        // Clamp pitch to avoid flipping and looking too far up/down
        const MAX_PITCH: f32 = 89.0f32.to_radians();
        const MIN_PITCH: f32 = -89.0f32.to_radians();
        self.pitch = self.pitch.clamp(MIN_PITCH, MAX_PITCH);

        // Normalize yaw to keep it within 0 to 2*PI range (optional, but can be tidy)
        // self.yaw = self.yaw.rem_euclid(2.0 * std::f32::consts::PI);
    }

    // Placeholder for the main physics update logic
    // pub fn update_physics_and_collision(&mut self, dt: f32, chunk: &crate::chunk::Chunk) {
    // pub fn update_physics_and_collision(&mut self, dt: f32, chunk: &Chunk) { // Old signature
    pub fn update_physics_and_collision(&mut self, dt: f32, world: &World) { // New signature with World
        // 1. Apply Inputs & Intentions
        let mut intended_horizontal_velocity = Vec3::ZERO;
        // let forward_direction = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize_or_zero(); // Unused
        // let right_direction = Vec3::new(-self.yaw.sin(), 0.0, self.yaw.cos()).normalize_or_zero(); // Unused
                                                                                                    // Standard: positive yaw rotates counter-clockwise from +X towards +Z.
                                                                                                    // If yaw=0 is +X, then forward is (cos(yaw), 0, sin(yaw)). Right is (sin(yaw), 0, -cos(yaw)) or (cos(yaw-PI/2), 0, sin(yaw-PI/2))
                                                                                                    // Let's assume forward is (cos(yaw), 0, sin(yaw)).
                                                                                                    // Right vector is then (forward.z, 0, -forward.x) which is (sin(yaw), 0, -cos(yaw)).
                                                                                                    // No, standard right hand rule: Y-up, X-right, Z-forward from camera.
                                                                                                    // If player yaw is rotation around Y:
                                                                                                    // Yaw = 0 means looking along +X. Forward = (1,0,0). Right = (0,0,-1)
                                                                                                    // Yaw = PI/2 means looking along +Z. Forward = (0,0,1). Right = (1,0,0)
                                                                                                    // My current camera code: yaw: -90.0f32.to_radians(), // Initialize to look along -Z
                                                                                                    // front = Vec3::new(self.yaw.cos() * self.pitch.cos(), self.pitch.sin(), self.yaw.sin() * self.pitch.cos())
                                                                                                    // If yaw = -PI/2 (looking -Z): cos(-PI/2)=0, sin(-PI/2)=-1. Forward dir on XZ is (0, -1).
                                                                                                    // This means +Z is to the left. +X is forward. This is a common setup.
                                                                                                    // Let's stick to the current camera's convention for "front" based on yaw.
                                                                                                    // Front vector (horizontal part) based on yaw:
        let horizontal_forward = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize_or_zero();


        if self.movement_intention.forward {
            intended_horizontal_velocity += horizontal_forward;
        }
        if self.movement_intention.backward {
            intended_horizontal_velocity -= horizontal_forward;
        }
        // For right/left, we need the right vector.
        // If forward is (cos(yaw), 0, sin(yaw)), then right is (sin(yaw), 0, -cos(yaw))
        // No, if forward = (fx, 0, fz), right = (fz, 0, -fx) for Y-up.
        // Example: forward = (1,0,0) (yaw=0), right should be (0,0,-1).  sin(0)=0, -cos(0)=-1. Correct.
        // Example: forward = (0,0,1) (yaw=PI/2), right should be (1,0,0). sin(PI/2)=1, -cos(PI/2)=0. Correct.
        let horizontal_right = Vec3::new(horizontal_forward.z, 0.0, -horizontal_forward.x);


        if self.movement_intention.left {
            intended_horizontal_velocity += horizontal_right;
        }
        if self.movement_intention.right {
            intended_horizontal_velocity -= horizontal_right;
        }

        if intended_horizontal_velocity.length_squared() > 0.0 {
            intended_horizontal_velocity = intended_horizontal_velocity.normalize() * WALK_SPEED;
            self.velocity.x = intended_horizontal_velocity.x;
            self.velocity.z = intended_horizontal_velocity.z;
        } else {
            // Apply friction
            self.velocity.x *= (1.0 - FRICTION_COEFFICIENT * dt / (1.0/60.0) ).max(0.0); // rudimentary friction, assumes 60fps for coeff meaning
            self.velocity.z *= (1.0 - FRICTION_COEFFICIENT * dt / (1.0/60.0) ).max(0.0);
            // A better friction: self.velocity.x -= self.velocity.x * FRICTION_COEFFICIENT * dt;
            // Or even better, decelerate to zero:
            let speed = (self.velocity.x * self.velocity.x + self.velocity.z * self.velocity.z).sqrt();
            if speed > 0.01 { // some threshold
                let drop = speed * FRICTION_COEFFICIENT * dt; // this is more like drag
                let scale = (speed - drop).max(0.0) / speed;
                self.velocity.x *= scale;
                self.velocity.z *= scale;
            } else {
                self.velocity.x = 0.0;
                self.velocity.z = 0.0;
            }
        }

        // 2. Apply Gravity
        self.velocity.y -= GRAVITY * dt;

        // 3. Handle Jumping
        if self.movement_intention.jump && self.on_ground {
            self.velocity.y = JUMP_FORCE;
            self.on_ground = false; // Will be re-evaluated during Y-collision
        }
        // Reset jump intention so it's not sticky
        self.movement_intention.jump = false;


        // 4. Collision Detection and Resolution (Axis-by-Axis)
        let mut desired_move = self.velocity * dt;
        self.on_ground = false; // Reset before Y-axis collision check

        // --- Y-AXIS COLLISION ---
        self.position.y += desired_move.y;
        let mut player_world_box = self.get_world_bounding_box();
        // let nearby_y_blocks = get_nearby_block_aabbs(&player_world_box, chunk); // Old call
        let nearby_y_blocks = get_nearby_block_aabbs(&player_world_box, world); // New call with world

        for block_box in nearby_y_blocks {
            if player_world_box.intersects(&block_box) {
                if desired_move.y > 0.0 { // Moving up (hit ceiling)
                    self.position.y = block_box.min.y - self.local_bounding_box.max.y - 0.0001; // Subtract small epsilon
                } else { // Moving down (hit floor)
                    self.position.y = block_box.max.y - self.local_bounding_box.min.y + 0.0001; // Add small epsilon
                    self.on_ground = true;
                }
                self.velocity.y = 0.0;
                desired_move.y = 0.0; // No further movement on this axis this frame
                // player_world_box = self.get_world_bounding_box(); // Removed: Unused assignment as per compiler warning.
                                                                  // The important update to player_world_box is at the start of the next axis processing.
                break;
            }
        }

        // --- X-AXIS COLLISION ---
        self.position.x += desired_move.x;
        player_world_box = self.get_world_bounding_box(); // Update box for X-movement, using Y-resolved position
        // let nearby_x_blocks = get_nearby_block_aabbs(&player_world_box, chunk); // Old call
        let nearby_x_blocks = get_nearby_block_aabbs(&player_world_box, world); // New call with world

        for block_box in nearby_x_blocks {
            if player_world_box.intersects(&block_box) {
                if desired_move.x > 0.0 { // Moving right
                    self.position.x = block_box.min.x - self.local_bounding_box.max.x - 0.0001;
                } else { // Moving left
                    self.position.x = block_box.max.x - self.local_bounding_box.min.x + 0.0001;
                }
                self.velocity.x = 0.0;
                // desired_move.x = 0.0;
                // player_world_box = self.get_world_bounding_box(); // Removed: Unused assignment as per compiler warning.
                                                                  // The important update to player_world_box is at the start of the next axis processing.
                break;
            }
        }

        // --- Z-AXIS COLLISION ---
        self.position.z += desired_move.z;
        player_world_box = self.get_world_bounding_box(); // Update box for Z-movement
        // let nearby_z_blocks = get_nearby_block_aabbs(&player_world_box, chunk); // Old call
        let nearby_z_blocks = get_nearby_block_aabbs(&player_world_box, world); // New call with world

        for block_box in nearby_z_blocks {
            if player_world_box.intersects(&block_box) {
                if desired_move.z > 0.0 { // Moving "forward" relative to world +Z (e.g. larger Z values)
                    self.position.z = block_box.min.z - self.local_bounding_box.max.z - 0.0001;
                } else { // Moving "backward" relative to world +Z
                    self.position.z = block_box.max.z - self.local_bounding_box.min.z + 0.0001;
                }
                self.velocity.z = 0.0;
                // player_world_box = self.get_world_bounding_box(); // This assignment was unused before break and end of function.
                break;
            }
        }
    }

    pub fn get_world_bounding_box(&self) -> AABB {
        AABB {
            min: self.position + self.local_bounding_box.min,
            max: self.position + self.local_bounding_box.max,
        }
    }
}

// Helper function to get AABBs of solid blocks near the player
// This function now queries the World instead of a single Chunk.
fn get_nearby_block_aabbs(player_world_box: &AABB, world: &World) -> Vec<AABB> {
    let mut nearby_blocks = Vec::new();

    // Determine the range of world block coordinates that the player's AABB might overlap.
    // Add a small buffer (e.g., 1 block) just in case.
    // These are absolute world block coordinates.
    let min_world_block_x = (player_world_box.min.x - 1.0).floor() as i32;
    let max_world_block_x = (player_world_box.max.x + 1.0).ceil() as i32;
    let min_world_block_y = (player_world_box.min.y - 1.0).floor() as i32;
    let max_world_block_y = (player_world_box.max.y + 1.0).ceil() as i32;
    let min_world_block_z = (player_world_box.min.z - 1.0).floor() as i32;
    let max_world_block_z = (player_world_box.max.z + 1.0).ceil() as i32;

    for world_bx in min_world_block_x..max_world_block_x {
        for world_by in min_world_block_y..max_world_block_y {
            for world_bz in min_world_block_z..max_world_block_z {
                // Y coordinates for blocks are absolute from 0 upwards.
                // We can pre-filter Y here if it's outside the general world height,
                // though world.get_block_at_world also handles Y bounds.
                if world_by < 0 || world_by >= crate::chunk::CHUNK_HEIGHT as i32 {
                    continue;
                }

                // Use world.get_block_at_world to get block data
                // This method handles chunk boundaries internally.
                let current_block_world_pos = glam::ivec3(world_bx, world_by, world_bz);
                if let Some(block) = world.get_block_at_world(current_block_world_pos) {
                    if block.is_solid() {
                        // The AABB for the block should be in world coordinates
                        let block_min_corner = Vec3::new(world_bx as f32, world_by as f32, world_bz as f32);
                        let block_max_corner = Vec3::new(world_bx as f32 + 1.0, world_by as f32 + 1.0, world_bz as f32 + 1.0);
                        nearby_blocks.push(AABB::new(block_min_corner, block_max_corner));
                    }
                }
            }
        }
    }
    nearby_blocks
}
