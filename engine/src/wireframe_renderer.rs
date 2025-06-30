use wgpu::util::DeviceExt;
use glam::{Mat4, IVec3, Vec3};

// Imports for culling
use crate::world::World;
use crate::block::BlockType; // Assuming BlockType::Air is defined
use crate::raycast::BlockFace; // To identify faces

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

const MARGIN: f32 = 0.05; // Distance from the edge of the face
const QUAD_THICKNESS: f32 = 0.03; // Thickness of the quad strips

// Helper function to create a single quad given 4 corner points
fn create_strip_quad(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, vertices: &mut Vec<WireframeVertex>, indices: &mut Vec<u16>) {
    let base_vertex_index = vertices.len() as u16;
    vertices.push(WireframeVertex { position: p0.into() });
    vertices.push(WireframeVertex { position: p1.into() });
    vertices.push(WireframeVertex { position: p2.into() });
    vertices.push(WireframeVertex { position: p3.into() });
    indices.extend_from_slice(&[
        base_vertex_index, base_vertex_index + 1, base_vertex_index + 2,
        base_vertex_index, base_vertex_index + 2, base_vertex_index + 3,
    ]);
}

// Generates vertices and indices for 4 quads on a single face of a cube.
// The cube is assumed to be at (0,0,0) to (1,1,1).
// `face_normal` points outwards from the face.
// `axis1` and `axis2` are orthogonal unit vectors spanning the plane of the face.
// `face_center_offset` is the offset from cube origin (0,0,0) to the center of the face (e.g., (1, 0.5, 0.5) for +X face)
fn generate_quads_for_face(
    face_center_offset: Vec3,
    axis1: Vec3, // e.g., Vec3::Y for +X face
    axis2: Vec3, // e.g., Vec3::Z for +X face
    vertices: &mut Vec<WireframeVertex>,
    indices: &mut Vec<u16>,
) {
    let m = MARGIN;
    let t = QUAD_THICKNESS;

    // Half extents of the face
    let h1 = 0.5; // Half extent along axis1
    let h2 = 0.5; // Half extent along axis2

    // Quad 1 (along axis1, min end of axis2)
    let q1p0 = face_center_offset - axis2 * h2 + axis1 * (-h1 + m) + axis2 * m;
    let q1p1 = face_center_offset - axis2 * h2 + axis1 * (h1 - m)  + axis2 * m;
    let q1p2 = face_center_offset - axis2 * h2 + axis1 * (h1 - m)  + axis2 * (m + t);
    let q1p3 = face_center_offset - axis2 * h2 + axis1 * (-h1 + m) + axis2 * (m + t);
    create_strip_quad(q1p0, q1p1, q1p2, q1p3, vertices, indices);

    // Quad 2 (along axis1, max end of axis2)
    let q2p0 = face_center_offset + axis2 * h2 + axis1 * (-h1 + m) - axis2 * (m + t);
    let q2p1 = face_center_offset + axis2 * h2 + axis1 * (h1 - m)  - axis2 * (m + t);
    let q2p2 = face_center_offset + axis2 * h2 + axis1 * (h1 - m)  - axis2 * m;
    let q2p3 = face_center_offset + axis2 * h2 + axis1 * (-h1 + m) - axis2 * m;
    create_strip_quad(q2p0, q2p1, q2p2, q2p3, vertices, indices);

    // Quad 3 (along axis2, min end of axis1)
    // Adjusted to avoid overlap and use full inner length
    let q3p0 = face_center_offset - axis1 * h1 + axis2 * (-h2 + m + t) + axis1 * m; // Start after quad1's thickness
    let q3p1 = face_center_offset - axis1 * h1 + axis2 * (h2 - m - t)  + axis1 * m; // End before quad2's thickness
    let q3p2 = face_center_offset - axis1 * h1 + axis2 * (h2 - m - t)  + axis1 * (m + t);
    let q3p3 = face_center_offset - axis1 * h1 + axis2 * (-h2 + m + t) + axis1 * (m + t);
    create_strip_quad(q3p0, q3p1, q3p2, q3p3, vertices, indices);

    // Quad 4 (along axis2, max end of axis1)
    // Adjusted to avoid overlap and use full inner length
    let q4p0 = face_center_offset + axis1 * h1 + axis2 * (-h2 + m + t) - axis1 * (m+t);
    let q4p1 = face_center_offset + axis1 * h1 + axis2 * (h2 - m - t)  - axis1 * (m+t);
    let q4p2 = face_center_offset + axis1 * h1 + axis2 * (h2 - m - t)  - axis1 * m;
    let q4p3 = face_center_offset + axis1 * h1 + axis2 * (-h2 + m + t) - axis1 * m;
    create_strip_quad(q4p0, q4p1, q4p2, q4p3, vertices, indices);
}


