use crate::block::{Block, BlockType}; // BlockType needed for set_block
use crate::chunk::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::lighting; // Added for lighting initialization
use glam::IVec3; // Added for IVec3
use std::collections::HashMap;

pub struct World {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl World {
    pub fn new() -> Self {
        World {
            chunks: HashMap::new(),
        }
    }

    // Gets a mutable reference to a chunk if it exists.
    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(chunk_x, chunk_z))
    }

    // Gets a reference to a chunk if it exists, otherwise generates/loads it.
    pub fn get_or_create_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &mut Chunk {
        // Check if the chunk already exists and has lighting calculated (e.g. by a flag or by checking some light values)
        // For now, we assume if it exists, lighting is done. If it's newly inserted, calculate lighting.
        // This logic might need refinement if chunks can be generated without immediate lighting.

        let is_new_chunk = !self.chunks.contains_key(&(chunk_x, chunk_z));

        let chunk_entry = self.chunks.entry((chunk_x, chunk_z)).or_insert_with(|| {
            let mut new_chunk = Chunk::new(chunk_x, chunk_z);
            new_chunk.generate_terrain();
            new_chunk // Terrain generated, but lighting not yet done for this new_chunk
        });

        if is_new_chunk {
            // The chunk was just inserted and generated. Now calculate its lighting.
            // This requires `&mut World` (which `self` is), so we do it after `or_insert_with`.
            // We pass chunk_x, chunk_z because initialize_lighting_for_generated_chunk
            // will need to operate on this chunk and potentially its neighbors.
            // Note: `chunk_entry` is a mutable reference to the chunk within the HashMap.
            // `initialize_lighting_for_generated_chunk` takes `&mut World`.
            // This is a classic borrow checker problem: we have a mutable borrow of a part of `self.chunks`
            // via `chunk_entry`, and `initialize_lighting_for_generated_chunk` needs `&mut self` (the whole World).

            // To resolve this, we can't hold `chunk_entry` across the lighting call.
            // We can release the borrow from `or_insert_with` by letting it go out of scope,
            // then call lighting, then get the chunk reference again.
            // However, `or_insert_with` returns the mutable reference directly.

            // A common pattern:
            // 1. Generate terrain for the new chunk (done by `or_insert_with` if new).
            // 2. *After* the `or_insert_with` block (so the mutable borrow of `self.chunks` from `entry()` is released),
            //    call the lighting function which takes `&mut self`.
            // 3. Then, retrieve the chunk again to return it.

            // Let's restructure:
            // First, ensure the chunk is generated if it's new.
            if !self.chunks.contains_key(&(chunk_x, chunk_z)) {
                let mut new_chunk = Chunk::new(chunk_x, chunk_z);
                new_chunk.generate_terrain();
                self.chunks.insert((chunk_x, chunk_z), new_chunk);
                // Now the chunk is in the map, and we don't have a direct borrow from `entry()`.
                // We can call the lighting function.
                lighting::initialize_lighting_for_generated_chunk(self, chunk_x, chunk_z);
            }
            // This `self` in the lighting call refers to the entire World struct.
        }

        // Now that lighting is done (if it was a new chunk), get and return the chunk.
        self.chunks.get_mut(&(chunk_x, chunk_z)).unwrap() // Should always exist now
    }

    // Gets an immutable reference to a chunk if it exists.
    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        self.chunks.get(&(chunk_x, chunk_z))
    }

    // Converts world IVec3 block coordinates to chunk coordinates (i32, i32) and local IVec3 block coordinates.
    pub fn world_to_chunk_and_local_coords(world_pos: IVec3) -> ((i32, i32), IVec3) {
        let chunk_x = (world_pos.x as f32 / CHUNK_WIDTH as f32).floor() as i32;
        let chunk_z = (world_pos.z as f32 / CHUNK_DEPTH as f32).floor() as i32;

        let local_x = world_pos.x.rem_euclid(CHUNK_WIDTH as i32);
        let local_y = world_pos.y; // Y is absolute, no chunk transformation
        let local_z = world_pos.z.rem_euclid(CHUNK_DEPTH as i32);

        (
            (chunk_x, chunk_z),
            IVec3::new(local_x, local_y, local_z),
        )
    }

    // Gets a block at absolute world IVec3 coordinates.
    pub fn get_block_world_space(&self, world_pos: IVec3) -> Option<&Block> {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return None; // Y is out of any possible chunk's bounds
        }
        let ((chunk_x, chunk_z), local_pos) = Self::world_to_chunk_and_local_coords(world_pos);

        match self.get_chunk(chunk_x, chunk_z) {
            Some(chunk) => chunk.get_block(
                local_pos.x as usize,
                local_pos.y as usize,
                local_pos.z as usize,
            ),
            None => None, // Chunk doesn't exist
        }
    }

    // Gets a block at absolute world f32 coordinates. (Kept for compatibility if needed by existing code)
    pub fn get_block_at_world_f32(&self, world_x: f32, world_y: f32, world_z: f32) -> Option<&Block> {
        let world_block_pos = IVec3::new(world_x.floor() as i32, world_y.floor() as i32, world_z.floor() as i32);
        self.get_block_world_space(world_block_pos)
    }


    // Sets a block at absolute world IVec3 coordinates.
    // Returns the world chunk coordinates (cx, cz) of the modified chunk if successful.
    pub fn set_block_world_space(
        &mut self,
        world_pos: IVec3,
        block_type: BlockType,
    ) -> Result<(i32, i32), &'static str> {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return Err("Y coordinate out of world bounds");
        }

        let ((chunk_x, chunk_z), local_pos) = Self::world_to_chunk_and_local_coords(world_pos);

        // Check if the block being replaced is Bedrock
        if let Some(chunk) = self.get_chunk(chunk_x, chunk_z) {
            if let Some(existing_block) = chunk.get_block(
                local_pos.x as usize,
                local_pos.y as usize,
                local_pos.z as usize,
            ) {
                if existing_block.block_type == BlockType::Bedrock {
                    return Err("Cannot replace Bedrock");
                }
            }
        }

        let chunk = self.get_or_create_chunk(chunk_x, chunk_z);
        match chunk.set_block(
            local_pos.x as usize,
            local_pos.y as usize,
            local_pos.z as usize,
            block_type,
        ) {
            Ok(_) => Ok((chunk_x, chunk_z)),
            Err(e) => Err(e), // Propagate error from chunk.set_block
        }
    }

    // Light access functions using world coordinates
    pub fn get_sky_light_world_space(&self, world_pos: IVec3) -> Option<u8> {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return Some(0); // Outside vertical bounds, considered dark for sky light (or max if above world)
                            // For sky light, if y >= CHUNK_HEIGHT, it should be max light.
                            // Let's refine this:
                            // if world_pos.y >= CHUNK_HEIGHT as i32 { return Some(15); }
                            // if world_pos.y < 0 { return Some(0); }
                            // This simple check is better handled by caller context or within chunk if pos is invalid.
                            // For now, if it's out of bounds, it means no chunk data, so None.
        }
        let ((chunk_x, chunk_z), local_pos) = Self::world_to_chunk_and_local_coords(world_pos);
        self.get_chunk(chunk_x, chunk_z)
            .map(|chunk| chunk.get_sky_light(local_pos))
    }

    pub fn set_sky_light_world_space(&mut self, world_pos: IVec3, level: u8) {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return; // Cannot set light outside vertical bounds
        }
        let ((chunk_x, chunk_z), local_pos) = Self::world_to_chunk_and_local_coords(world_pos);
        if let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_z) {
            chunk.set_sky_light(local_pos, level);
        }
        // If chunk doesn't exist, we can't set light. Could be an error or silent fail.
        // For lighting propagation, we might need to load/create chunks.
        // For now, if chunk not loaded, light is not set.
    }

    pub fn get_block_light_world_space(&self, world_pos: IVec3) -> Option<u8> {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return Some(0); // Outside vertical bounds, considered dark
        }
        let ((chunk_x, chunk_z), local_pos) = Self::world_to_chunk_and_local_coords(world_pos);
        self.get_chunk(chunk_x, chunk_z)
            .map(|chunk| chunk.get_block_light(local_pos))
    }

    pub fn set_block_light_world_space(&mut self, world_pos: IVec3, level: u8) {
        if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
            return; // Cannot set light outside vertical bounds
        }
        let ((chunk_x, chunk_z), local_pos) = Self::world_to_chunk_and_local_coords(world_pos);
        if let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_z) {
            chunk.set_block_light(local_pos, level);
        }
    }
}

// Default implementation for World
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
