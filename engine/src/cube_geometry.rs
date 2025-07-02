use crate::Vertex;

const CUBE_VERTICES_DATA: &[Vertex] = &[
    // Front face (Z = -0.5)
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [-0.5, 0.5, -0.5], color: [1.0, 0.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, 0.5, -0.5], color: [1.0, 0.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    // Back face (Z = 0.5)
    Vertex { position: [-0.5, -0.5, 0.5], color: [0.0, 1.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, -0.5, 0.5], color: [0.0, 1.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, 0.5, 0.5], color: [0.0, 1.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [-0.5, 0.5, 0.5], color: [0.0, 1.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    // Right face (X = 0.5)
    Vertex { position: [0.5, -0.5, -0.5], color: [0.0, 0.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, 0.5, -0.5], color: [0.0, 0.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, 0.5, 0.5], color: [0.0, 0.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, -0.5, 0.5], color: [0.0, 0.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    // Left face (X = -0.5)
    Vertex { position: [-0.5, -0.5, 0.5], color: [1.0, 1.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [-0.5, 0.5, 0.5], color: [1.0, 1.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [-0.5, 0.5, -0.5], color: [1.0, 1.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 0.0], uv: [0.0, 0.0], light_level: 1.0, },
    // Top face (Y = 0.5)
    Vertex { position: [-0.5, 0.5, 0.5], color: [1.0, 0.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, 0.5, 0.5], color: [1.0, 0.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, 0.5, -0.5], color: [1.0, 0.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [-0.5, 0.5, -0.5], color: [1.0, 0.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    // Bottom face (Y = -0.5)
    Vertex { position: [-0.5, -0.5, 0.5], color: [0.0, 1.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
    Vertex { position: [0.5, -0.5, 0.5], color: [0.0, 1.0, 1.0], uv: [0.0, 0.0], light_level: 1.0, },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CubeFace {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

const NUM_VERTICES_PER_FACE: usize = 4;

const FACE_VERTEX_START_INDICES: [usize; 6] = [
    0,  // Front
    4,  // Back
    8,  // Right
    12, // Left
    16, // Top
    20, // Bottom
];

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
}
