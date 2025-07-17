use crate::block::{Block, BlockType};
use rand::Rng; // Assuming block.rs is in the same directory
use std::collections::VecDeque;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 32;
pub const CHUNK_DEPTH: usize = 16;

pub struct Chunk {
    pub coord: (i32, i32),
    blocks: Vec<Vec<Vec<Block>>>,
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

    pub fn generate_terrain(&mut self) {
        let surface_level = CHUNK_HEIGHT / 2;

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                self.blocks[x][0][z] = Block::new(BlockType::Bedrock);
                for y in 1..CHUNK_HEIGHT {
                    if y < surface_level {
                        self.blocks[x][y][z] = Block::new(BlockType::Dirt);
                    } else if y == surface_level {
                        self.blocks[x][y][z] = Block::new(BlockType::Grass);
                    }
                }
            }
        }

        let mut rng = rand::rng(); // Per compiler hint
        const TREE_CHANCE: f64 = 0.02;
        let mut next_tree_id: u32 = 1;

        for x_coord in 2..(CHUNK_WIDTH - 2) {
            for z_coord in 2..(CHUNK_DEPTH - 2) {
                if self.blocks[x_coord][surface_level][z_coord].block_type == BlockType::Grass {
                    if rng.random_bool(TREE_CHANCE) {
                        // Per compiler hint
                        self.place_tree(
                            x_coord,
                            surface_level + 1,
                            z_coord,
                            next_tree_id,
                            &mut rng,
                        );
                        next_tree_id = next_tree_id.wrapping_add(1);
                        if next_tree_id == 0 {
                            next_tree_id = 1;
                        }
                    }
                }
            }
        }
    }

    fn place_tree(&mut self, x: usize, y_base: usize, z: usize, tree_id: u32, rng: &mut impl Rng) {
        let trunk_height: usize = rng.random_range(3..=5); // Per compiler hint
        let canopy_radius: isize = 2;
        let canopy_base_y_offset: usize = trunk_height.saturating_sub(2);
        let canopy_max_height_above_base: usize = 3;

        let tree_top_y = y_base + trunk_height;
        if tree_top_y + canopy_max_height_above_base >= CHUNK_HEIGHT {
            return;
        }

        for i in 0..trunk_height {
            if y_base + i < CHUNK_HEIGHT {
                self.set_block(x, y_base + i, z, BlockType::OakLog)
                    .unwrap_or_default();
            }
        }

        let canopy_center_y_world = y_base + trunk_height - 1;
        let y_start_canopy = (y_base + canopy_base_y_offset).max(0);
        let y_end_canopy = (y_base + trunk_height + 1).min(CHUNK_HEIGHT - 1);

        for ly_world in y_start_canopy..=y_end_canopy {
            let y_dist_from_canopy_center =
                (ly_world as isize - canopy_center_y_world as isize).abs();
            let current_layer_radius = if y_dist_from_canopy_center <= 0 {
                canopy_radius
            } else if y_dist_from_canopy_center == 1 {
                canopy_radius
            } else {
                (canopy_radius - 1).max(1)
            };

            for lx_offset in -current_layer_radius..=current_layer_radius {
                for lz_offset in -current_layer_radius..=current_layer_radius {
                    let current_x_world = x as isize + lx_offset;
                    let current_z_world = z as isize + lz_offset;

                    if current_x_world < 0
                        || current_x_world >= CHUNK_WIDTH as isize
                        || current_z_world < 0
                        || current_z_world >= CHUNK_DEPTH as isize
                    {
                        continue;
                    }

                    let (ux, uy, uz) =
                        (current_x_world as usize, ly_world, current_z_world as usize);

                    if lx_offset == 0 && lz_offset == 0 && uy < canopy_center_y_world {
                        continue;
                    }

                    let dist_sq_horiz = lx_offset * lx_offset + lz_offset * lz_offset; // Defined here

                    if dist_sq_horiz > current_layer_radius * current_layer_radius {
                        if y_dist_from_canopy_center > 0 {
                            continue;
                        } else if dist_sq_horiz > (canopy_radius + 1) * (canopy_radius + 1) {
                            continue;
                        }
                    }

                    let probability = if current_layer_radius == 0 {
                        0.0
                    } else {
                        // Ensure radius - 1 does not underflow for isize if current_layer_radius is 0,
                        // though current_layer_radius logic already ensures it's at least 1 if not 0.
                        let r_core_sq = ((current_layer_radius - 1).max(0) as f32).powi(2);
                        let r_edge_sq = (current_layer_radius as f32).powi(2);
                        // let r_fuzzy_sq = ((current_layer_radius + 1) as f32).powi(2); // Optional outer fuzzy layer

                        if (dist_sq_horiz as f32) <= r_core_sq {
                            0.95 // Core part: almost always place
                        } else if (dist_sq_horiz as f32) <= r_edge_sq {
                            0.6 // Edge part: moderate chance
                        // } else if (dist_sq_horiz as f32) <= r_fuzzy_sq { // Uncomment for fuzzier edges
                        //     0.25 // Fuzzy outer part: lower chance
                        } else {
                            0.0 // Outside defined canopy for this layer
                        }
                    };

                    if probability > 0.0 && rng.random_bool(probability) {
                        if self.blocks[ux][uy][uz].block_type == BlockType::Air {
                            self.set_block_with_tree_id(ux, uy, uz, BlockType::OakLeaves, tree_id)
                                .unwrap_or_default();
                        }
                    }
                }
            }
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            Some(&self.blocks[x][y][z])
        } else {
            None
        }
    }

    pub fn set_block(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        block_type: BlockType,
    ) -> Result<(), &'static str> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            self.blocks[x][y][z] = Block::new(block_type);
            Ok(())
        } else {
            Err("Coordinates out of chunk bounds")
        }
    }

    pub fn set_block_with_tree_id(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        block_type: BlockType,
        tree_id: u32,
    ) -> Result<(), &'static str> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            self.blocks[x][y][z] = Block::new_with_tree_id(block_type, tree_id);
            Ok(())
        } else {
            Err("Coordinates out of chunk bounds")
        }
    }

    pub fn calculate_sky_light(&mut self) {
        // --- Phase 0: Reset ---
        // Start by setting all sky light levels to 0. This ensures we have a clean slate
        // and that blocks under overhangs aren't prematurely marked as permanently dark.
        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_DEPTH {
                    self.blocks[x][y][z].sky_light = 0;
                }
            }
        }

        let mut light_queue: VecDeque<(usize, usize, usize)> = VecDeque::new();

        // --- Phase 1: Vertical Sky Light Pass & Optimized Queue Seeding ---
        // Find all blocks with direct sky access, set their light to 15,
        // and add the transparent ones to the queue to act as light sources.
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                for y in (0..CHUNK_HEIGHT).rev() {
                    let block = &mut self.blocks[x][y][z];

                    // A block with sky access always has a light value of 15.
                    block.sky_light = 15;

                    // Only transparent blocks can propagate this light to neighbors.
                    if block.is_transparent() {
                        light_queue.push_back((x, y, z));
                    } else {
                        // If we hit a solid block, it receives light but stops it from
                        // continuing down this column.
                        break; // Move to the next column.
                    }
                }
            }
        }

        // --- Phase 2: Propagation Flood Fill ---
        // Spread light from the source blocks in the queue into adjacent dark areas.
        while let Some((x, y, z)) = light_queue.pop_front() {
            let current_light_level = self.blocks[x][y][z].sky_light;
            let neighbor_light_level = current_light_level.saturating_sub(1);

            if neighbor_light_level <= 0 {
                continue; // This light has faded to nothing.
            }

            // Check all 6 neighbors
            let neighbors = [
                (x.wrapping_sub(1), y, z),
                (x + 1, y, z),
                (x, y.wrapping_sub(1), z),
                (x, y + 1, z),
                (x, y, z.wrapping_sub(1)),
                (x, y, z + 1),
            ];

            // TODO handle blocks from neighboring chunks

            for (nx, ny, nz) in neighbors {
                // Check if the neighbor is within the chunk's bounds
                if nx < CHUNK_WIDTH && ny < CHUNK_HEIGHT && nz < CHUNK_DEPTH {
                    let neighbor_block = &mut self.blocks[nx][ny][nz];

                    // If the neighbor is transparent and we can make it brighter, update it.
                    if neighbor_block.is_transparent()
                        && neighbor_block.sky_light < neighbor_light_level
                    {
                        neighbor_block.sky_light = neighbor_light_level;
                        light_queue.push_back((nx, ny, nz));
                    }
                }
                // Note: Cross-chunk propagation would be handled here by querying the `World`.
            }
        }
    }

    pub fn set_block_light(&mut self, x: usize, y: usize, z: usize, sky_light: u8) {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            self.blocks[x][y][z].sky_light = sky_light;
        }
    }
}
