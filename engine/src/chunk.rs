use crate::block::{Block, BlockType};
use rand::Rng; // Assuming block.rs is in the same directory

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 32; // Reduced height for now
pub const CHUNK_DEPTH: usize = 16;

pub struct Chunk {
    pub coord: (i32, i32),        // Add world coordinates (x, z) for the chunk
    blocks: Vec<Vec<Vec<Block>>>, // Stored as [x][y][z]
}

impl Chunk {
    pub fn new(coord_x: i32, coord_z: i32) -> Self {
        // Initialize with Air blocks
        let blocks =
            vec![vec![vec![Block::new(BlockType::Air); CHUNK_DEPTH]; CHUNK_HEIGHT]; CHUNK_WIDTH];
        Chunk {
            coord: (coord_x, coord_z),
            blocks,
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
        let mut rng = rand::thread_rng();
        const TREE_CHANCE: f64 = 0.02;
        let mut next_tree_id: u32 = 1;

        for x_coord in 2..(CHUNK_WIDTH - 2) {
            for z_coord in 2..(CHUNK_DEPTH - 2) {
                if self.blocks[x_coord][surface_level][z_coord].block_type == BlockType::Grass {
                    if rng.gen_bool(TREE_CHANCE) {
                        self.place_tree(x_coord, surface_level + 1, z_coord, next_tree_id, &mut rng);
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
        let trunk_height: usize = rng.gen_range(3..=5);
        let canopy_radius: isize = 2;
        let canopy_base_y_offset: usize = trunk_height.saturating_sub(2); // Canopy starts relative to trunk top
        let canopy_max_height_above_base: usize = 3; // How many layers of leaves max

        let tree_top_y = y_base + trunk_height;
        if tree_top_y + canopy_max_height_above_base >= CHUNK_HEIGHT { // Check overall tree height
            return;
        }

        // Place the trunk
        for i in 0..trunk_height {
            if y_base + i < CHUNK_HEIGHT { // Ensure trunk is within bounds
                 self.set_block(x, y_base + i, z, BlockType::OakLog).unwrap_or_default();
            }
        }

        // Place the leaves canopy (randomized)
        let canopy_center_y_world = y_base + trunk_height -1; // Top of the trunk block

        // Iterate a bounding box for potential leaves
        // Y iterates from a bit below the top of the trunk to a bit above
        let y_start_canopy = (y_base + canopy_base_y_offset).max(0); // Ensure non-negative
        let y_end_canopy = (y_base + trunk_height + 1).min(CHUNK_HEIGHT -1) ; // Extend a bit above trunk

        for ly_world in y_start_canopy..=y_end_canopy {
            // Adjust radius based on y level (e.g. smaller at very top/bottom of canopy)
            let y_dist_from_canopy_center = (ly_world as isize - canopy_center_y_world as isize).abs();
            let current_layer_radius = if y_dist_from_canopy_center <= 0 { // center layer (around top of trunk)
                canopy_radius
            } else if y_dist_from_canopy_center == 1 {
                canopy_radius // one layer above/below also full radius
            } else {
                (canopy_radius - 1).max(1) // taper off for layers further away
            };

            for lx_offset in -current_layer_radius..=current_layer_radius {
                for lz_offset in -current_layer_radius..=current_layer_radius {
                    let current_x_world = x as isize + lx_offset;
                    let current_z_world = z as isize + lz_offset;

                    // Boundary checks for X and Z
                    if current_x_world < 0 || current_x_world >= CHUNK_WIDTH as isize ||
                       current_z_world < 0 || current_z_world >= CHUNK_DEPTH as isize {
                        continue;
                    }

                    let (ux, uy, uz) = (current_x_world as usize, ly_world, current_z_world as usize);

                    // Don't place leaves inside the trunk core, except for the very top layer
                    if lx_offset == 0 && lz_offset == 0 && uy < canopy_center_y_world {
                        continue;
                    }

                    // Simple spherical/rounded shape check (optional, can be adjusted)
                    if lx_offset * lx_offset + lz_offset * lz_offset > current_layer_radius * current_layer_radius {
                        if y_dist_from_canopy_center > 0 { // Only apply stricter spherical for non-central Y layers
                           continue;
                        } else if lx_offset*lx_offset + lz_offset*lz_offset > (canopy_radius+1)*(canopy_radius+1) { // allow slightly more spread on central layer
                            continue;
                        }
                    }

                    let mut probability = 0.65; // Base probability
                    // Reduce probability for outer parts of a layer, more aggressively for non-central y-layers
                    let radius_check = if y_dist_from_canopy_center > 0 {
                        (current_layer_radius * current_layer_radius) as f64 * 0.5 // Tighter for upper/lower layers
                    } else {
                        (current_layer_radius * current_layer_radius) as f64 * 0.75 // Looser for main canopy layer
                    };
                    if (dist_sq_horiz as f64) > radius_check {
                        probability *= 0.5; // Halve probability if further out
                    }


                    if rng.gen_bool(probability) {
                        if self.blocks[ux][uy][uz].block_type == BlockType::Air {
                            self.set_block_with_tree_id(ux, uy, uz, BlockType::OakLeaves, tree_id).unwrap_or_default();
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
}
