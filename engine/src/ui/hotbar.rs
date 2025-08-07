// engine/src/ui/hotbar.rs

use super::item::{ItemType, ItemStack};
use crate::block::BlockType;
use wgpu::util::DeviceExt;

const NUM_SLOTS: usize = 9;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HotbarVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl HotbarVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<HotbarVertex>() as wgpu::BufferAddress,
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
            ],
        }
    }
}

pub struct Hotbar {
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    render_pipeline: wgpu::RenderPipeline,
    projection_bind_group: wgpu::BindGroup,
    pub items: [Option<ItemStack>; NUM_SLOTS],
    // Store positions to avoid recalculating them in draw loop
    pub slot_positions: [[f32; 2]; NUM_SLOTS],
}

impl Hotbar {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        const SLOT_SIZE: f32 = 50.0;
        const SLOT_MARGIN: f32 = 5.0;
        const TOTAL_SLOT_SIZE: f32 = SLOT_SIZE + SLOT_MARGIN;

        let mut vertices: Vec<HotbarVertex> = Vec::new();

        let hotbar_width = (NUM_SLOTS as f32 * TOTAL_SLOT_SIZE) + SLOT_MARGIN * 2.0;
        let hotbar_height = TOTAL_SLOT_SIZE + SLOT_MARGIN;
        let hotbar_start_x = (config.width as f32 - hotbar_width) / 2.0;
        let hotbar_start_y = config.height as f32 - hotbar_height - SLOT_MARGIN;
        let bg_color = [0.1, 0.1, 0.1, 0.8];

        vertices.extend_from_slice(&[
            HotbarVertex { position: [hotbar_start_x, hotbar_start_y], color: bg_color },
            HotbarVertex { position: [hotbar_start_x + hotbar_width, hotbar_start_y + hotbar_height], color: bg_color },
            HotbarVertex { position: [hotbar_start_x, hotbar_start_y + hotbar_height], color: bg_color },
            HotbarVertex { position: [hotbar_start_x, hotbar_start_y], color: bg_color },
            HotbarVertex { position: [hotbar_start_x + hotbar_width, hotbar_start_y], color: bg_color },
            HotbarVertex { position: [hotbar_start_x + hotbar_width, hotbar_start_y + hotbar_height], color: bg_color },
        ]);

        let grid_width = NUM_SLOTS as f32 * TOTAL_SLOT_SIZE - SLOT_MARGIN;
        let start_x = (config.width as f32 - grid_width) / 2.0;
        let start_y_slots = config.height as f32 - SLOT_SIZE - (SLOT_MARGIN * 2.0);
        let slot_color = [0.3, 0.3, 0.3, 0.8];
        let mut slot_positions = [[0.0; 2]; NUM_SLOTS];

        for col in 0..NUM_SLOTS {
            let x = start_x + col as f32 * TOTAL_SLOT_SIZE;
            let y = start_y_slots;
            slot_positions[col] = [x + SLOT_SIZE / 2.0, y + SLOT_SIZE / 2.0];
            vertices.extend_from_slice(&[
                HotbarVertex { position: [x, y], color: slot_color },
                HotbarVertex { position: [x + SLOT_SIZE, y + SLOT_SIZE], color: slot_color },
                HotbarVertex { position: [x, y + SLOT_SIZE], color: slot_color },
                HotbarVertex { position: [x, y], color: slot_color },
                HotbarVertex { position: [x + SLOT_SIZE, y], color: slot_color },
                HotbarVertex { position: [x + SLOT_SIZE, y + SLOT_SIZE], color: slot_color },
            ]);
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hotbar Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_vertices = vertices.len() as u32;

        let projection_matrix = glam::Mat4::orthographic_rh(
            0.0,
            config.width as f32,
            config.height as f32,
            0.0,
            -1.0,
            1.0,
        );

        let projection_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hotbar Projection Buffer"),
            contents: bytemuck::cast_slice(projection_matrix.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let projection_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("hotbar_projection_bind_group_layout"),
            });

        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &projection_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
            label: Some("hotbar_projection_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../ui_shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Hotbar Render Pipeline Layout"),
                bind_group_layouts: &[&projection_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hotbar Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[HotbarVertex::desc()],
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let mut items: [Option<ItemStack>; NUM_SLOTS] = Default::default();
        items[0] = Some(ItemStack::new(ItemType::Block(BlockType::Dirt), 1));

        Self {
            vertex_buffer,
            num_vertices,
            render_pipeline,
            projection_bind_group,
            items,
            slot_positions,
        }
    }

    pub fn draw<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.projection_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.num_vertices, 0..1);
    }

    pub fn handle_mouse_click(
        &mut self,
        input: &crate::input::InputState,
        drag_item: &mut Option<ItemStack>,
    ) {
        const SLOT_SIZE: f32 = 50.0;
        let (cursor_x, cursor_y) = input.cursor_position;

        for i in 0..NUM_SLOTS {
            let slot_x = self.slot_positions[i][0] - SLOT_SIZE / 2.0;
            let slot_y = self.slot_positions[i][1] - SLOT_SIZE / 2.0;

            let is_in_slot = cursor_x >= slot_x
                && cursor_x <= slot_x + SLOT_SIZE
                && cursor_y >= slot_y
                && cursor_y <= slot_y + SLOT_SIZE;

            if is_in_slot {
                if input.left_mouse_released_this_frame {
                    let slot_item = self.items[i].take();
                    let taken_drag_item = drag_item.take();

                    if let Some(mut s_item) = slot_item {
                        if let Some(mut d_item) = taken_drag_item {
                            if s_item.item_type == d_item.item_type {
                                let total = s_item.count + d_item.count;
                                s_item.count = total.min(64);
                                d_item.count = total.saturating_sub(64);
                                self.items[i] = Some(s_item);
                                if d_item.count > 0 {
                                    *drag_item = Some(d_item);
                                }
                            } else {
                                self.items[i] = Some(d_item);
                                *drag_item = Some(s_item);
                            }
                        } else {
                            *drag_item = Some(s_item);
                        }
                    } else if let Some(d_item) = taken_drag_item {
                        self.items[i] = Some(d_item);
                    }
                } else if input.right_mouse_released_this_frame {
                    if let Some(d_item) = drag_item.as_mut() {
                        if let Some(s_item) = self.items[i].as_mut() {
                            if s_item.item_type == d_item.item_type {
                                if s_item.count < 64 {
                                    s_item.count += 1;
                                    d_item.count -= 1;
                                    if d_item.count == 0 {
                                        *drag_item = None;
                                    }
                                }
                            }
                        } else {
                            self.items[i] = Some(super::item::ItemStack::new(d_item.item_type, 1));
                            d_item.count -= 1;
                            if d_item.count == 0 {
                                *drag_item = None;
                            }
                        }
                    } else if let Some(s_item) = self.items[i].as_mut() {
                        if s_item.count > 1 {
                            let half = (s_item.count + 1) / 2;
                            let new_stack =
                                super::item::ItemStack::new(s_item.item_type, half);
                            s_item.count -= half;
                            if s_item.count == 0 {
                                self.items[i] = None;
                            }
                            *drag_item = Some(new_stack);
                        }
                    }
                }
                return;
            }
        }
    }
}
