// engine/src/ui/item_renderer.rs

use super::item::{ItemType, ItemStack};
use crate::block::Block;

// This vertex is now specific to the item renderer
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl UIVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const ATLAS_WIDTH_IN_BLOCKS: f32 = 16.0;
const ATLAS_HEIGHT_IN_BLOCKS: f32 = 1.0;

pub struct ItemRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    buffer_size: u64,
}

impl ItemRenderer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        projection_bind_group_layout: &wgpu::BindGroupLayout,
        ui_texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Item Renderer Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../item_ui_shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Item Renderer Pipeline Layout"),
                bind_group_layouts: &[projection_bind_group_layout, ui_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Item Renderer Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[UIVertex::desc()],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let buffer_size = 1024 * 4;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Item Renderer Vertex Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            buffer_size,
        }
    }

    pub fn draw<'pass>(
        &'pass mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'pass>,
        projection_bind_group: &'pass wgpu::BindGroup,
        ui_texture_bind_group: &'pass wgpu::BindGroup,
        items: &[(ItemStack, [f32; 2], f32)],
    ) {
        let mut vertices: Vec<UIVertex> = Vec::new();
        for (item_stack, position, size) in items {
            generate_item_vertices(item_stack.item_type, *position, *size, &mut vertices);
        }

        if vertices.is_empty() {
            return;
        }

        let vertex_data = bytemuck::cast_slice(&vertices);
        let required_size = vertex_data.len() as u64;

        if required_size > self.buffer_size {
            self.buffer_size = required_size.next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Item Renderer Vertex Buffer (Resized)"),
                size: self.buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.vertex_buffer, 0, vertex_data);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, projection_bind_group, &[]);
        render_pass.set_bind_group(1, ui_texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..required_size));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }
}

fn generate_item_vertices(
    item_type: ItemType,
    position: [f32; 2],
    size: f32,
    vertices: &mut Vec<UIVertex>,
) {
    let block_type = match item_type {
        ItemType::Block(bt) => bt,
    };

    let temp_block = Block::new(block_type);
    let indices = temp_block.get_texture_atlas_indices();
    let (uv_top, uv_side, uv_front) = (indices[4], indices[0], indices[2]);

    let texel_width = 1.0 / ATLAS_WIDTH_IN_BLOCKS;
    let texel_height = 1.0 / ATLAS_HEIGHT_IN_BLOCKS;

    let (u_top, v_top) = (uv_top[0] * texel_width, uv_top[1] * texel_height);
    let (u_side, v_side) = (uv_side[0] * texel_width, uv_side[1] * texel_height);
    let (u_front, v_front) = (uv_front[0] * texel_width, uv_front[1] * texel_height);

    let top_face_uv = [[u_top, v_top], [u_top + texel_width, v_top], [u_top + texel_width, v_top + texel_height], [u_top, v_top + texel_height]];
    let side_face_uv = [[u_side, v_side], [u_side + texel_width, v_side], [u_side + texel_width, v_side + texel_height], [u_side, v_side + texel_height]];
    let front_face_uv = [[u_front, v_front], [u_front + texel_width, v_front], [u_front + texel_width, v_front + texel_height], [u_front, v_front + texel_height]];

    let color = [1.0, 1.0, 1.0, 1.0];
    let s = size / 2.0;
    let y_squish = 0.5;

    let position = [position[0], position[1] + size / 4.0];

    let p1 = [position[0], position[1] + s * y_squish];
    let p2 = [position[0] + s, position[1]];
    let p3 = [position[0], position[1] - s * y_squish];
    let p4 = [position[0] - s, position[1]];
    add_quad(vertices, [p1, p2, p3, p4], top_face_uv, color);

    let p5 = [p4[0], p4[1]];
    let p6 = [p3[0], p3[1]];
    let p7 = [p3[0], p3[1] - s];
    let p8 = [p4[0], p4[1] - s];
    add_quad(vertices, [p5, p6, p7, p8], side_face_uv, color);

    let p9 = [p3[0], p3[1]];
    let p10 = [p2[0], p2[1]];
    let p11 = [p2[0], p2[1] - s];
    let p12 = [p3[0], p3[1] - s];
    add_quad(vertices, [p9, p10, p11, p12], front_face_uv, color);
}

fn add_quad(
    vertices: &mut Vec<UIVertex>,
    points: [[f32; 2]; 4],
    uvs: [[f32; 2]; 4],
    color: [f32; 4],
) {
    let v = [
        UIVertex { position: points[0], color, tex_coords: uvs[0] },
        UIVertex { position: points[3], color, tex_coords: uvs[3] },
        UIVertex { position: points[1], color, tex_coords: uvs[1] },
        UIVertex { position: points[1], color, tex_coords: uvs[1] },
        UIVertex { position: points[3], color, tex_coords: uvs[3] },
        UIVertex { position: points[2], color, tex_coords: uvs[2] },
    ];
    vertices.extend_from_slice(&v);
}
