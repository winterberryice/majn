// To store properties for each BlockType
pub struct BlockTypeProperties {
    pub is_solid: bool,
    pub is_transparent: bool,
    pub texture_atlas_indices: [[f32; 2]; 6],
    pub emission: u8, // Light emission level (0-15)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] // Added Hash
pub enum BlockType {
    Air,
    Dirt,
    Grass,
    Bedrock,
    OakLog,
    OakLeaves,
    // TODO: Add Torch with emission: 14
}

// Array to store properties for each block type.
// This allows us to define block properties in one place.
pub const BLOCK_TYPE_PROPERTIES: [BlockTypeProperties; 6] = [
    // Air
    BlockTypeProperties {
        is_solid: false,
        is_transparent: true, // Air is transparent
        texture_atlas_indices: [[15.0, 15.0]; 6], // Assuming a transparent/empty texture or not rendered
        emission: 0,
    },
    // Dirt
    BlockTypeProperties {
        is_solid: true,
        is_transparent: false,
        texture_atlas_indices: [[2.0, 0.0]; 6],
        emission: 0,
    },
    // Grass
    BlockTypeProperties {
        is_solid: true,
        is_transparent: false,
        texture_atlas_indices: [
            [3.0, 0.0], // Front (Grass Side)
            [3.0, 0.0], // Back (Grass Side)
            [3.0, 0.0], // Right (Grass Side)
            [3.0, 0.0], // Left (Grass Side)
            [0.0, 0.0], // Top (Grass Top)
            [2.0, 0.0], // Bottom (Dirt)
        ],
        emission: 0,
    },
    // Bedrock
    BlockTypeProperties {
        is_solid: true,
        is_transparent: false,
        texture_atlas_indices: [[1.0, 1.0]; 6],
        emission: 0,
    },
    // OakLog
    BlockTypeProperties {
        is_solid: true,
        is_transparent: false,
        texture_atlas_indices: [
            [4.0, 1.0], // Side
            [4.0, 1.0], // Side
            [4.0, 1.0], // Side
            [4.0, 1.0], // Side
            [5.0, 1.0], // Top/Bottom
            [5.0, 1.0], // Top/Bottom
        ],
        emission: 0,
    },
    // OakLeaves
    BlockTypeProperties {
        is_solid: false, // Leaves are not solid for physics but opaque for light initially
        is_transparent: true, // Visually transparent, but consider light opacity separately
        texture_atlas_indices: [[4.0, 3.0]; 6],
        emission: 0,
    },
];

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
    // We can add more properties later, like light levels, custom data, etc.
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Block { block_type }
    }

    // Helper to get properties for this block's type
    pub fn properties(&self) -> &'static BlockTypeProperties {
        &BLOCK_TYPE_PROPERTIES[self.block_type as usize]
    }

    pub fn is_solid(&self) -> bool {
        self.properties().is_solid
    }

    // This now refers to visual transparency for rendering.
    // For lighting, we'll need a new concept or adjust this.
    // Let's assume for now: if a block is visually transparent, light passes through.
    // If it's visually opaque, light is blocked. This might need refinement.
    pub fn is_transparent(&self) -> bool {
        self.properties().is_transparent
    }

    // NEW: Opacity for light propagation.
    // For now, let's say it's the opposite of visual transparency.
    // This might need to be a separate property later if some blocks are
    // visually transparent but block light (e.g. stained glass) or vice-versa.
    pub fn is_opaque_for_light(&self) -> bool {
        !self.properties().is_transparent
        // A more direct way:
        // match self.block_type {
        //     BlockType::Air => false,
        //     BlockType::OakLeaves => false, // Light passes through leaves
        //     _ => true, // Other blocks are opaque to light
        // }
    }


    pub fn get_light_emission(&self) -> u8 {
        self.properties().emission
    }

    // Returns atlas indices (column, row) for each face: [Front, Back, Right, Left, Top, Bottom]
    pub fn get_texture_atlas_indices(&self) -> [[f32; 2]; 6] {
        self.properties().texture_atlas_indices
    }
}
