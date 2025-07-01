// Add MAX_LIGHT_LEVEL constant
pub const MAX_LIGHT_LEVEL: u8 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Dirt,
    Grass,
    Bedrock,
    OakLog,
    OakLeaves,
    Stone, // Added Stone
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    Water,
    Sand,
    Glass,     // Added Glass
    Torch,     // Added Torch
    Glowstone, // Added Glowstone
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
    pub sky_light_level: u8,  // 0-15
    pub block_light_level: u8, // 0-15
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Block {
            block_type,
            sky_light_level: 0, // Default to dark, will be updated by lighting engine
            block_light_level: 0, // Default to dark
        }
    }

    // Helper to get the brightest light level for rendering
    pub fn light_level(&self) -> u8 {
        self.sky_light_level.max(self.block_light_level)
    }

    // Light properties
    pub fn emission(&self) -> u8 {
        match self.block_type {
            BlockType::Torch => 14,
            BlockType::Glowstone => 15,
            // Potentially Lava in the future
            _ => 0,
        }
    }

    // How much this block impedes light passing *through* it.
    // Higher numbers mean more light is blocked.
    // 0 means fully transparent to light (like air to skylight).
    // 1 is standard for transparent blocks like leaves/glass when light spreads.
    // 15+ means opaque.
    pub fn opacity(&self) -> u8 {
        match self.block_type {
            BlockType::Air => 0, // Air doesn't block light spreading, skylight passes freely
            BlockType::Glass => 0, // Glass doesn't block light spreading (or very little, like 1 if preferred)
            BlockType::Water => 2, // Water dims light more significantly
            BlockType::OakLeaves => 1, // Leaves block a little light
            BlockType::Torch => 0, // Torches themselves don't block light
            // All other blocks are considered fully opaque for now
            BlockType::Dirt
            | BlockType::Grass
            | BlockType::Bedrock
            | BlockType::OakLog
            | BlockType::Stone
            | BlockType::CoalOre
            | BlockType::IronOre
            | BlockType::GoldOre
            | BlockType::DiamondOre
            | BlockType::Sand
            | BlockType::Glowstone // Glowstone emits light but is opaque to light passing through it
            => 255, // Using a high value to signify effectively infinite opacity for spread calculation
        }
    }

    pub fn is_solid(&self) -> bool {
        match self.block_type {
            BlockType::Air => false,
            BlockType::OakLeaves => false, // Leaves are not solid for physics
            BlockType::Water => false,     // Water is not solid
            BlockType::Glass => false,     // Glass is not solid in the same way as stone
            BlockType::Torch => false,
            _ => true, // All other current types are solid
        }
    }

    // is_transparent is now more about visual transparency for rendering culling,
    // rather than light transmission (which is handled by opacity).
    pub fn is_transparent_for_rendering(&self) -> bool {
        match self.block_type {
            BlockType::Air => true,
            BlockType::OakLeaves => true,
            BlockType::Water => true, // Water is visually transparent
            BlockType::Glass => true, // Glass is visually transparent
            BlockType::Torch => true,
            _ => false,
        }
    }

    // Returns atlas indices (column, row) for each face: [Front, Back, Right, Left, Top, Bottom]
    // TODO: Update with new block types
    pub fn get_texture_atlas_indices(&self) -> [[f32; 2]; 6] {
        match self.block_type {
            BlockType::Dirt => [[2.0, 0.0]; 6],
            BlockType::Grass => [
                [3.0, 0.0], // Side
                [3.0, 0.0], // Side
                [3.0, 0.0], // Side
                [3.0, 0.0], // Side
                [0.0, 0.0], // Top
                [2.0, 0.0], // Bottom (Dirt)
            ],
            BlockType::Bedrock => [[1.0, 1.0]; 6],
            BlockType::OakLog => [[4.0, 1.0]; 6], // Top/Bottom are [5.0, 1.0] in current, but let's simplify for now or assume side texture for all
            // BlockType::OakLog => [ // More accurate OakLog
            //     [4.0, 1.0], // Side
            //     [4.0, 1.0], // Side
            //     [4.0, 1.0], // Side
            //     [4.0, 1.0], // Side
            //     [5.0, 1.0], // Top
            //     [5.0, 1.0], // Bottom
            // ],
            BlockType::OakLeaves => [[4.0, 3.0]; 6], // Assuming leaves are similar on all sides for now
            BlockType::Stone => [[1.0, 0.0]; 6],     // Stone texture
            BlockType::CoalOre => [[2.0, 2.0]; 6],   // Coal Ore texture
            BlockType::IronOre => [[1.0, 2.0]; 6],   // Iron Ore texture
            BlockType::GoldOre => [[0.0, 2.0]; 6],   // Gold Ore texture
            BlockType::DiamondOre => [[0.0, 3.0]; 6], // Diamond Ore texture (example, might not be in terrain.png)
            BlockType::Water => [[13.0, 12.0]; 6],  // Water texture (example from Minecraft)
            BlockType::Sand => [[2.0, 1.0]; 6],     // Sand texture
            BlockType::Glass => [[1.0, 3.0]; 6],    // Glass texture
            BlockType::Torch => [[0.0, 5.0]; 6],    // Torch texture (often a special model, but atlas coords for a flat representation for now)
            BlockType::Glowstone => [[3.0, 6.0]; 6], // Glowstone texture
            BlockType::Air => [[15.0, 15.0]; 6], // Default/error texture (far corner of atlas) - Should not be rendered
        }
    }
}
