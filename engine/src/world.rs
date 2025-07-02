use std::collections::HashMap;
use crate::chunk::{Chunk, CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH};
use crate::block::Block; // Removed BlockType as it's unused

pub struct World {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl World {
    pub fn new() -> Self {
        World {
            chunks: HashMap::new(),
        }
    }

    // Gets a reference to a chunk if it exists, otherwise generates/loads it.
    pub fn get_or_create_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &mut Chunk {
        self.chunks
            .entry((chunk_x, chunk_z))
            .or_insert_with(|| {
                let mut new_chunk = Chunk::new(chunk_x, chunk_z);
                new_chunk.generate_terrain(); // Or some other generation logic
                new_chunk
            })
    }

    // Gets an immutable reference to a chunk if it exists.
    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        self.chunks.get(&(chunk_x, chunk_z))
    }

    // Converts world block coordinates to chunk coordinates and local block coordinates.
    pub fn world_to_chunk_coords(world_x: f32, world_y: f32, world_z: f32) -> ((i32, i32), (usize, usize, usize)) {
        let chunk_x = (world_x / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_z = (world_z / CHUNK_DEPTH as f32).floor() as i32;

        let local_x = ((world_x % CHUNK_WIDTH as f32) + CHUNK_WIDTH as f32) % CHUNK_WIDTH as f32;
        let _local_y = world_y; // Assuming y is absolute for now, or chunks are full height slices. Prefixed as unused.
        let local_z = ((world_z % CHUNK_DEPTH as f32) + CHUNK_DEPTH as f32) % CHUNK_DEPTH as f32;

        // Clamping y to be within chunk height. This might need adjustment based on how
        // world_y interacts with chunk vertical slices if that becomes a feature.
        let clamped_y = world_y.max(0.0).min(CHUNK_HEIGHT as f32 - 1.0);

        ((chunk_x, chunk_z), (local_x as usize, clamped_y as usize, local_z as usize))
    }

    // Gets a block at absolute world coordinates.
    pub fn get_block_at_world(&self, world_x: f32, world_y: f32, world_z: f32) -> Option<&Block> {
        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) = World::world_to_chunk_coords(world_x, world_y, world_z);

        if local_y >= CHUNK_HEIGHT { // Check against actual CHUNK_HEIGHT
            return None; // y is out of any possible chunk's bounds
        }

        match self.get_chunk(chunk_x, chunk_z) {
            Some(chunk) => chunk.get_block(local_x, local_y, local_z),
            None => None, // Chunk doesn't exist
        }
    }

    // Note: A `set_block_at_world` method would be similar but would need mutable access
    // and potentially create the chunk if it doesn't exist.
    // pub fn set_block_at_world(&mut self, world_x: f32, world_y: f32, world_z: f32, block_type: BlockType) -> Result<(), &'static str> { // Old signature
    // New set_block method using IVec3 and returning modified chunk coordinates
    // Returns the world chunk coordinates (cx, cz) of the modified chunk if successful.
    pub fn set_block(&mut self, world_block_pos: glam::IVec3, block_type: crate::block::BlockType) -> Result<(i32, i32), &'static str> {
        if world_block_pos.y < 0 || world_block_pos.y >= CHUNK_HEIGHT as i32 {
            return Err("Y coordinate out of world bounds");
        }

        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) =
            World::world_to_chunk_coords(world_block_pos.x as f32, world_block_pos.y as f32, world_block_pos.z as f32);

        // Ensure local_y is also valid after conversion, though the initial check should mostly cover it.
        if local_y >= CHUNK_HEIGHT {
             // This case should ideally not be hit if the initial Y check is correct
             // and world_to_chunk_coords handles y correctly.
            return Err("Calculated local Y coordinate out of chunk bounds");
        }

        // Check if the block being replaced is Bedrock
        if let Some(chunk) = self.get_chunk(chunk_x, chunk_z) {
            if let Some(existing_block) = chunk.get_block(local_x, local_y, local_z) {
                if existing_block.block_type == crate::block::BlockType::Bedrock {
                    return Err("Cannot replace Bedrock");
                }
            }
        }

        let chunk = self.get_or_create_chunk(chunk_x, chunk_z);
        match chunk.set_block(local_x, local_y, local_z, block_type) {
            Ok(_) => {
                // After setting a block, recalculate sunlight and propagate light for this chunk.
                chunk.calculate_initial_sunlight();
                chunk.propagate_light(); // Add this call
                Ok((chunk_x, chunk_z))
            }
            Err(e) => Err(e), // Propagate error from chunk.set_block
        }
    }
}

// Default implementation for World
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
