use glam::Mat4;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    // Using separate components for model matrix to avoid potential padding issues with Mat4 directly.
    // Alternatively, ensure Mat4 is correctly Pod. glam::Mat4 should be fine, but this is explicit.
    model_matrix_col_0: [f32; 4],
    model_matrix_col_1: [f32; 4],
    model_matrix_col_2: [f32; 4],
    model_matrix_col_3: [f32; 4],
    color: [f32; 3], // R, G, B
    _padding: f32, // Ensure alignment for next instance if necessary (vec3 is 12 bytes, mat4 is 64)
                   // A single f32 here makes InstanceRaw 16*4 + 12 + 4 = 64 + 16 = 80 bytes.
                   // This should be fine. If issues, align to 16 bytes multiple if colors were vec4.
}

impl InstanceRaw {
    pub fn new(model_matrix: Mat4, color: [f32; 3]) -> Self {
        Self {
            model_matrix_col_0: model_matrix.x_axis.into(), // Column 0
            model_matrix_col_1: model_matrix.y_axis.into(), // Column 1
            model_matrix_col_2: model_matrix.z_axis.into(), // Column 2
            model_matrix_col_3: model_matrix.w_axis.into(), // Column 3
            color,
            _padding: 0.0,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // This attribute describes data that changes per instance.
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Model Matrix (mat4x4<f32>) is passed as four vec4<f32> attributes
                // Shader location 5, 6, 7, 8 (assuming vertex attributes use 0-4)
                wgpu::VertexAttribute { // Column 0
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute { // Column 1
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute { // Column 2
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute { // Column 3
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Color attribute
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress, // after 4 vec4s for matrix
                    shader_location: 9, // Next available location
                    format: wgpu::VertexFormat::Float32x3, // For RGB color
                },
            ],
        }
    }
}
