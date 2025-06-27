use wgpu::util::DeviceExt;
use glam::{Mat4, IVec3, Vec3};
// use lazy_static::lazy_static; // Removed to test warning - macro might handle scope

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

const LINE_THICKNESS: f32 = 0.05; // Control thickness here

fn create_quad(p1: Vec3, p2: Vec3, offset_axis: Vec3, thickness_half: f32) -> (Vec<WireframeVertex>, Vec<u16>) {
    let mut quad_vertices = Vec::with_capacity(4);
    let mut quad_indices = Vec::with_capacity(6);

    let offset = offset_axis * thickness_half;

    quad_vertices.push(WireframeVertex { position: (p1 - offset).into() }); // 0
    quad_vertices.push(WireframeVertex { position: (p2 - offset).into() }); // 1
    quad_vertices.push(WireframeVertex { position: (p2 + offset).into() }); // 2
    quad_vertices.push(WireframeVertex { position: (p1 + offset).into() }); // 3

    quad_indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

    (quad_vertices, quad_indices)
}


fn generate_thick_line_cube_geometry() -> (Vec<WireframeVertex>, Vec<u16>) {
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let ht = LINE_THICKNESS / 2.0;

    let corners_raw: [[f32; 3]; 8] = [
        [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0], // Bottom
        [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0], // Top
    ];
    let corners: Vec<Vec3> = corners_raw.iter().map(|&arr| Vec3::from(arr)).collect();

    let edges: [(usize, usize); 12] = [
        (0, 1), (1, 2), (2, 3), (3, 0), // Bottom face
        (4, 5), (5, 6), (6, 7), (7, 4), // Top face
        (0, 4), (1, 5), (2, 6), (3, 7), // Vertical edges
    ];

    for &(idx1, idx2) in edges.iter() {
        let p1 = corners[idx1];
        let p2 = corners[idx2];
        let current_vertex_offset = all_vertices.len() as u16;

        let line_direction = (p2 - p1).normalize_or_zero();
        let offset_axis: Vec3;

        if line_direction.x.abs() > 0.9 { // Line along X-axis
            offset_axis = Vec3::Y; // Expand quad in Y direction
        } else if line_direction.y.abs() > 0.9 { // Line along Y-axis
            offset_axis = Vec3::X; // Expand quad in X direction
        } else { // Line along Z-axis (or diagonal, though cube edges are axis-aligned)
            offset_axis = Vec3::X; // Expand quad in X direction (arbitrary for Z-lines)
        }

        let (quad_verts, quad_idxs) = create_quad(p1, p2, offset_axis, ht);
        all_vertices.extend(quad_verts);
        all_indices.extend(quad_idxs.into_iter().map(|i| i + current_vertex_offset));
    }
    (all_vertices, all_indices)
}


lazy_static::lazy_static! {
    static ref THICK_LINE_CUBE_GEOMETRY: (Vec<WireframeVertex>, Vec<u16>) = generate_thick_line_cube_geometry();
}

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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // Changed to TriangleList
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for simple quads, or Some(wgpu::Face::Back) if normals are consistent
                polygon_mode: wgpu::PolygonMode::Fill,
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

        let (vertices, indices) = THICK_LINE_CUBE_GEOMETRY.clone(); // Clone from lazy_static

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Vertex Buffer (Thick Lines)"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Index Buffer (Thick Lines)"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;

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
