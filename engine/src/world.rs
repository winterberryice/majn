use crate::block::Block;
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

        let mut re_light_queue: VecDeque<glam::IVec3> = VecDeque::new();

        while let Some((pos, light_level)) = removal_queue.pop_front() {
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

                let neighbor_light = if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_z)) {
                    chunk.get_block(lx, ly, lz).map(|b| b.sky_light)
                } else {
                    None
                };

                if let Some(n_light) = neighbor_light {
                    if n_light != 0 {
                        if n_light < light_level {
                            if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
                                chunk.set_block_light(lx, ly, lz, 0);
                            }
                            removal_queue.push_back((neighbor_pos, n_light));
                        } else {
                            re_light_queue.push_back(neighbor_pos);
                        }
                    }
                }
            }
        }

        self.propagate_light_addition(re_light_queue);
    }

    fn propagate_light_addition(&mut self, mut queue: VecDeque<glam::IVec3>) {
        while let Some(pos) = queue.pop_front() {
            let ((chunk_x, chunk_z), (lx, ly, lz)) =
                World::world_to_chunk_coords(pos.x as f32, pos.y as f32, pos.z as f32);

            let current_light_level = self
                .chunks
                .get(&(chunk_x, chunk_z))
                .and_then(|c| c.get_block(lx, ly, lz))
                .map_or(0, |b| b.sky_light);

            let neighbors = [
                (pos + glam::IVec3::X, false),
                (pos - glam::IVec3::X, false),
                (pos + glam::IVec3::Y, false),
                (pos - glam::IVec3::Y, true), // is_vertical_down
                (pos + glam::IVec3::Z, false),
                (pos - glam::IVec3::Z, false),
            ];

            for (neighbor_pos, is_vertical_down) in neighbors {
                if neighbor_pos.y < 0 || neighbor_pos.y >= CHUNK_HEIGHT as i32 {
                    continue;
                }

                // **THE CRITICAL CHANGE IS HERE**
                // Calculate the potential light level for the neighbor.
                let neighbor_light_level = if is_vertical_down && current_light_level == 15 {
                    15 // Sky light does NOT decay when going straight down.
                } else {
                    current_light_level.saturating_sub(1) // Light decays in all other cases.
                };

                // Read the neighbor's current properties before trying to change them.
                let ((n_chunk_x, n_chunk_z), (nx, ny, nz)) = World::world_to_chunk_coords(
                    neighbor_pos.x as f32,
                    neighbor_pos.y as f32,
                    neighbor_pos.z as f32,
                );

                let neighbor_properties =
                    if let Some(chunk) = self.chunks.get(&(n_chunk_x, n_chunk_z)) {
                        chunk
                            .get_block(nx, ny, nz)
                            .map(|b| (b.sky_light, b.is_transparent()))
                    } else {
                        None
                    };

                // If the potential new light is brighter, update the neighbor.
                if let Some((neighbor_sky_light, is_neighbor_transparent)) = neighbor_properties {
                    if neighbor_light_level > neighbor_sky_light {
                        if let Some(chunk) = self.chunks.get_mut(&(n_chunk_x, n_chunk_z)) {
                            chunk.set_block_light(nx, ny, nz, neighbor_light_level);
                            // Only continue the process from transparent blocks.
                            if is_neighbor_transparent {
                                queue.push_back(neighbor_pos);
                            }
                        }
                    }
                }
            }
        }
    }

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

        let old_block = self
            .get_block_at_world(
                world_block_pos.x as f32,
                world_block_pos.y as f32,
                world_block_pos.z as f32,
            )
            .cloned();

        if let Some(ref b) = old_block {
            if b.block_type == crate::block::BlockType::Bedrock {
                return Err("Cannot replace Bedrock");
            }
        }

        let old_light_level = old_block.as_ref().map_or(0, |b| b.sky_light);
        let was_transparent = old_block.as_ref().map_or(true, |b| b.is_transparent());

        // Place the new block first
        let new_block = Block::new(block_type);
        let is_transparent = new_block.is_transparent();
        self.get_or_create_chunk(chunk_x, chunk_z)
            .set_block(local_x, local_y, local_z, block_type)
            .unwrap();

        if is_transparent && !was_transparent {
            // A solid block was removed. We need to introduce light.
            // We'll create one queue and add all initial light sources to it.
            let mut light_addition_queue = VecDeque::new();

            // The new air block itself is a potential path for light.
            light_addition_queue.push_back(world_block_pos);

            // The block above the one we broke is now a source of downward light.
            let pos_above = world_block_pos + glam::IVec3::Y;
            if pos_above.y < CHUNK_HEIGHT as i32 {
                light_addition_queue.push_back(pos_above);
            }

            // Propagate light from all sources at once.
            self.propagate_light_addition(light_addition_queue);
        } else if !is_transparent && was_transparent {
            // A solid block was placed. We need to remove light.
            if old_light_level > 0 {
                self.get_or_create_chunk(chunk_x, chunk_z)
                    .set_block_light(local_x, local_y, local_z, 0);
                self.propagate_light_removal(world_block_pos, old_light_level);
            }
        }

        Ok((chunk_x, chunk_z))
    }
}

