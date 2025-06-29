use glam::{IVec3, Vec3};
use crate::player::Player;
use crate::world::World;
use crate::physics::PLAYER_EYE_HEIGHT;
use crate::block::BlockType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockFace {
    PosX, // +X face (East)
    NegX, // -X face (West)
    PosY, // +Y face (Top)
    NegY, // -Y face (Bottom)
    PosZ, // +Z face (South)
    NegZ, // -Z face (North)
}

// Main raycasting function
pub fn cast_ray(
    player: &Player,
    world: &World,
    max_distance: f32,
) -> Option<(IVec3, BlockFace)> {
    let eye_position = player.position + Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0);
    let ray_direction = Vec3::new(
        player.yaw.cos() * player.pitch.cos(),
        player.pitch.sin(),
        player.yaw.sin() * player.pitch.cos(),
    ).normalize();

    let mut current_voxel_coord = IVec3::new(
        eye_position.x.floor() as i32,
        eye_position.y.floor() as i32,
        eye_position.z.floor() as i32,
    );

    let step_x = if ray_direction.x > 0.0 { 1 } else { -1 };
    let step_y = if ray_direction.y > 0.0 { 1 } else { -1 };
    let step_z = if ray_direction.z > 0.0 { 1 } else { -1 };

    // Avoid division by zero if ray_direction component is zero
    let t_delta_x = if ray_direction.x.abs() < 1e-6 { f32::MAX } else { (1.0 / ray_direction.x).abs() };
    let t_delta_y = if ray_direction.y.abs() < 1e-6 { f32::MAX } else { (1.0 / ray_direction.y).abs() };
    let t_delta_z = if ray_direction.z.abs() < 1e-6 { f32::MAX } else { (1.0 / ray_direction.z).abs() };

    let mut t_max_x = if ray_direction.x > 0.0 {
        (current_voxel_coord.x as f32 + 1.0 - eye_position.x) / ray_direction.x
    } else {
        (eye_position.x - current_voxel_coord.x as f32) / -ray_direction.x
    };
    let mut t_max_y = if ray_direction.y > 0.0 {
        (current_voxel_coord.y as f32 + 1.0 - eye_position.y) / ray_direction.y
    } else {
        (eye_position.y - current_voxel_coord.y as f32) / -ray_direction.y
    };
    let mut t_max_z = if ray_direction.z > 0.0 {
        (current_voxel_coord.z as f32 + 1.0 - eye_position.z) / ray_direction.z
    } else {
        (eye_position.z - current_voxel_coord.z as f32) / -ray_direction.z
    };

    // Handle cases where ray starts exactly on a boundary
    if t_max_x.is_nan() || t_max_x < 0.0 { t_max_x = t_delta_x; }
    if t_max_y.is_nan() || t_max_y < 0.0 { t_max_y = t_delta_y; }
    if t_max_z.is_nan() || t_max_z < 0.0 { t_max_z = t_delta_z; }


    let mut current_distance = 0.0;
    let mut last_face: BlockFace; // Declare, will be assigned in loop before use if a block is hit

    // Initial check for the block the player is standing in
    // (or rather, the block the eye is in)
    if let Some(block) = world.get_block_at_world(
        current_voxel_coord.x as f32,
        current_voxel_coord.y as f32,
        current_voxel_coord.z as f32,
    ) {
        if block.block_type != BlockType::Air {
            // Cannot select the block the eye is inside. This case is tricky.
            // For now, we assume the first step will take us out of it.
            // Or, we could determine a face based on ray direction from center of this block.
            // However, standard voxel traversal starts by *entering* the first voxel.
        }
    }


    while current_distance < max_distance {
        //let entered_face; // This variable is removed
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                current_distance = t_max_x;
                current_voxel_coord.x += step_x;
                t_max_x += t_delta_x;
                last_face = if step_x > 0 { BlockFace::NegX } else { BlockFace::PosX }; // Assign directly
            } else {
                current_distance = t_max_z;
                current_voxel_coord.z += step_z;
                t_max_z += t_delta_z;
                last_face = if step_z > 0 { BlockFace::NegZ } else { BlockFace::PosZ }; // Assign directly
            }
        } else {
            if t_max_y < t_max_z {
                current_distance = t_max_y;
                current_voxel_coord.y += step_y;
                t_max_y += t_delta_y;
                last_face = if step_y > 0 { BlockFace::NegY } else { BlockFace::PosY }; // Assign directly
            } else {
                current_distance = t_max_z;
                current_voxel_coord.z += step_z;
                t_max_z += t_delta_z;
                last_face = if step_z > 0 { BlockFace::NegZ } else { BlockFace::PosZ }; // Assign directly
            }
        }
        // last_face = entered_face; // This line is now removed

        if current_distance > max_distance {
            return None;
        }

        // Check block at current_voxel_coord
        // World coordinates for get_block_at_world can be any point within the block,
        // so using the corner (current_voxel_coord) is fine.
        if let Some(block) = world.get_block_at_world(
            current_voxel_coord.x as f32,
            current_voxel_coord.y as f32,
            current_voxel_coord.z as f32,
        ) {
            if block.block_type != BlockType::Air { // Found a solid block
                return Some((current_voxel_coord, last_face));
            }
        } else {
            // Ray went into an unloaded/undefined part of the world. Treat as miss.
            return None;
        }
    }

    None // No block hit within max_distance
}
