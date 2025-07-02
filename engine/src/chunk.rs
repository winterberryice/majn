use crate::block::{Block, BlockType, MAX_LIGHT_LEVEL};
use rand::Rng; // Import Rng trait for rng.gen_bool and rng.gen_range
use std::collections::VecDeque;
use super::world::LightNode; // Import LightNode

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

    pub fn calculate_initial_skylight(&mut self) {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let mut current_sky_light = MAX_LIGHT_LEVEL;
                for y in (0..CHUNK_HEIGHT).rev() {
                    let block = &mut self.blocks[x][y][z];
                    if block.opacity() >= MAX_LIGHT_LEVEL {
                        current_sky_light = 0;
                    }
                    block.sky_light_level = current_sky_light;
                }
            }
        }
    }

    pub fn generate_terrain(&mut self) {
        let surface_level = CHUNK_HEIGHT / 2;
        let mut rng = rand::thread_rng(); // Initialize rng

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                self.blocks[x][0][z] = Block::new(BlockType::Bedrock);

                for y in 1..CHUNK_HEIGHT {
                    if y < surface_level {
                        self.blocks[x][y][z] = Block::new(BlockType::Stone);
                    } else if y == surface_level {
                        self.blocks[x][y][z] = Block::new(BlockType::Grass);
                    }
                }
            }
        }

        const TREE_CHANCE: f64 = 0.02;

        for x in 2..(CHUNK_WIDTH - 2) {
            for z in 2..(CHUNK_DEPTH - 2) {
                if self.blocks[x][surface_level][z].block_type == BlockType::Grass {
                    if rng.gen_bool(TREE_CHANCE) {
                        self.place_tree(x, surface_level + 1, z, &mut rng); // Pass rng
                    }
                }
            }
        }
    }

    fn place_tree(&mut self, x: usize, y_base: usize, z: usize, rng: &mut impl Rng) { // Accept Rng
        let trunk_height: usize = rng.gen_range(3..=5);
        let leaves_radius: usize = 2;

        if y_base + trunk_height + leaves_radius >= CHUNK_HEIGHT {
            return;
        }

        for i in 0..trunk_height {
            // Directly set block, ignore result for simplicity in terrain gen
            let _ = self.set_block(x, y_base + i, z, BlockType::OakLog);
        }

        let leaves_y_base = y_base + trunk_height - 1;
        for ly_offset in 0..=leaves_radius {
            let current_y = leaves_y_base + ly_offset;
            if current_y >= CHUNK_HEIGHT { continue; }

            for lx_offset in -(leaves_radius as isize)..=(leaves_radius as isize) {
                for lz_offset in -(leaves_radius as isize)..=(leaves_radius as isize) {
                    let block_x_isize = x as isize + lx_offset;
                    let block_z_isize = z as isize + lz_offset;

                    if block_x_isize < 0 || block_x_isize >= CHUNK_WIDTH as isize ||
                       block_z_isize < 0 || block_z_isize >= CHUNK_DEPTH as isize {
                        continue;
                    }

                    let block_x = block_x_isize as usize;
                    let block_z = block_z_isize as usize;

                    let dist_sq = lx_offset.pow(2) + lz_offset.pow(2);
                    if ly_offset < leaves_radius && dist_sq > (leaves_radius as isize).pow(2) {
                        continue;
                    }
                    if ly_offset == leaves_radius && dist_sq > ((leaves_radius -1) as isize).pow(2) {
                        continue;
                    }

                    if self.blocks[block_x][current_y][block_z].block_type == BlockType::Air {
                         let _ = self.set_block(block_x, current_y, block_z, BlockType::OakLeaves);
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

    pub fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Block> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            Some(&mut self.blocks[x][y][z])
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
            let new_emission = self.blocks[x][y][z].emission();
            if new_emission > 0 {
                self.blocks[x][y][z].block_light_level = new_emission;
            } else {
                self.blocks[x][y][z].block_light_level = 0;
            }
            Ok(())
        } else {
            Err("Coordinates out of chunk bounds")
        }
    }

    pub fn recalculate_skylight_based_on_global(
        &mut self,
        new_global_skylight_level: u8,
        sky_light_removal_queue: &mut VecDeque<LightNode>,
        sky_light_propagation_queue: &mut VecDeque<LightNode>,
        chunk_world_x_offset: i32,
        chunk_world_z_offset: i32,
    ) {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let mut current_max_sky = new_global_skylight_level;
                for y in (0..CHUNK_HEIGHT).rev() {
                    let block = &mut self.blocks[x][y][z];
                    let world_pos = glam::ivec3(
                        chunk_world_x_offset + x as i32,
                        y as i32,
                        chunk_world_z_offset + z as i32,
                    );

                    let old_sky_light = block.sky_light_level;

                    if block.opacity() >= MAX_LIGHT_LEVEL {
                        current_max_sky = 0;
                    }

                    let new_sky_light_for_block = current_max_sky;

                    if old_sky_light != new_sky_light_for_block {
                        if old_sky_light > 0 {
                            sky_light_removal_queue.push_back(LightNode {
                                pos: world_pos,
                                light_level: old_sky_light,
                            });
                        }
                        block.sky_light_level = new_sky_light_for_block;
                        if new_sky_light_for_block > 0 {
                            sky_light_propagation_queue.push_back(LightNode {
                                pos: world_pos,
                                light_level: new_sky_light_for_block,
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn propagate_light_in_chunk(&mut self) {
        let mut sky_queue: VecDeque<(usize, usize, usize, u8)> = VecDeque::new();
        let mut block_queue: VecDeque<(usize, usize, usize, u8)> = VecDeque::new();

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

        while let Some((x, y, z, light_val)) = sky_queue.pop_front() {
            let current_light_val = self.blocks[x][y][z].sky_light_level;
            if light_val < current_light_val && light_val != MAX_LIGHT_LEVEL {
                // continue;
            }
            if current_light_val == 0 { continue; }

            for dx_i32 in -1..=1_i32 {
                for dy_i32 in -1..=1_i32 {
                    for dz_i32 in -1..=1_i32 {
                        if (dx_i32.abs() + dy_i32.abs() + dz_i32.abs()) != 1 { continue; }

                        let nx = x as i32 + dx_i32;
                        let ny = y as i32 + dy_i32;
                        let nz = z as i32 + dz_i32;

                        if nx < 0 || nx >= CHUNK_WIDTH as i32 ||
                           ny < 0 || ny >= CHUNK_HEIGHT as i32 ||
                           nz < 0 || nz >= CHUNK_DEPTH as i32 {
                            continue;
                        }

                        let neighbor_x = nx as usize;
                        let neighbor_y = ny as usize;
                        let neighbor_z = nz as usize;

                        let neighbor_block_mut = &mut self.blocks[neighbor_x][neighbor_y][neighbor_z];
                        let opacity = neighbor_block_mut.opacity();

                        let mut reduction = 1u8;
                        if opacity >= MAX_LIGHT_LEVEL {
                            reduction = MAX_LIGHT_LEVEL.saturating_add(1);
                        } else if opacity > 0 {
                            reduction = reduction.saturating_add(opacity);
                        }

                        if dy_i32 == -1 && opacity == 0 && current_light_val == MAX_LIGHT_LEVEL {
                            reduction = 0;
                        }

                        let new_light = current_light_val.saturating_sub(reduction);

                        if new_light > neighbor_block_mut.sky_light_level {
                            neighbor_block_mut.sky_light_level = new_light;
                            sky_queue.push_back((neighbor_x, neighbor_y, neighbor_z, new_light));
                        }
                    }
                }
            }
        }

        while let Some((x, y, z, light_val)) = block_queue.pop_front() {
            let current_light_val = self.blocks[x][y][z].block_light_level;
            if light_val < current_light_val {
                // continue;
            }
            if current_light_val == 0 { continue; }

            for dx_i32 in -1..=1_i32 {
                for dy_i32 in -1..=1_i32 {
                    for dz_i32 in -1..=1_i32 {
                        if (dx_i32.abs() + dy_i32.abs() + dz_i32.abs()) != 1 { continue; }

                        let nx = x as i32 + dx_i32;
                        let ny = y as i32 + dy_i32;
                        let nz = z as i32 + dz_i32;

                        if nx < 0 || nx >= CHUNK_WIDTH as i32 ||
                           ny < 0 || ny >= CHUNK_HEIGHT as i32 ||
                           nz < 0 || nz >= CHUNK_DEPTH as i32 {
                            continue;
                        }

                        let neighbor_x = nx as usize;
                        let neighbor_y = ny as usize;
                        let neighbor_z = nz as usize;

                        let neighbor_block_mut = &mut self.blocks[neighbor_x][neighbor_y][neighbor_z];
                        let opacity = neighbor_block_mut.opacity();

                        let attenuation = 1u8.saturating_add(opacity);
                        let new_light = current_light_val.saturating_sub(attenuation);

                        if new_light > neighbor_block_mut.block_light_level {
                            neighbor_block_mut.block_light_level = new_light;
                            block_queue.push_back((neighbor_x, neighbor_y, neighbor_z, new_light));
                        }
                    }
                }
            }
        }
    }
}
