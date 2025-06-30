#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air, // Optional, for empty spaces
    Dirt,
    Grass,
    Bedrock,
    OakLog,
    OakLeaves,
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

    // Returns atlas indices (column, row) for each face: [Front, Back, Right, Left, Top, Bottom]
    pub fn get_texture_atlas_indices(&self) -> [[f32; 2]; 6] {
        match self.block_type {
            BlockType::Dirt => [[2.0, 0.0]; 6], // Dirt texture at (2,0) in terrain.png
            BlockType::Grass => [
                [3.0, 0.0], // Front (Grass Side)
                [3.0, 0.0], // Back (Grass Side)
                [3.0, 0.0], // Right (Grass Side)
                [3.0, 0.0], // Left (Grass Side)
                [0.0, 0.0], // Top (Grass Top)
                [2.0, 0.0], // Bottom (Dirt)
            ],
            BlockType::Bedrock => [[1.0, 1.0]; 6], // Bedrock at (1,1)
            BlockType::Air => [[15.0, 15.0]; 6],   // Default/error texture (far corner of atlas)
            BlockType::OakLog => [
                [4.0, 1.0],
                [4.0, 1.0],
                [4.0, 1.0],
                [4.0, 1.0],
                [5.0, 1.0],
                [5.0, 1.0],
            ],
            BlockType::OakLeaves => [[4.0, 3.0]; 6],
        }
    }
}
