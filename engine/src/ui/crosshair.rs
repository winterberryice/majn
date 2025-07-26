// engine/src/ui/crosshair.rs

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CrosshairVertex {
    position: [f32; 2],
}

impl CrosshairVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CrosshairVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // Corresponds to `layout(location = 0)` in shader
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Crosshair {
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)] // projection_matrix is used for resize, not directly for drawing after buffer creation
    projection_matrix: glam::Mat4,
    pub projection_buffer: wgpu::Buffer,
    #[allow(dead_code)] // layout is stored because it's needed for pipeline creation, not directly for drawing
    projection_bind_group_layout: wgpu::BindGroupLayout,
    pub projection_bind_group: wgpu::BindGroup,
}

impl Crosshair {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        use wgpu::util::DeviceExt;

        // Define vertices for the crosshair (+) shape
        // Each line of the crosshair is a thin rectangle (2 triangles)
        // Dimensions are in screen pixels.
        let half_length = 10.0; // Length of each arm from the center
        let thickness = 1.0;    // Thickness of the arms

        let vertices: &[CrosshairVertex] = &[
            // Horizontal bar
            CrosshairVertex { position: [-half_length, -thickness] }, // Top-left
            CrosshairVertex { position: [ half_length, -thickness] }, // Top-right
            CrosshairVertex { position: [-half_length,  thickness] }, // Bottom-left

            CrosshairVertex { position: [ half_length, -thickness] }, // Top-right
            CrosshairVertex { position: [ half_length,  thickness] }, // Bottom-right
            CrosshairVertex { position: [-half_length,  thickness] }, // Bottom-left

            // Vertical bar
            CrosshairVertex { position: [-thickness, -half_length] }, // Top-left
            CrosshairVertex { position: [ thickness, -half_length] }, // Top-right
            CrosshairVertex { position: [-thickness,  half_length] }, // Bottom-left

            CrosshairVertex { position: [ thickness, -half_length] }, // Top-right
            CrosshairVertex { position: [ thickness,  half_length] }, // Bottom-right
            CrosshairVertex { position: [-thickness,  half_length] }, // Bottom-left
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Crosshair Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_vertices = vertices.len() as u32;

        // Orthographic projection matrix
        // Adjusted to center coordinates: (0,0) in UI space will be screen center.
        let projection_matrix = glam::Mat4::orthographic_rh(
            -(config.width as f32) / 2.0, // left
            config.width as f32 / 2.0,  // right
            config.height as f32 / 2.0, // bottom (maps to NDC -1, bottom of screen)
            -(config.height as f32) / 2.0, // top (maps to NDC +1, top of screen)
            -1.0,                       // znear
            1.0,                        // zfar
        );

        let projection_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Crosshair Projection Buffer"),
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
            label: Some("crosshair_projection_bind_group_layout"),
        });

        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &projection_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
            label: Some("crosshair_projection_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Crosshair Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../crosshair_shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Crosshair Render Pipeline Layout"),
            bind_group_layouts: &[&projection_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Crosshair Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[CrosshairVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format, // Use the surface/swapchain format
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for 2D UI
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // No depth/stencil testing for the crosshair
            multisample: wgpu::MultisampleState::default(), // No MSAA for UI
            multiview: None,
            cache: None,
        });

        Self {
            vertex_buffer,
            num_vertices,
            render_pipeline,
            projection_matrix,
            projection_buffer,
            projection_bind_group_layout,
            projection_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, queue: &wgpu::Queue) {
        if new_size.width > 0 && new_size.height > 0 {
            self.projection_matrix = glam::Mat4::orthographic_rh(
                -(new_size.width as f32) / 2.0, // left
                new_size.width as f32 / 2.0,  // right
                new_size.height as f32 / 2.0, // bottom
                -(new_size.height as f32) / 2.0, // top
                -1.0,                         // znear
                1.0,                          // zfar
            );
            queue.write_buffer(
                &self.projection_buffer,
                0,
                bytemuck::cast_slice(self.projection_matrix.as_ref()),
            );
        }
    }

    pub fn draw<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.projection_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.num_vertices, 0..1);
    }
}
