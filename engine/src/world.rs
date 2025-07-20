use crate::block::{Block, BlockType};
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use std::collections::{HashMap, VecDeque};

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
            return None;
        }
        self.get_chunk(chunk_x, chunk_z)
            .and_then(|chunk| chunk.get_block(local_x, local_y, local_z))
    }

    fn get_light_level(&self, pos: glam::IVec3) -> u8 {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return 0;
        }
        self.get_block_at_world(pos.x as f32, pos.y as f32, pos.z as f32)
            .map_or(0, |b| b.sky_light)
    }

    fn set_light_level(&mut self, pos: glam::IVec3, level: u8) {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return;
        }
        let ((chunk_x, chunk_z), (lx, ly, lz)) =
            World::world_to_chunk_coords(pos.x as f32, pos.y as f32, pos.z as f32);
        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
            chunk.set_block_light(lx, ly, lz, level);
        }
    }

    fn is_block_transparent(&self, pos: glam::IVec3) -> bool {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return true; // Outside world is transparent
        }
        self.get_block_at_world(pos.x as f32, pos.y as f32, pos.z as f32)
            .map_or(true, |b| b.is_transparent())
    }

    fn propagate_light_addition(&mut self, new_air_block_pos: glam::IVec3) {
        let mut max_light_from_neighbors: u8 = 0;
        let mut light_propagation_queue = VecDeque::new();

        for offset in [
            glam::IVec3::X,
            glam::IVec3::NEG_X,
            glam::IVec3::Y,
            glam::IVec3::NEG_Y,
            glam::IVec3::Z,
            glam::IVec3::NEG_Z,
        ] {
            let neighbor_pos = new_air_block_pos + offset;
            let neighbor_light = self.get_light_level(neighbor_pos);

            if neighbor_light == 0 {
                continue;
            }

            // Check if light is coming from directly above with full strength
            let potential_light = if offset == glam::IVec3::Y && neighbor_light == 15 {
                15
            } else {
                neighbor_light - 1
            };

            if potential_light > max_light_from_neighbors {
                max_light_from_neighbors = potential_light;
            }
        }

        self.set_light_level(new_air_block_pos, max_light_from_neighbors);

        if max_light_from_neighbors > 0 {
            light_propagation_queue.push_back(new_air_block_pos);
        }

        self.run_light_propagation_queue(light_propagation_queue);
    }

    fn propagate_light_removal(&mut self, new_solid_block_pos: glam::IVec3) {
        let light_level_removed = self.get_light_level(new_solid_block_pos);
        if light_level_removed == 0 {
            return;
        }

        self.set_light_level(new_solid_block_pos, 0);

        let mut removal_queue = VecDeque::new();
        let mut relight_queue = VecDeque::new();
        removal_queue.push_back((new_solid_block_pos, light_level_removed));

        while let Some((pos, light_level)) = removal_queue.pop_front() {
            for offset in [
                glam::IVec3::X,
                glam::IVec3::NEG_X,
                glam::IVec3::Y,
                glam::IVec3::NEG_Y,
                glam::IVec3::Z,
                glam::IVec3::NEG_Z,
            ] {
                let neighbor_pos = pos + offset;
                let neighbor_light = self.get_light_level(neighbor_pos);

                if neighbor_light == 0 {
                    continue;
                }

                if neighbor_light < light_level {
                    self.set_light_level(neighbor_pos, 0);
                    removal_queue.push_back((neighbor_pos, neighbor_light));
                } else {
                    relight_queue.push_back(neighbor_pos);
                }
            }
        }

        self.run_light_propagation_queue(relight_queue);
    }

    // Helper function to run the propagation loop, used by both addition and removal logic
    fn run_light_propagation_queue(&mut self, mut queue: VecDeque<glam::IVec3>) {
        while let Some(pos) = queue.pop_front() {
            let current_light = self.get_light_level(pos);
            let neighbor_light = current_light - 1;

            if neighbor_light <= 0 {
                continue;
            }

            for offset in [
                glam::IVec3::X,
                glam::IVec3::NEG_X,
                glam::IVec3::Y,
                glam::IVec3::NEG_Y,
                glam::IVec3::Z,
                glam::IVec3::NEG_Z,
            ] {
                let neighbor_pos = pos + offset;

                let potential_light = if offset == glam::IVec3::NEG_Y && current_light == 15 {
                    15
                } else {
                    neighbor_light
                };

                let neighbor_current_light = self.get_light_level(neighbor_pos);

                if self.is_block_transparent(neighbor_pos)
                    && potential_light > neighbor_current_light
                {
                    self.set_light_level(neighbor_pos, potential_light);
                    queue.push_back(neighbor_pos);
                }
            }
        }
    }

    pub fn set_block(
        &mut self,
        world_block_pos: glam::IVec3,
        block_type: BlockType,
    ) -> Result<(i32, i32), &'static str> {
        if world_block_pos.y < 0 || world_block_pos.y >= CHUNK_HEIGHT as i32 {
            return Err("Y coordinate out of world bounds");
        }

        let ((chunk_x, chunk_z), (local_x, local_y, local_z)) = World::world_to_chunk_coords(
            world_block_pos.x as f32,
            world_block_pos.y as f32,
            world_block_pos.z as f32,
        );

        let old_block_was_transparent = self.is_block_transparent(world_block_pos);
        if old_block_was_transparent && self.get_light_level(world_block_pos) == 0 {
            // Optimization: if we are placing a block in a dark spot, no light update needed
        }

        let chunk = self.get_or_create_chunk(chunk_x, chunk_z);
        if chunk
            .get_block(local_x, local_y, local_z)
            .map_or(false, |b| b.block_type == BlockType::Bedrock)
        {
            return Err("Cannot replace Bedrock");
        }

        chunk
            .set_block(local_x, local_y, local_z, block_type)
            .unwrap();

        let new_block_is_transparent = Block::new(block_type).is_transparent();

        if !old_block_was_transparent && new_block_is_transparent {
            // Solid -> Air: Add light
            self.propagate_light_addition(world_block_pos);
        } else if old_block_was_transparent && !new_block_is_transparent {
            // Air -> Solid: Remove light
            self.propagate_light_removal(world_block_pos);
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
        world
            .set_block(block_underneath_pos, BlockType::Air)
            .expect("Setting block should succeed.");

        // 4. ASSERT FINAL STATE: Check the light level of the newly exposed block.
        let air_block = world
            .get_block_at_world(
                block_underneath_pos.x as f32,
                block_underneath_pos.y as f32,
                block_underneath_pos.z as f32,
            )
            .expect("Dirt block should exist.");

        // This is the crucial test. It will fail until our logic is correct.
        assert_eq!(
            air_block.sky_light, 15,
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

        //
        let under_surface_air = world
            .get_block_at_world(
                under_surface_pos.x as f32,
                under_surface_pos.y as f32,
                under_surface_pos.z as f32,
            )
            .expect("Block should exist.");
        assert_eq!(
            under_surface_air.sky_light, 15,
            "Tunnel block should initially be dark."
        );
        //

        world
            .set_block(tunnel_end_pos, BlockType::Air)
            .expect("Digging block should succeed.");

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
