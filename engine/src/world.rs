use crate::block::{Block, BlockType};
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use std::collections::{HashMap, VecDeque}; // Removed BlockType as it's unused

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
        self.chunks.entry((chunk_x, chunk_z)).or_insert_with(|| {
            let mut new_chunk = Chunk::new(chunk_x, chunk_z);
            new_chunk.generate_terrain(); // Or some other generation logic
            new_chunk.calculate_sky_light();
            new_chunk
        })
    }

    // Gets an immutable reference to a chunk if it exists.
    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        self.chunks.get(&(chunk_x, chunk_z))
    }

    // Converts world block coordinates to chunk coordinates and local block coordinates.
    pub fn world_to_chunk_coords(
        world_x: f32,
        world_y: f32,
        world_z: f32,
    ) -> ((i32, i32), (usize, usize, usize)) {
        let chunk_x = (world_x / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_z = (world_z / CHUNK_DEPTH as f32).floor() as i32;

        let local_x = ((world_x % CHUNK_WIDTH as f32) + CHUNK_WIDTH as f32) % CHUNK_WIDTH as f32;
        let _local_y = world_y; // Assuming y is absolute for now, or chunks are full height slices. Prefixed as unused.
        let local_z = ((world_z % CHUNK_DEPTH as f32) + CHUNK_DEPTH as f32) % CHUNK_DEPTH as f32;

        // Clamping y to be within chunk height. This might need adjustment based on how
        // world_y interacts with chunk vertical slices if that becomes a feature.
        let clamped_y = world_y.max(0.0).min(CHUNK_HEIGHT as f32 - 1.0);

        (
            (chunk_x, chunk_z),
            (local_x as usize, clamped_y as usize, local_z as usize),
        )
    }

    // Gets a block at absolute world coordinates.
    // TODO this type should be not float?
    pub fn get_block_at_world(&self, world_x: f32, world_y: f32, world_z: f32) -> Option<&Block> {
        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) =
            World::world_to_chunk_coords(world_x, world_y, world_z);

        if local_y >= CHUNK_HEIGHT {
            // Check against actual CHUNK_HEIGHT
            return None; // y is out of any possible chunk's bounds
        }

        match self.get_chunk(chunk_x, chunk_z) {
            Some(chunk) => chunk.get_block(local_x, local_y, local_z),
            None => None, // Chunk doesn't exist
        }
    }

    /// Propagates the removal of light.
    fn propagate_light_removal(&mut self, start_pos: glam::IVec3, initial_light_level: u8) {
        let mut removal_queue: VecDeque<(glam::IVec3, u8)> = VecDeque::new();
        removal_queue.push_back((start_pos, initial_light_level));

        while let Some((pos, light_level)) = removal_queue.pop_front() {
            // Check all 6 neighbors
            let neighbors = [
                pos + glam::IVec3::X,
                pos - glam::IVec3::X,
                pos + glam::IVec3::Y,
                pos - glam::IVec3::Y,
                pos + glam::IVec3::Z,
                pos - glam::IVec3::Z,
            ];

            for neighbor_pos in neighbors {
                let ((chunk_x, chunk_z), (lx, ly, lz)) = World::world_to_chunk_coords(
                    neighbor_pos.x as f32,
                    neighbor_pos.y as f32,
                    neighbor_pos.z as f32,
                );

                if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
                    if let Some(neighbor_block) = chunk.get_block(lx, ly, lz).cloned() {
                        if neighbor_block.sky_light != 0 {
                            if neighbor_block.sky_light < light_level {
                                // This block was lit by a path that is now darker.
                                // Set its light to 0 and add it to the queue to propagate darkness.
                                chunk.set_block_light(lx, ly, lz, 0);
                                removal_queue.push_back((neighbor_pos, neighbor_block.sky_light));
                            } else {
                                // This block has a stronger light source from another path.
                                // We need to re-propagate its light. (We'll handle this in the next step)
                            }
                        }
                    }
                }
            }
        }
    }

    /// Propagates the addition of new light.
    fn propagate_light_addition(&mut self, mut queue: VecDeque<glam::IVec3>) {
        while let Some(pos) = queue.pop_front() {
            let ((chunk_x, chunk_z), (lx, ly, lz)) =
                World::world_to_chunk_coords(pos.x as f32, pos.y as f32, pos.z as f32);

            let current_light_level = self
                .chunks
                .get(&(chunk_x, chunk_z))
                .and_then(|c| c.get_block(lx, ly, lz))
                .map_or(0, |b| b.sky_light);

            let neighbor_light_level = current_light_level.saturating_sub(1);

            if neighbor_light_level == 0 {
                continue;
            }

            // Check all 6 neighbors
            let neighbors = [
                pos + glam::IVec3::X,
                pos - glam::IVec3::X,
                pos + glam::IVec3::Y,
                pos - glam::IVec3::Y,
                pos + glam::IVec3::Z,
                pos - glam::IVec3::Z,
            ];

            for neighbor_pos in neighbors {
                let ((n_chunk_x, n_chunk_z), (nx, ny, nz)) = World::world_to_chunk_coords(
                    neighbor_pos.x as f32,
                    neighbor_pos.y as f32,
                    neighbor_pos.z as f32,
                );

                if let Some(chunk) = self.chunks.get_mut(&(n_chunk_x, n_chunk_z)) {
                    if let Some(neighbor_block) = chunk.get_block(nx, ny, nz) {
                        if neighbor_block.is_transparent()
                            && neighbor_block.sky_light < neighbor_light_level
                        {
                            chunk.set_block_light(nx, ny, nz, neighbor_light_level);
                            queue.push_back(neighbor_pos);
                        }
                    }
                }
            }
        }
    }

    // Note: A `set_block_at_world` method would be similar but would need mutable access
    // and potentially create the chunk if it doesn't exist.
    // pub fn set_block_at_world(&mut self, world_x: f32, world_y: f32, world_z: f32, block_type: BlockType) -> Result<(), &'static str> { // Old signature
    // New set_block method using IVec3 and returning modified chunk coordinates
    // Returns the world chunk coordinates (cx, cz) of the modified chunk if successful.
    pub fn set_block(
        &mut self,
        world_block_pos: glam::IVec3,
        block_type: crate::block::BlockType,
    ) -> Result<(i32, i32), &'static str> {
        if world_block_pos.y < 0 || world_block_pos.y >= CHUNK_HEIGHT as i32 {
            return Err("Y coordinate out of world bounds");
        }

        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) = World::world_to_chunk_coords(
            world_block_pos.x as f32,
            world_block_pos.y as f32,
            world_block_pos.z as f32,
        );

        // --- ADD THIS LIGHTING LOGIC ---
        // Get the old light level BEFORE we change the block
        let old_light_level = self
            .get_chunk(chunk_x, chunk_z)
            .and_then(|c| c.get_block(local_x, local_y, local_z))
            .map_or(0, |b| b.sky_light);

        // If we are removing a block (placing Air) that had light, we need to update lighting.
        if block_type == BlockType::Air && old_light_level > 0 {
            // First, remove the old light
            self.propagate_light_removal(world_block_pos, old_light_level);

            // Now, create a new queue for adding light back in, starting with the neighbors
            let mut addition_queue: VecDeque<glam::IVec3> = VecDeque::new();
            let neighbors = [
                world_block_pos + glam::IVec3::X,
                world_block_pos - glam::IVec3::X,
                world_block_pos + glam::IVec3::Y,
                world_block_pos - glam::IVec3::Y,
                world_block_pos + glam::IVec3::Z,
                world_block_pos - glam::IVec3::Z,
            ];
            for neighbor_pos in neighbors {
                // We just add all neighbors that might be light sources to the queue
                addition_queue.push_back(neighbor_pos);
            }
            // A special case: if we opened a hole to the sky, the new Air block is now a source
            if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
                if chunk
                    .get_block(local_x, local_y + 1, local_z)
                    .map_or(false, |b| b.sky_light == 15)
                {
                    chunk.set_block_light(local_x, local_y, local_z, 15);
                    addition_queue.push_back(world_block_pos);
                }
            }

            self.propagate_light_addition(addition_queue);
        }
        // --- END OF NEW LOGIC ---

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
            Ok(_) => Ok((chunk_x, chunk_z)),
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
