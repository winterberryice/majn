use crate::block::{Block, BlockType}; // Assuming block.rs is in the same directory

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
                for y in 0..CHUNK_HEIGHT {
                    if y < surface_level {
                        self.blocks[x][y][z] = Block::new(BlockType::Dirt);
                    } else if y == surface_level {
                        self.blocks[x][y][z] = Block::new(BlockType::Grass);
                    } // Above surface_level remains Air (as initialized)
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
            Ok(())
        } else {
            Err("Coordinates out of chunk bounds")
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