// Default implementation for World
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;
    use glam::IVec3;

    #[test]
    fn test_sky_light_update_on_dig() {
        // 1. SETUP: Create a world and generate a chunk.
        let mut world = World::new();
        // This generates a chunk at (0,0) with standard terrain and runs the initial
        // sky light calculation. The surface level will be at Y=16.
        world.get_or_create_chunk(0, 0);

        // Define the coordinates for our test blocks.
        let grass_block_pos = IVec3::new(5, 16, 5);
        let block_underneath_pos = IVec3::new(5, 15, 5);

        // 2. ASSERT INITIAL STATE: Verify the world is as we expect before the change.
        // Check the grass block on the surface.
        let grass_block = world
            .get_block_at_world(
                grass_block_pos.x as f32,
                grass_block_pos.y as f32,
                grass_block_pos.z as f32,
            )
            .expect("Grass block should exist.");
        assert_eq!(grass_block.block_type, BlockType::Grass);
        assert_eq!(
            grass_block.sky_light, 15,
            "Surface block should have full sky light."
        );

        // Check the block directly underneath it.
        let block_underneath = world
            .get_block_at_world(
                block_underneath_pos.x as f32,
                block_underneath_pos.y as f32,
                block_underneath_pos.z as f32,
            )
            .expect("Block underneath should exist.");
        assert_eq!(
            block_underneath.sky_light, 0,
            "Block underneath should initially be dark."
        );

        // 3. ACTION: Dig the grass block, replacing it with Air.
        world
            .set_block(grass_block_pos, BlockType::Air)
            .expect("Setting block should succeed.");

        // 4. ASSERT FINAL STATE: Check the light level of the newly exposed block.
        let newly_exposed_block = world
            .get_block_at_world(
                block_underneath_pos.x as f32,
                block_underneath_pos.y as f32,
                block_underneath_pos.z as f32,
            )
            .expect("Newly exposed block should exist.");

        // This is the crucial test. It will fail until our logic is correct.
        assert_eq!(
            newly_exposed_block.sky_light, 15,
            "Newly exposed block should now have full sky light."
        );
    }

    #[test]
    fn test_sky_light_spreads_horizontally_into_tunnel() {
        // 1. SETUP: Create a world and generate a chunk.
        let mut world = World::new();
        world.get_or_create_chunk(0, 0);

        // Define the coordinates for the tunnel we will dig.
        let surface_block_pos = IVec3::new(8, 16, 8); // The grass block on top.
        let under_surface_pos = IVec3::new(8, 15, 8); // The block directly below.
        let tunnel_end_pos = IVec3::new(8, 15, 9); // One block forward, this is where light should be 14.

        // 2. ASSERT INITIAL STATE: Verify the tunnel end is dark.
        let tunnel_block_initial = world
            .get_block_at_world(
                tunnel_end_pos.x as f32,
                tunnel_end_pos.y as f32,
                tunnel_end_pos.z as f32,
            )
            .expect("Block should exist.");
        assert_eq!(
            tunnel_block_initial.sky_light, 0,
            "Tunnel block should initially be dark."
        );

        // 3. ACTION: Dig the two blocks to create the tunnel entrance.
        world
            .set_block(surface_block_pos, BlockType::Air)
            .expect("Digging surface block should succeed.");
        world
            .set_block(under_surface_pos, BlockType::Air)
            .expect("Digging block underneath should succeed.");

        // 4. ASSERT FINAL STATE: Check the light level at the end of the one-block tunnel.
        let tunnel_block_final = world
            .get_block_at_world(
                tunnel_end_pos.x as f32,
                tunnel_end_pos.y as f32,
                tunnel_end_pos.z as f32,
            )
            .expect("Final tunnel block should exist.");

        // This assertion will fail until we correctly implement light propagation.
        assert_eq!(
            tunnel_block_final.sky_light, 14,
            "Light should spread one block into the tunnel, decreasing to 14."
        );
    }

    #[test]
    fn test_light_in_horizontal_tunnel_from_shaft() {
        // Simulates the pattern:
        // aaaa (y=17, sky)
        // gagg (y=16, surface)
        // dahd (y=15, player head level)
        // dmfd (y=14, player foot level)
        // dddd (y=13, floor)

        // 1. SETUP: Create a world and a custom chunk with a predictable surface at y=16.
        // 1. SETUP: Create a world and generate a chunk.
        let mut world = World::new();
        world.get_or_create_chunk(0, 0);

        // 2. ACTION: Dig out the specific pattern based on the corrected levels.
        // Place the 'f' block at a known coordinate, e.g., (5, 14, 6).
        // This means the vertical shaft is at z=5.
        let f_pos = IVec3::new(5, 14, 6); // Player foot level
        let m_pos = IVec3::new(5, 14, 5); // The air block in the shaft at the same level as 'f'.
        let h_pos = IVec3::new(5, 15, 6); // Player head level.

        // Dig the vertical shaft at z=5. This will trigger light propagation.
        world
            .set_block(IVec3::new(5, 16, 5), BlockType::Air)
            .unwrap(); // Top grass block
        world
            .set_block(IVec3::new(5, 15, 5), BlockType::Air)
            .unwrap(); // Air at head level in shaft
        world.set_block(m_pos, BlockType::Air).unwrap(); // Air at foot level in shaft ('m')

        // Dig the horizontal tunnel for the player
        world.set_block(h_pos, BlockType::Air).unwrap(); // 'h'
        world.set_block(f_pos, BlockType::Air).unwrap(); // 'f'

        // 3. ASSERT: Check the light levels.
        let block_at_m = world
            .get_block_at_world(m_pos.x as f32, m_pos.y as f32, m_pos.z as f32)
            .expect("Block 'm' should exist.");
        assert_eq!(
            block_at_m.sky_light, 15,
            "Block 'm' in the open shaft should have full sky light."
        );

        let block_at_f = world
            .get_block_at_world(f_pos.x as f32, f_pos.y as f32, f_pos.z as f32)
            .expect("Block 'f' should exist.");
        assert_eq!(
            block_at_f.sky_light, 14,
            "Block 'f' one block into the tunnel should have light level 14."
        );
    }

    //
}
