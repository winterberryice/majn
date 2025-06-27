use wgpu::util::DeviceExt;
use glam::{Mat4, IVec3}; // Import IVec3, remove Vec3 for now to test warning

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct WireframeVertex {
    position: [f32; 3],
}

impl WireframeVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<WireframeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // Corresponds to layout(location = 0) in shader
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// Vertices for a 1x1x1 cube centered at (0.5, 0.5, 0.5) to align with block coords
// Blocks are from (0,0,0) to (1,1,1) in their local space.
// Wireframe will be slightly larger to avoid z-fighting, or rendered with depth bias/offset.
// For now, let's define it from 0.0 to 1.0.
const WIREFRAME_VERTICES: &[WireframeVertex] = &[
    // Bottom face
    WireframeVertex { position: [0.0, 0.0, 0.0] }, // 0
    WireframeVertex { position: [1.0, 0.0, 0.0] }, // 1
    WireframeVertex { position: [1.0, 0.0, 1.0] }, // 2
    WireframeVertex { position: [0.0, 0.0, 1.0] }, // 3
    // Top face
    WireframeVertex { position: [0.0, 1.0, 0.0] }, // 4
    WireframeVertex { position: [1.0, 1.0, 0.0] }, // 5
    WireframeVertex { position: [1.0, 1.0, 1.0] }, // 6
    WireframeVertex { position: [0.0, 1.0, 1.0] }, // 7
];

const WIREFRAME_INDICES: &[u16] = &[
    0, 1, 1, 2, 2, 3, 3, 0, // Bottom face
    4, 5, 5, 6, 6, 7, 7, 4, // Top face
    0, 4, 1, 5, 2, 6, 3, 7, // Connecting lines
];

pub struct ModelUniformData {
    model_matrix: Mat4,
}

impl ModelUniformData {
    pub fn new() -> Self {
        Self {
            model_matrix: Mat4::IDENTITY,
        }
    }
    pub fn update_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
    }
}


pub struct WireframeRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    model_uniform_buffer: wgpu::Buffer,
    model_bind_group: wgpu::BindGroup,
    model_data: ModelUniformData, // To store matrix before writing to buffer
}

impl WireframeRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Wireframe Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("wireframe_shader.wgsl").into()),
        });

        let model_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("wireframe_model_bind_group_layout"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Wireframe Render Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout, &model_bind_group_layout], // Group 0 for camera, Group 1 for model
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Wireframe Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[WireframeVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // Use alpha blending for wireframe
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for wireframes
                polygon_mode: wgpu::PolygonMode::Fill, // Does not apply to LineList
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float, // Ensure this matches main pass
                depth_write_enabled: true, // Usually true, but might be false if you want it to always draw on top of closer objects (not desired here)
                depth_compare: wgpu::CompareFunction::LessEqual, // Draw if equal or closer
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState { // Add depth bias to prevent Z-fighting
                    constant: -2, // Negative bias pulls towards camera slightly (experiment with values)
                    slope_scale: -2.0, // Experiment with values
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Vertex Buffer"),
            contents: bytemuck::cast_slice(WIREFRAME_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Index Buffer"),
            contents: bytemuck::cast_slice(WIREFRAME_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = WIREFRAME_INDICES.len() as u32;

        let model_data = ModelUniformData::new();
        let model_uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Wireframe Model Uniform Buffer"),
                contents: bytemuck::cast_slice(&[model_data.model_matrix.to_cols_array_2d()]), // Store as [[f32;4];4]
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &model_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("wireframe_model_bind_group"),
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            model_uniform_buffer,
            model_bind_group,
            model_data,
        }
    }

    pub fn update_model_matrix(&mut self, position: IVec3) {
        let translation = Mat4::from_translation(position.as_vec3());
        // If wireframe cube is 0-1, this translation is enough.
        // If it was centered at 0,0,0 and size 1, you'd add 0.5 to position.
        self.model_data.update_matrix(translation);
    }

    pub fn draw<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, queue: &wgpu::Queue) {
        // Update the GPU buffer with the current model matrix
        queue.write_buffer(
            &self.model_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.model_data.model_matrix.to_cols_array_2d()]),
        );

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.model_bind_group, &[]); // Bind group 1 for model
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