fn generate_face_quads_cube_geometry() -> (Vec<WireframeVertex>, Vec<u16>, Vec<(BlockFace, u32, u32)>) {
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut face_render_info = Vec::new(); // Stores (BlockFace, index_offset, num_indices_for_face)

    let mut generate_and_record = |face: BlockFace, center_offset, axis1, axis2| {
        let start_index_count = all_indices.len() as u32;
        let start_vertex_count = all_vertices.len();
        generate_quads_for_face(center_offset, axis1, axis2, &mut all_vertices, &mut all_indices);

        // Correct indices to be relative to the start of *this face's* vertices, then add global offset
        // This is complex if create_strip_quad adds global indices.
        // Let's assume create_strip_quad correctly uses current all_vertices.len()
        let num_new_indices = (all_indices.len() as u32) - start_index_count;
        if num_new_indices > 0 { // Only add if quads were actually generated
             face_render_info.push((face, start_index_count, num_new_indices));
        }
    };

    // Cube centered at (0.5, 0.5, 0.5) with side length 1. Vertices from 0.0 to 1.0.
    // +X face (East)
    generate_and_record(BlockFace::PosX, Vec3::new(1.0, 0.5, 0.5), Vec3::Y, Vec3::Z);
    // -X face (West)
    generate_and_record(BlockFace::NegX, Vec3::new(0.0, 0.5, 0.5), Vec3::Y, Vec3::Z);
    // +Y face (Top)
    generate_and_record(BlockFace::PosY, Vec3::new(0.5, 1.0, 0.5), Vec3::X, Vec3::Z);
    // -Y face (Bottom)
    generate_and_record(BlockFace::NegY, Vec3::new(0.5, 0.0, 0.5), Vec3::X, Vec3::Z);
    // +Z face (South)
    generate_and_record(BlockFace::PosZ, Vec3::new(0.5, 0.5, 1.0), Vec3::X, Vec3::Y);
    // -Z face (North)
    generate_and_record(BlockFace::NegZ, Vec3::new(0.5, 0.5, 0.0), Vec3::X, Vec3::Y);

    (all_vertices, all_indices, face_render_info)
}


lazy_static::lazy_static! {
    static ref FACE_QUADS_CUBE_GEOMETRY: (Vec<WireframeVertex>, Vec<u16>, Vec<(BlockFace, u32, u32)>) =
        generate_face_quads_cube_geometry();
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
    // num_indices: u32, // No longer a single number for all indices
    face_render_info: Vec<(BlockFace, u32, u32)>, // (face, index_offset, count)
    model_uniform_buffer: wgpu::Buffer,
    model_bind_group: wgpu::BindGroup,
    model_data: ModelUniformData,
    current_selected_block_pos: Option<IVec3>, // Needed for culling
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
            bind_group_layouts: &[camera_bind_group_layout, &model_bind_group_layout],
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: -2,
                    slope_scale: -2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let (vertices, indices, face_info) = FACE_QUADS_CUBE_GEOMETRY.clone();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Vertex Buffer (Face Quads)"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Index Buffer (Face Quads)"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let model_data = ModelUniformData::new();
        let model_uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Wireframe Model Uniform Buffer"),
                contents: bytemuck::cast_slice(&[model_data.model_matrix.to_cols_array_2d()]),
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
            face_render_info: face_info,
            model_uniform_buffer,
            model_bind_group,
            model_data,
            current_selected_block_pos: None,
        }
    }

    pub fn update_selection(&mut self, position: Option<IVec3>) {
        self.current_selected_block_pos = position;
        if let Some(pos) = position {
            let translation = Mat4::from_translation(pos.as_vec3());
            self.model_data.update_matrix(translation);
        }
    }

    fn get_neighbor_offset(face: BlockFace) -> IVec3 {
        match face {
            BlockFace::PosX => IVec3::X,
            BlockFace::NegX => IVec3::NEG_X,
            BlockFace::PosY => IVec3::Y,
            BlockFace::NegY => IVec3::NEG_Y,
            BlockFace::PosZ => IVec3::Z,
            BlockFace::NegZ => IVec3::NEG_Z,
        }
    }

    pub fn draw<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, queue: &wgpu::Queue, world: &World) {
        if self.current_selected_block_pos.is_none() {
            return; // Nothing selected, nothing to draw
        }

        let selected_pos = self.current_selected_block_pos.unwrap();

        queue.write_buffer(
            &self.model_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.model_data.model_matrix.to_cols_array_2d()]),
        );

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.model_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for (face, index_offset, num_indices_for_face) in &self.face_render_info {
            if *num_indices_for_face == 0 {
                continue;
            }

            let neighbor_offset = Self::get_neighbor_offset(*face);
            let neighbor_pos = selected_pos + neighbor_offset;

            let mut should_draw_face = true;
            if let Some(neighbor_block) = world.get_block_at_world(neighbor_pos.x as f32, neighbor_pos.y as f32, neighbor_pos.z as f32) {
                if neighbor_block.is_solid() { // Assuming Block has is_solid() method
                    should_draw_face = false;
                }
            }
            // If neighbor is outside loaded chunks (get_block_at_world returns None), we might still want to draw the face.
            // Current logic: if neighbor doesn't exist or is air, draw. If solid, don't.

            if should_draw_face {
                let start = *index_offset;
                let end = start + *num_indices_for_face;
                render_pass.draw_indexed(start..end, 0, 0..1);
            }
        }
    }
}
