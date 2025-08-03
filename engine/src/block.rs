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
    pub tree_id: Option<u32>, // Added to identify trees
    pub sky_light: u8,
    pub block_light: u8,
    // We can add more properties later, like light levels, custom data, etc.
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Block {
            block_type,
            tree_id: None,
            sky_light: 0,
            block_light: 0,
        }
    }

    // Helper to create a block with a tree_id
    pub fn new_with_tree_id(block_type: BlockType, tree_id: u32) -> Self {
        Block {
            block_type,
            tree_id: Some(tree_id),
            sky_light: 0,
            block_light: 0,
        }
    }

    pub fn is_solid(&self) -> bool {
        match self.block_type {
            BlockType::Air => false,
            BlockType::OakLeaves => false, // Leaves are not solid for physics
            _ => true,                     // All other current types are solid
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self.block_type {
            BlockType::OakLeaves => true,
            BlockType::Air => true, // Air is also visually transparent
            _ => false,
        }
    }

    // Later, we can add methods here to get texture coordinates
    // based on BlockType and potentially block face.
    // For now, we'll keep it simple.

    // Returns atlas indices (column, row) for each face: [Front, Back, Right, Left, Top, Bottom]
    pub fn get_texture_atlas_indices(&self) -> [[f32; 2]; 6] {
        match self.block_type {
            BlockType::Dirt => [[2.0, 0.0]; 6], // dirt.png
            BlockType::Grass => [
                [1.0, 0.0], // Front (grass_block_side.png)
                [1.0, 0.0], // Back (grass_block_side.png)
                [1.0, 0.0], // Right (grass_block_side.png)
                [1.0, 0.0], // Left (grass_block_side.png)
                [0.0, 0.0], // Top (grass_block_top.png)
                [2.0, 0.0], // Bottom (dirt.png)
            ],
            BlockType::Bedrock => [[3.0, 0.0]; 6], // bedrock.png
            BlockType::Air => [[15.0, 15.0]; 6],   // Default/error texture (far corner of atlas)
            BlockType::OakLog => [
                [4.0, 0.0], // Side (oak_log.png)
                [4.0, 0.0], // Side (oak_log.png)
                [4.0, 0.0], // Side (oak_log.png)
                [4.0, 0.0], // Side (oak_log.png)
                [5.0, 0.0], // Top (oak_log_top.png)
                [5.0, 0.0], // Bottom (oak_log_top.png)
            ],
            BlockType::OakLeaves => [[6.0, 0.0]; 6], // oak_leaves.png
        }
    }
}
