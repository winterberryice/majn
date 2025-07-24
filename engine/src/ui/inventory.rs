// engine/src/ui/inventory.rs

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InventoryVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl InventoryVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InventoryVertex>() as wgpu::BufferAddress,
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

pub struct Inventory {
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub render_pipeline: wgpu::RenderPipeline,
    pub projection_bind_group: wgpu::BindGroup,
}

impl Inventory {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        use wgpu::util::DeviceExt;

        const SLOT_SIZE: f32 = 50.0;
        const SLOT_MARGIN: f32 = 5.0;
        const TOTAL_SLOT_SIZE: f32 = SLOT_SIZE + SLOT_MARGIN;

        const GRID_COLS: i32 = 9;
        const GRID_ROWS: i32 = 4;

        let mut vertices: Vec<InventoryVertex> = Vec::new();

        // Background
        let bg_width = (GRID_COLS as f32 * TOTAL_SLOT_SIZE) + SLOT_MARGIN * 2.0;
        let bg_height = (GRID_ROWS as f32 * TOTAL_SLOT_SIZE) + SLOT_MARGIN * 2.0;
        let bg_start_x = -bg_width / 2.0;
        let bg_start_y = -bg_height / 2.0;
        let bg_color = [0.1, 0.1, 0.1, 0.8];

        vertices.push(InventoryVertex { position: [bg_start_x, bg_start_y], color: bg_color });
        vertices.push(InventoryVertex { position: [bg_start_x + bg_width, bg_start_y], color: bg_color });
        vertices.push(InventoryVertex { position: [bg_start_x, bg_start_y + bg_height], color: bg_color });

        vertices.push(InventoryVertex { position: [bg_start_x + bg_width, bg_start_y], color: bg_color });
        vertices.push(InventoryVertex { position: [bg_start_x + bg_width, bg_start_y + bg_height], color: bg_color });
        vertices.push(InventoryVertex { position: [bg_start_x, bg_start_y + bg_height], color: bg_color });


        // Main inventory grid (4x9)
        let grid_width = GRID_COLS as f32 * TOTAL_SLOT_SIZE - SLOT_MARGIN;
        let grid_height = GRID_ROWS as f32 * TOTAL_SLOT_SIZE - SLOT_MARGIN;
        let start_x = -grid_width / 2.0;
        let start_y = -grid_height / 2.0;
        let slot_color = [0.3, 0.3, 0.3, 0.8];

        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                let x = start_x + col as f32 * TOTAL_SLOT_SIZE;
                let y = start_y + row as f32 * TOTAL_SLOT_SIZE;

                // Create a quad for each slot
                vertices.push(InventoryVertex { position: [x, y], color: slot_color });
                vertices.push(InventoryVertex { position: [x + SLOT_SIZE, y], color: slot_color });
                vertices.push(InventoryVertex { position: [x, y + SLOT_SIZE], color: slot_color });

                vertices.push(InventoryVertex { position: [x + SLOT_SIZE, y], color: slot_color });
                vertices.push(InventoryVertex { position: [x + SLOT_SIZE, y + SLOT_SIZE], color: slot_color });
                vertices.push(InventoryVertex { position: [x, y + SLOT_SIZE], color: slot_color });
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Inventory Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_vertices = vertices.len() as u32;

        let projection_matrix = glam::Mat4::orthographic_rh(
            -(config.width as f32) / 2.0,
            config.width as f32 / 2.0,
            config.height as f32 / 2.0,
            -(config.height as f32) / 2.0,
            -1.0,
            1.0,
        );

        let projection_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Inventory Projection Buffer"),
            contents: bytemuck::cast_slice(projection_matrix.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let projection_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("inventory_projection_bind_group_layout"),
        });

        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &projection_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
            label: Some("inventory_projection_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../ui_shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Inventory Render Pipeline Layout"),
            bind_group_layouts: &[&projection_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Inventory Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[InventoryVertex::desc()],
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

        Self {
            vertex_buffer,
            num_vertices,
            render_pipeline,
            projection_bind_group,
        }
    }

    pub fn draw<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.projection_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.num_vertices, 0..1);
    }
}
