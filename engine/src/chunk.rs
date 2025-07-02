use crate::block::{Block, BlockType};
use rand::Rng; // Assuming block.rs is in the same directory

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 32; // Reduced height for now
pub const CHUNK_DEPTH: usize = 16;
pub const MAX_LIGHT: u8 = 15;

pub struct Chunk {
    pub coord: (i32, i32),        // Add world coordinates (x, z) for the chunk
    blocks: Vec<Vec<Vec<Block>>>, // Stored as [x][y][z]
    light_levels: Vec<Vec<Vec<u8>>>, // Stored as [x][y][z]
}

impl Chunk {
    pub fn new(coord_x: i32, coord_z: i32) -> Self {
        // Initialize with Air blocks
        let blocks =
            vec![vec![vec![Block::new(BlockType::Air); CHUNK_DEPTH]; CHUNK_HEIGHT]; CHUNK_WIDTH];
        // Initialize light levels to 0
        let light_levels =
            vec![vec![vec![0u8; CHUNK_DEPTH]; CHUNK_HEIGHT]; CHUNK_WIDTH];
        Chunk {
            coord: (coord_x, coord_z),
            blocks,
            light_levels,
        }
    }

    pub fn calculate_initial_sunlight(&mut self) {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let mut light_penetration = MAX_LIGHT;
                for y in (0..CHUNK_HEIGHT).rev() { // Iterate from top to bottom
                    let block = &mut self.blocks[x][y][z];
                    if light_penetration > 0 {
                        if block.is_solid() {
                             // Solid blocks stop light penetration for blocks below them in this column.
                             // The current solid block still gets the current light_penetration value.
                            self.light_levels[x][y][z] = light_penetration;
                            light_penetration = 0; // Stop light for blocks below this one in this column
                        } else if block.is_transparent() {
                            // Transparent blocks (like leaves) get full light and reduce it slightly for below.
                            self.light_levels[x][y][z] = light_penetration;
                            // For now, leaves don't dim light, but this is where you could add it:
                            // light_penetration = light_penetration.saturating_sub(1);
                        } else { // Air block
                            self.light_levels[x][y][z] = light_penetration;
                        }
                    } else {
                        self.light_levels[x][y][z] = 0;
                    }
                }
            }
        }
    }

    #[allow(deprecated)] // Allow rand::thread_rng and rng.random_bool for now
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
        let mut rng = rand::thread_rng(); // Create a random number generator
        const TREE_CHANCE: f64 = 0.02; // 2% chance to try and place a tree at any given spot

        for x in 2..(CHUNK_WIDTH - 2) {
            // Iterate with a margin to keep trees from chunk edges
            for z in 2..(CHUNK_DEPTH - 2) {
                // Check if the block at the surface level is grass
                if self.blocks[x][surface_level][z].block_type == BlockType::Grass {
                    // Use the random number generator to decide if a tree should grow here
                    if rng.random_bool(TREE_CHANCE) { // Changed to random_bool as per warning
                        // The ground level for the tree trunk is one block above the grass.
                        self.place_tree(x, surface_level + 1, z);
                    }
                }
            }
        }
        // After all terrain and features are placed, calculate sunlight
        self.calculate_initial_sunlight();
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

    // Helper to get a light level at a given coordinate
    pub fn get_light_level(&self, x: usize, y: usize, z: usize) -> Option<u8> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            Some(self.light_levels[x][y][z])
        } else {
            None
        }
    }

    // Helper to set a block at a given coordinate
    // Returns Result<(), &str> to indicate success or out-of-bounds error
    pub fn set_block(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        block_type: BlockType,
    ) -> Result<(), &'static str> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            self.blocks[x][y][z] = Block::new(block_type);
            // NOTE: Light level is not set here directly.
            // The light calculation pass will handle it.
            // For dynamic updates, a call to recalculate light for this chunk
            // (and potentially neighbors) would be needed after a block is set.
            Ok(())
        } else {
            Err("Coordinates out of chunk bounds")
        }
    }

    // Helper to set a light level at a given coordinate
    pub fn set_light_level(&mut self, x: usize, y: usize, z: usize, level: u8) -> Result<(), &'static str> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            self.light_levels[x][y][z] = level.min(MAX_LIGHT); // Ensure light level doesn't exceed max
            Ok(())
        } else {
            Err("Coordinates out of chunk bounds for light level")
        }
    }
}

// Default implementation for Chunk, useful for initialization
// Now requires coordinates, so a generic default might not make sense
// unless we default to (0,0). For now, let's remove it or make it explicit.
// impl Default for Chunk {
//     fn default() -> Self {
//         Self::new(0, 0) // Default to chunk at (0,0)
//     }
// }
