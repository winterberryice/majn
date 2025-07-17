use crate::Vertex;

// 8 corners of a cube, unit size centered at origin
const CUBE_VERTICES_DATA: &[Vertex] = &[
    // Front face (Z = -0.5) - REORDERED FOR CCW from front (viewing towards +Z, normal 0,0,-1)
    // Order: BL, TL, TR, BR
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 0 (orig V0)
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 1 (orig V3)
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 2 (orig V2)
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 3 (orig V1)
    // Back face (Z = 0.5) - CCW from back (viewing towards -Z, normal 0,0,1)
    // Order: BL, BR, TR, TL (relative to its own view, e.g. V4,V5,V6,V7 is already this)
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 4 (orig V4)
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 5 (orig V5)
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 6 (orig V6)
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 7 (orig V7) - This order is correct
    // Right face (X = 0.5) - REORDERED FOR CCW from right (viewing towards -X, normal 1,0,0)
    // Order: BFR, TFR, TBR, BBR (Bottom-Front, Top-Front, Top-Back, Bottom-Back)
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 8 (orig V8)
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 9 (orig V11)
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 10 (orig V10)
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 11 (orig V9)
    // Left face (X = -0.5) - REORDERED FOR CCW from left (viewing towards +X, normal -1,0,0)
    // Order: BBL, TBL, TFL, BFL (Bottom-Back, Top-Back, Top-Front, Bottom-Front)
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 12 (orig V12)
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 13 (orig V15)
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 14 (orig V14)
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 15 (orig V13)
    // Top face (Y = 0.5) - REORDERED FOR CCW from above (viewing towards -Y, normal 0,1,0)
    // Order: Near-Left, Near-Right, Far-Right, Far-Left (relative to view from +Y)
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 16 (Original V19)
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 17 (Original V18)
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 18 (Original V17)
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [1.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 19 (Original V16)
    // Bottom face (Y = -0.5) - REORDERED FOR CCW from below (viewing towards +Y, normal 0,-1,0)
    // Order: Near-Left, Far-Left, Far-Right, Near-Right (relative to view from -Y)
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 1.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 20 (Original V20)
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.0, 1.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 21 (Original V23)
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.0, 1.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 22 (Original V22)
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 1.0, 1.0],
        uv: [0.0, 0.0],
        tree_id: 0,
        sky_light: 0.0,
    }, // 23 (Original V21)
];

// Indices for 24 vertices (6 faces * 4 vertices per face)
// Each face is two triangles (3 indices * 2 = 6 indices per face)
// 6 faces * 6 indices/face = 36 indices
// const CUBE_INDICES_DATA: &[u16] = &[
//     0, 1, 2, 0, 2, 3, // Front
//     4, 5, 6, 4, 6, 7, // Back
//     8, 9, 10, 8, 10, 11, // Right
//     12, 13, 14, 12, 14, 15, // Left
//     16, 17, 18, 16, 18, 19, // Top
//     20, 21, 22, 20, 22, 23, // Bottom
// ];

// pub fn cube_vertices() -> &'static [Vertex] {
//     CUBE_VERTICES_DATA
// }

// pub fn cube_indices() -> &'static [u16] {
//     CUBE_INDICES_DATA
// }

// Enum to identify cube faces. The order must match CUBE_VERTICES_DATA face order.
// Front (-Z), Back (+Z), Right (+X), Left (-X), Top (+Y), Bottom (-Y)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CubeFace {
    Front,  // Negative Z
    Back,   // Positive Z
    Right,  // Positive X
    Left,   // Negative X
    Top,    // Positive Y
    Bottom, // Negative Y
}

// These offsets assume each face has 4 vertices and 6 indices (2 triangles)
const NUM_VERTICES_PER_FACE: usize = 4;
// const NUM_INDICES_PER_FACE: usize = 6; // Not directly used for slicing CUBE_INDICES_DATA

// Start index in CUBE_VERTICES_DATA for each face
const FACE_VERTEX_START_INDICES: [usize; 6] = [
    0,  // Front
    4,  // Back
    8,  // Right
    12, // Left
    16, // Top
    20, // Bottom
];

// The indices for a single face (two triangles), assuming vertices are ordered 0, 1, 2, 3
// in a quad (e.g., bottom-left, bottom-right, top-right, top-left for CCW)
const LOCAL_FACE_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

impl CubeFace {
    pub fn get_vertices_template(&self) -> &'static [Vertex] {
        let start_index = match self {
            CubeFace::Front => FACE_VERTEX_START_INDICES[0],
            CubeFace::Back => FACE_VERTEX_START_INDICES[1],
            CubeFace::Right => FACE_VERTEX_START_INDICES[2],
            CubeFace::Left => FACE_VERTEX_START_INDICES[3],
            CubeFace::Top => FACE_VERTEX_START_INDICES[4],
            CubeFace::Bottom => FACE_VERTEX_START_INDICES[5],
        };
        &CUBE_VERTICES_DATA[start_index..start_index + NUM_VERTICES_PER_FACE]
    }

    pub fn get_local_indices(&self) -> &'static [u16] {
        &LOCAL_FACE_INDICES
    }

    // pub fn all_faces() -> [CubeFace; 6] {
    //     [
    //         CubeFace::Front,
    //         CubeFace::Back,
    //         CubeFace::Right,
    //         CubeFace::Left,
    //         CubeFace::Top,
    //         CubeFace::Bottom,
    //     ]
    // }
}

// Later we will add functions to get texture coordinates too.
// For now, color is part of Vertex.
// pub fn get_face_texture_coordinates(face_index: usize) -> &'static [[f32; 2]; 4] { ... }
