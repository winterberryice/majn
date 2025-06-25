#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air,    // Optional, for empty spaces
    Dirt,
    Grass,
    // Add more block types here later if needed
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
    // We can add more properties later, like light levels, custom data, etc.
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Block { block_type }
    }

    pub fn is_solid(&self) -> bool {
        match self.block_type {
            BlockType::Air => false,
            _ => true, // All other current types are solid
        }
    }

    // Later, we can add methods here to get texture coordinates
    // based on BlockType and potentially block face.
    // For now, we'll keep it simple.
}
