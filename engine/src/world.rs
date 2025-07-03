use std::collections::HashMap;
use crate::chunk::{Chunk, CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH};
use crate::block::{Block, BlockType}; // Added BlockType back
use crate::lighting; // Import the lighting module

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
        let is_new_chunk = !self.chunks.contains_key(&(chunk_x, chunk_z));
        let chunk = self.chunks
            .entry((chunk_x, chunk_z))
            .or_insert_with(|| {
                let mut new_chunk = Chunk::new(chunk_x, chunk_z);
                new_chunk.generate_terrain();
                new_chunk
            });

        if is_new_chunk {
            // This is tricky because propagate_sky_light needs &mut World,
            // but we are already borrowing self.chunks mutably via `chunk`.
            // We need to release the mutable borrow of `chunk` before calling lighting.
            // A common pattern is to do the insertion, then retrieve and light.
            // However, or_insert_with makes this a bit direct.
            // Let's recalculate sky light after the chunk is definitely in the map.
            // This will be handled by a separate call after get_or_create_chunk in main loop or similar.
            // For now, we'll defer the lighting call to where `get_or_create_chunk` is called from
            // or rethink how `propagate_sky_light` accesses world.
            //
            // Temporary solution: We can't call lighting::propagate_sky_light(self, ...) here directly
            // due to Rust's borrow checker (self is mutably borrowed by `chunk`).
            // The lighting for new chunks will be initiated from the main game loop's update phase for now.
        }
        chunk
    }

    // Added to allow mutable access to a chunk, needed by lighting system.
    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(chunk_x, chunk_z))
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
    pub fn set_block(&mut self, world_block_pos: glam::IVec3, new_block_type: crate::block::BlockType) -> Result<(i32, i32), &'static str> {
        if world_block_pos.y < 0 || world_block_pos.y >= CHUNK_HEIGHT as i32 {
            return Err("Y coordinate out of world bounds");
        }

        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) =
            World::world_to_chunk_coords(world_block_pos.x as f32, world_block_pos.y as f32, world_block_pos.z as f32);

        let local_pos_ivec = glam::IVec3::new(local_x as i32, local_y as i32, local_z as i32);

        // Ensure local_y is also valid after conversion
        if local_y >= CHUNK_HEIGHT {
            return Err("Calculated local Y coordinate out of chunk bounds");
        }

        let old_block_type_option: Option<BlockType> = self.get_chunk(chunk_x, chunk_z)
            .and_then(|c| c.get_block(local_x, local_y, local_z))
            .map(|b| b.block_type);

        let old_block_is_opaque = old_block_type_option.map_or(false, |bt| Block::new(bt).is_opaque());


        // Check for Bedrock
        if old_block_type_option == Some(BlockType::Bedrock) {
            return Err("Cannot replace Bedrock");
        }

        let old_block_emission = old_block_type_option.map_or(0, |bt| Block::new(bt).get_light_emission());

        // --- Start Lighting Updates ---
        // 1. Remove old block light if it was a source
        if old_block_emission > 0 {
            lighting::remove_light(self, (chunk_x, chunk_z), local_pos_ivec, false);
        }

        // --- Update the block itself ---
        let set_block_result = {
            let chunk = self.get_or_create_chunk(chunk_x, chunk_z); // get_or_create_chunk ensures chunk exists
            chunk.set_block(local_x, local_y, local_z, new_block_type)
        };

        if set_block_result.is_err() {
            return Err(set_block_result.err().unwrap()); // Propagate error
        }

        // --- Post-Block-Update Lighting ---
        let new_block = Block::new(new_block_type); // Create a temporary Block for properties
        let new_block_emission = new_block.get_light_emission();
        let new_block_is_opaque = new_block.is_opaque();

        // 2. Propagate new block light if it's a source
        if new_block_emission > 0 {
            lighting::propagate_block_light(self, (chunk_x, chunk_z), local_pos_ivec, new_block_emission);
        }

        // 3. Handle Sky Light Changes
        // Case A: Opaque block placed where it was transparent, or vice-versa (potentially affecting sky light pass-through)
        if old_block_is_opaque != new_block_is_opaque {
            // This is a significant change. Re-calculate sky light for this chunk.
            // This is a broad stroke but safer for complex sky light interactions.
            // A more optimized approach would be to trace downwards from the changed block.
            lighting::propagate_sky_light(self, chunk_x, chunk_z);
            // Also need to consider neighboring chunks if near edge, propagate_sky_light should handle some of this.
            // For now, just the current chunk.
        }
        // Case B: Specifically, if an opaque block was removed (became transparent/Air)
        else if old_block_is_opaque && !new_block_is_opaque {
             // This implies a potential new path for skylight.
             // Re-propagating for the whole chunk is a safe bet.
            lighting::propagate_sky_light(self, chunk_x, chunk_z);
        }


        // TODO: More nuanced sky light handling:
        // - If an opaque block is ADDED: remove sky light below it.
        //   Iterate y from local_y - 1 down to 0. If sky_light > 0, set to 0 and add to a sky_remove_queue.
        //   Then process sky_remove_queue similar to remove_light but only for sky, and queue affected neighbors for repropagation.
        // - If an opaque block is REMOVED (new_block_type is Air/transparent):
        //   The current call to propagate_sky_light for the whole chunk should handle this by re-evaluating from top.

        Ok((chunk_x, chunk_z))
    }
}

// Default implementation for World
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
