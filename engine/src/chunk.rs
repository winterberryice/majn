use crate::block::{Block, BlockType};
use rand::Rng;
use std::collections::VecDeque;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 32; // Reduced height for now
pub const CHUNK_DEPTH: usize = 16;

const MAX_LIGHT: u8 = 15;

pub struct Chunk {
    pub coord: (i32, i32),
    blocks: Vec<Vec<Vec<Block>>>, // Stored as [x][y][z]
    // TODO: Consider if dirty flags for meshing/lighting are needed here
}

impl Chunk {
    pub fn new(coord_x: i32, coord_z: i32) -> Self {
        let blocks =
            vec![vec![vec![Block::new(BlockType::Air); CHUNK_DEPTH]; CHUNK_HEIGHT]; CHUNK_WIDTH];
        Chunk {
            coord: (coord_x, coord_z),
            blocks,
        }
    }

    // Initializes block light based on emission property
    pub fn initialize_block_light(&mut self) {
        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_DEPTH {
                    let emission = self.blocks[x][y][z].emission();
                    if emission > 0 {
                        self.blocks[x][y][z].block_light_level = emission;
                    }
                }
            }
        }
    }

    // Initial pass for skylight - straight down
    pub fn calculate_initial_skylight(&mut self) {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let mut current_sky_light = MAX_LIGHT;
                for y in (0..CHUNK_HEIGHT).rev() { // Iterate from top to bottom
                    let block = &mut self.blocks[x][y][z];
                    if block.opacity() >= MAX_LIGHT { // Opaque block
                        current_sky_light = 0; // Sky light is blocked
                    }
                    block.sky_light_level = current_sky_light;
                    if current_sky_light > 0 && block.opacity() > 0 && block.opacity() < MAX_LIGHT {
                        // If light passes through a semi-transparent block, it might dim.
                        // For simplicity now, direct skylight isn't reduced by semi-transparent blocks,
                        // only by fully opaque ones. Spreading skylight will handle reduction.
                    }
                }
            }
        }
    }


    pub fn generate_terrain(&mut self) {
        let surface_level = CHUNK_HEIGHT / 2; // Grass will be at this Y level

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                // Place bedrock at y = 0
                self.blocks[x][0][z] = Block::new(BlockType::Bedrock);

                for y in 1..CHUNK_HEIGHT {
                    // Start from y = 1 since y = 0 is bedrock
                    if y < surface_level {
                        self.blocks[x][y][z] = Block::new(BlockType::Dirt);
                    } else if y == surface_level {
                        self.blocks[x][y][z] = Block::new(BlockType::Grass);
                    } // Above surface_level remains Air (as initialized)
                }
            }
        }

        // --- TASK: Pass 2: Generate Trees ---
        // After the main terrain is set, we make a second pass to add features like trees.
        let mut rng = rand::rng(); // Create a random number generator
        const TREE_CHANCE: f64 = 0.02; // 2% chance to try and place a tree at any given spot

        for x in 2..(CHUNK_WIDTH - 2) {
            // Iterate with a margin to keep trees from chunk edges
            for z in 2..(CHUNK_DEPTH - 2) {
                // Check if the block at the surface level is grass
                if self.blocks[x][surface_level][z].block_type == BlockType::Grass {
                    // Use the random number generator to decide if a tree should grow here
                    if rng.random_bool(TREE_CHANCE) {
                        // The ground level for the tree trunk is one block above the grass.
                        self.place_tree(x, surface_level + 1, z);
                    }
                }
            }
        }
    }

    // TASK: Create a helper function to place a tree at a specific location.
    // This makes the generation logic cleaner.
    fn place_tree(&mut self, x: usize, y_base: usize, z: usize) {
        // A simple tree structure.
        let trunk_height: usize = 4;
        let leaves_radius: usize = 2;

        // Don't place trees if they would grow out of the top of the chunk
        if y_base + trunk_height + leaves_radius >= CHUNK_HEIGHT {
            return;
        }

        // Place the trunk (Log blocks)
        for i in 0..trunk_height {
            self.set_block(x, y_base + i, z, BlockType::OakLog)
                .unwrap_or_default();
        }

        // Place the leaves canopy (a simple cube for now)
        let leaves_y_base = y_base + trunk_height - 1;
        for ly in 0..=leaves_radius {
            for lx in -(leaves_radius as isize)..=leaves_radius as isize {
                for lz in -(leaves_radius as isize)..=leaves_radius as isize {
                    // Simple square canopy shape
                    if lx == 0 && lz == 0 && ly < leaves_radius {
                        // Don't place leaves inside the top of the trunk
                        continue;
                    }
                    let block_x = x as isize + lx;
                    let block_y = leaves_y_base + ly;
                    let block_z = z as isize + lz;

                    // Check bounds before placing leaves
                    if block_x < CHUNK_WIDTH as isize
                        && block_y < CHUNK_HEIGHT
                        && block_z < CHUNK_DEPTH as isize
                    {
                        // Only place leaves if the spot is currently empty (Air)
                        if self.blocks[block_x as usize][block_y][block_z as usize].block_type
                            == BlockType::Air
                        {
                            self.set_block(
                                block_x as usize,
                                block_y,
                                block_z as usize,
                                BlockType::OakLeaves,
                            )
                            .unwrap_or_default();
                        }
                    }
                }
            }
        }
    }

    // Helper to get a block at a given coordinate
    // Returns Option<&Block> because coordinates might be out of bounds
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            Some(&self.blocks[x][y][z])
        } else {
            None
        }
    }

    // Mutable version of get_block
    pub fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Block> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            Some(&mut self.blocks[x][y][z])
        } else {
            None
        }
    }


    // Helper to set a block at a given coordinate
    // Returns Result<(), &str> to indicate success or out-of-bounds error
    // This function should also trigger light updates.
    pub fn set_block(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        block_type: BlockType,
    ) -> Result<(), &'static str> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            // Get old properties for light update
            // let old_block = self.blocks[x][y][z]; // This borrow would conflict
            // let old_opacity = old_block.opacity();
            // let old_emission = old_block.emission();

            self.blocks[x][y][z] = Block::new(block_type);
            let new_emission = self.blocks[x][y][z].emission(); // Re-borrow after modification

            // If new block is an emitter, set its base block_light_level.
            // Actual spreading will be handled by propagate_light.
            // If it's not an emitter, its block_light_level is already 0 from Block::new.
            if new_emission > 0 {
                self.blocks[x][y][z].block_light_level = new_emission;
            }

            // Skylight and further block light propagation are handled by World calling
            // calculate_initial_skylight and propagate_light after block changes.
            // For now, this function focuses on correctly initializing the new block's own light.

            // TODO: The World will need to manage light update queues (add/remove)
            // based on the difference between old and new block properties.

            Ok(())
        } else {
            Err("Coordinates out of chunk bounds")
        }
    }

    // Recalculates skylight for the entire chunk based on a new global skylight level.
    // Adds affected blocks to world-level removal and propagation queues.
    pub fn recalculate_skylight_based_on_global(
        &mut self,
        new_global_skylight_level: u8,
        sky_light_removal_queue: &mut VecDeque<super::world::LightNode>,
        sky_light_propagation_queue: &mut VecDeque<super::world::LightNode>,
        chunk_world_x_offset: i32,
        chunk_world_z_offset: i32,
    ) {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let mut current_max_sky = new_global_skylight_level;
                for y in (0..CHUNK_HEIGHT).rev() { // Iterate from top to bottom
                    let block = &mut self.blocks[x][y][z];
                    let world_pos = glam::ivec3(
                        chunk_world_x_offset + x as i32,
                        y as i32,
                        chunk_world_z_offset + z as i32,
                    );

                    let old_sky_light = block.sky_light_level;

                    if block.opacity() >= MAX_LIGHT_LEVEL { // Opaque block
                        current_max_sky = 0; // Sky light is blocked below this
                    }

                    let new_sky_light_for_block = current_max_sky;

                    if old_sky_light != new_sky_light_for_block {
                        // When adding to removal queue, light_level is the *old* light value
                        if old_sky_light > 0 {
                            sky_light_removal_queue.push_back(super::world::LightNode {
                                pos: world_pos,
                                light_level: old_sky_light,
                            });
                        }
                        block.sky_light_level = new_sky_light_for_block;
                        // When adding to propagation queue, light_level is the *new* light value
                        if new_sky_light_for_block > 0 {
                            sky_light_propagation_queue.push_back(super::world::LightNode {
                                pos: world_pos,
                                light_level: new_sky_light_for_block,
                            });
                        }
                    }
                }
            }
        }
    }


    // Propagates light within the chunk.
    // This version is for intra-chunk propagation. Cross-chunk will be handled by World.
    pub fn propagate_light_in_chunk(&mut self) {
        let mut sky_queue: VecDeque<(usize, usize, usize, u8)> = VecDeque::new();
        let mut block_queue: VecDeque<(usize, usize, usize, u8)> = VecDeque::new();

        // Initialize queues: add all blocks that currently have some light.
        // The light value in the queue is the current light level of that block.
        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_DEPTH {
                    let block = &self.blocks[x][y][z];
                    if block.sky_light_level > 0 {
                        sky_queue.push_back((x, y, z, block.sky_light_level));
                    }
                    if block.block_light_level > 0 {
                        block_queue.push_back((x, y, z, block.block_light_level));
                    }
                }
            }
        }

        // Skylight propagation (BFS)
        while let Some((x, y, z, light_val)) = sky_queue.pop_front() {
            // If the light_val from queue is weaker than current block's actual skylight,
            // it means a stronger path has already updated this block. Skip.
            if light_val < self.blocks[x][y][z].sky_light_level && light_val != MAX_LIGHT { // MAX_LIGHT check for initial sources from top
                 // Exception: if it's an initial MAX_LIGHT source from calculate_initial_skylight,
                 // it should still propagate even if its current value is somehow less (should not happen).
                 // More robustly: only propagate if light_val is the *source* of current block's light.
                 // For now, this check is a simple optimization.
            }
            // Re-fetch current_light, as it might have been updated by another path if not using the above optimization.
            let current_light = self.blocks[x][y][z].sky_light_level;
            if current_light == 0 { continue; }


            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if (dx.abs() + dy.abs() + dz.abs()) != 1 { continue; } // Cardinal directions only

                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        let nz = z as i32 + dz;

                        if nx < 0 || nx >= CHUNK_WIDTH as i32 ||
                           ny < 0 || ny >= CHUNK_HEIGHT as i32 ||
                           nz < 0 || nz >= CHUNK_DEPTH as i32 {
                            // Out of chunk bounds, world will handle this boundary.
                            continue;
                        }

                        let neighbor_x = nx as usize;
                        let neighbor_y = ny as usize;
                        let neighbor_z = nz as usize;

                        let neighbor_block = &mut self.blocks[neighbor_x][neighbor_y][neighbor_z];
                        let opacity = neighbor_block.opacity();

                        // Skylight does not lose strength passing through fully transparent blocks (opacity 0)
                        // but does lose 1 level for each step into a block with opacity > 0, plus that opacity.
                        // However, for skylight spread, it typically just loses 1 level per block unless it's very opaque.
                        // Minecraft logic: skylight loses 1 per step, unless it's downwards and current block is transparent, then it's same.
                        // For horizontal/upwards spread, it's current_light - 1 - opacity.
                        // Let's simplify: skylight passing into any block (even air) loses 1 level.
                        // If the block itself is opaque, it blocks more.
                        let mut reduction = 1;
                        if opacity >= MAX_LIGHT { // Fully opaque
                            reduction = MAX_LIGHT + 1; // Effectively blocks all light
                        } else if opacity > 0 {
                            reduction += opacity; // Reduce by opacity for semi-transparent
                        }

                        // Special handling for skylight going down into a transparent block from above.
                        // If dy == -1 (going down) and neighbor_block.opacity() == 0, new_light should be current_light.
                        // This is mostly handled by calculate_initial_skylight.
                        // Here, for spreading, we assume general case.
                        if dy == -1 && neighbor_block.opacity() == 0 && current_light == MAX_LIGHT {
                             // If source is MAX_LIGHT (direct sky) and moving down into fully transparent block
                            reduction = 0;
                        }


                        let new_light = current_light.saturating_sub(reduction);

                        if new_light > neighbor_block.sky_light_level {
                            neighbor_block.sky_light_level = new_light;
                            sky_queue.push_back((neighbor_x, neighbor_y, neighbor_z, new_light));
                        }
                    }
                }
            }
        }

        // Block light propagation (BFS)
        while let Some((x, y, z, light_val)) = block_queue.pop_front() {
            // Similar optimization check as for skylight
            if light_val < self.blocks[x][y][z].block_light_level {
                // continue;
            }
            let current_light = self.blocks[x][y][z].block_light_level;
            if current_light == 0 { continue; }


            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if (dx.abs() + dy.abs() + dz.abs()) != 1 { continue; } // Cardinal only

                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        let nz = z as i32 + dz;

                        if nx < 0 || nx >= CHUNK_WIDTH as i32 ||
                           ny < 0 || ny >= CHUNK_HEIGHT as i32 ||
                           nz < 0 || nz >= CHUNK_DEPTH as i32 {
                            continue;
                        }

                        let neighbor_x = nx as usize;
                        let neighbor_y = ny as usize;
                        let neighbor_z = nz as usize;

                        let neighbor_block = &mut self.blocks[neighbor_x][neighbor_y][neighbor_z];
                        let opacity = neighbor_block.opacity();

                        // Block light always attenuates by at least 1, plus opacity of the medium it enters.
                        let attenuation = 1u8.saturating_add(opacity);
                        let new_light = current_light.saturating_sub(attenuation);

                        if new_light > neighbor_block.block_light_level {
                            neighbor_block.block_light_level = new_light;
                            block_queue.push_back((neighbor_x, neighbor_y, neighbor_z, new_light));
                        }
                    }
                }
            }
        }
    }
}
