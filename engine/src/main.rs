// Content of main.rs from last read_files, with is_face_visible logic updated
// AND with the erroneous "[end of engine/src/main.rs]" lines removed.

mod block;
mod camera;
mod chunk;
mod cube_geometry;
mod debug_overlay;
mod input;
pub mod physics;
pub mod player;
mod raycast;
mod texture;
mod ui;
mod wireframe_renderer;
mod world;

use std::sync::Arc;
use wgpu::Trace;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
    mouse_grabbed: bool,
    last_mouse_position: Option<winit::dpi::PhysicalPosition<f64>>,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            state: None,
            mouse_grabbed: false,
            last_mouse_position: None,
        }
    }

    fn set_mouse_grab(&mut self, grab: bool) {
        if let Some(window) = self.window.as_ref() {
            if grab {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                    .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
                    .unwrap_or_else(|e| eprintln!("Failed to grab cursor: {:?}", e));
                window.set_cursor_visible(false);
            } else {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::None)
                    .unwrap_or_else(|e| eprintln!("Failed to release cursor: {:?}", e));
                window.set_cursor_visible(true);
            }
            self.mouse_grabbed = grab;
        }
    }

    fn handle_window_event(&mut self, event: WindowEvent, active_loop: &ActiveEventLoop) {
        let mut event_consumed_by_grab_logic = false;
        match event {
            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } if key_event.physical_key == PhysicalKey::Code(KeyCode::Escape)
                && key_event.state == ElementState::Pressed =>
            {
                if self.mouse_grabbed {
                    self.set_mouse_grab(false);
                    event_consumed_by_grab_logic = true;
                } else {
                    active_loop.exit();
                    return;
                }
            }
            WindowEvent::MouseInput {
                button,
                state: mouse_element_state,
                ..
            } => {
                if let Some(s) = self.state.as_mut() {
                    s.input_state.on_mouse_input(button, mouse_element_state);
                }
                if mouse_element_state == ElementState::Pressed {
                    if !self.mouse_grabbed {
                        self.set_mouse_grab(true);
                    }
                }
            }
            _ => {}
        }

        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        let mut event_handled_by_state_input = false;
        if !(event_consumed_by_grab_logic
            && matches!(
                event,
                WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                }
            ))
        {
            event_handled_by_state_input = state.input(&event);
        }

        let mut cursor_moved_while_grabbed = false;
        if self.mouse_grabbed {
            if let WindowEvent::CursorMoved { position, .. } = event {
                let mut mouse_delta = (0.0, 0.0);
                if let Some(last_pos) = self.last_mouse_position {
                    mouse_delta.0 = position.x - last_pos.x;
                    mouse_delta.1 = position.y - last_pos.y;
                }
                self.last_mouse_position = Some(position);
                cursor_moved_while_grabbed = true;
            }
        }

        if !event_consumed_by_grab_logic
            && !event_handled_by_state_input
            && !cursor_moved_while_grabbed
        {
            match event {
                WindowEvent::CloseRequested => {
                    active_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => active_loop.exit(),
                        Err(e) => eprintln!("Error rendering: {:?}", e),
                    }
                }
                _ => {}
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        if self.window.is_none() {
            let window_attributes =
                Window::default_attributes().with_title("Hello WGPU with ApplicationHandler!");
            let window_arc = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(Arc::clone(&window_arc));
            let initial_size = window_arc.inner_size();
            let state_val = pollster::block_on(State::new(Arc::clone(&window_arc), initial_size));
            self.state = Some(state_val);
            self.set_mouse_grab(true);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = self.window.as_ref() {
            if window.id() == window_id {
                self.handle_window_event(event, event_loop);
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_grabbed {
                    if let Some(ref mut state_obj) = self.state {
                        state_obj.process_mouse_motion(delta.0, delta.1);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        println!("ApplicationHandler: Event loop is exiting. Cleaning up.");
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
    pub tree_id: u32,
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2 + std::mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

use crate::block::BlockType;
use crate::camera::CameraUniform;
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::cube_geometry::CubeFace;
use crate::debug_overlay::DebugOverlay;
use crate::physics::AABB;
use crate::physics::PLAYER_EYE_HEIGHT;
use crate::player::Player;
use crate::raycast::BlockFace;
use crate::wireframe_renderer::WireframeRenderer;
use crate::world::World;
use glam::IVec3;
use glam::Mat4;
use std::collections::HashMap;

struct ChunkRenderBuffers {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

struct ChunkRenderData {
    opaque_buffers: Option<ChunkRenderBuffers>,
    transparent_buffers: Option<ChunkRenderBuffers>,
    last_transparent_mesh_camera_pos: Option<glam::Vec3>,
    last_transparent_mesh_camera_yaw: Option<f32>,
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    transparent_render_pipeline: wgpu::RenderPipeline,
    player: Player,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    world: World,
    chunk_render_data: HashMap<(i32, i32), ChunkRenderData>,
    active_chunk_coords: Vec<(i32, i32)>,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    debug_overlay: DebugOverlay,
    crosshair: ui::crosshair::Crosshair,
    wireframe_renderer: WireframeRenderer,
    selected_block: Option<(IVec3, BlockFace)>,
    diffuse_bind_group: wgpu::BindGroup,
    input_state: input::InputState,
}

// Constants for threshold-based re-meshing of transparent chunks
const TRANSPARENT_REMESH_DISTANCE_SQUARED_THRESHOLD: f32 = 9.0;
const TRANSPARENT_REMESH_YAW_THRESHOLD_RADIANS: f32 = 0.349066;

impl State {
    async fn new(
        window_surface_target: Arc<Window>,
        initial_size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window_surface_target).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: wgpu::MemoryHints::default(),
                    trace: Trace::Off,
                },
                // None, // trace_path argument removed
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: initial_size.width,
            height: initial_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        const TERRAIN_ATLAS_BYTES: &[u8] = include_bytes!("../assets/resources/terrain.png");

        let diffuse_texture = match crate::texture::Texture::load_from_memory(
            &device,
            &queue,
            TERRAIN_ATLAS_BYTES,
            "terrain_atlas_from_memory",
        ) {
            Ok(tex) => tex,
            Err(e) => {
                eprintln!(
                    "Failed to load embedded terrain.png from memory: {}. Using placeholder.",
                    e
                );
                crate::texture::Texture::create_placeholder(
                    &device,
                    &queue,
                    Some("Placeholder Terrain"),
                )
            }
        };

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera_bind_group_layout =
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
                label: Some("camera_bind_group_layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let transparent_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Transparent Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
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
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let initial_player_position = glam::Vec3::new(
            CHUNK_WIDTH as f32 / 2.0,
            (CHUNK_HEIGHT / 2) as f32 + 2.0,
            CHUNK_DEPTH as f32 / 2.0,
        );
        let initial_yaw = -std::f32::consts::FRAC_PI_2;
        let initial_pitch = 0.0;
        let mouse_sensitivity = 0.003;

        let player = Player::new(
            initial_player_position,
            initial_yaw,
            initial_pitch,
            mouse_sensitivity,
        );

        let camera_uniform = CameraUniform::new();

        use wgpu::util::DeviceExt;
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let world = World::new();

        let depth_texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Depth Texture"),
            view_formats: &[],
        };
        let depth_texture = device.create_texture(&depth_texture_desc);
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let debug_overlay = DebugOverlay::new(&device, &config);
        let crosshair = ui::crosshair::Crosshair::new(&device, &config);
        let wireframe_renderer =
            WireframeRenderer::new(&device, &config, &camera_bind_group_layout);

        Self {
            surface,
            device,
            queue,
            config,
            size: initial_size,
            render_pipeline,
            transparent_render_pipeline,
            player,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            world,
            chunk_render_data: HashMap::new(),
            active_chunk_coords: Vec::new(),
            depth_texture,
            depth_texture_view,
            debug_overlay,
            wireframe_renderer,
            selected_block: None,
            crosshair,
            diffuse_bind_group,
            input_state: input::InputState::new(),
        }
    }

    fn build_or_rebuild_chunk_mesh(&mut self, chunk_cx: i32, chunk_cz: i32) {
        let mut opaque_vertices: Vec<Vertex> = Vec::new();
        let mut opaque_indices: Vec<u16> = Vec::new();
        let mut opaque_vertex_offset: u16 = 0;

        let mut transparent_vertices: Vec<Vertex> = Vec::new();
        let mut transparent_indices: Vec<u16> = Vec::new();
        let mut transparent_vertex_offset: u16 = 0;

        struct TransparentBlockData {
            block: block::Block,
            lx: usize,
            ly: usize,
            lz: usize,
            world_center: glam::Vec3,
        }
        let mut transparent_block_render_list: Vec<TransparentBlockData> = Vec::new();

        let chunk_opt = self.world.get_chunk(chunk_cx, chunk_cz);
        if chunk_opt.is_none() {
            eprintln!(
                "Attempted to build mesh for non-existent or non-generated chunk ({}, {})",
                chunk_cx, chunk_cz
            );
            self.chunk_render_data.remove(&(chunk_cx, chunk_cz));
            return;
        }
        let chunk = chunk_opt.unwrap();

        let chunk_world_origin_x = chunk_cx as f32 * CHUNK_WIDTH as f32;
        let chunk_world_origin_z = chunk_cz as f32 * CHUNK_DEPTH as f32;

        for lx in 0..CHUNK_WIDTH {
            for ly in 0..CHUNK_HEIGHT {
                for lz in 0..CHUNK_DEPTH {
                    if let Some(block) = chunk.get_block(lx, ly, lz) {
                        if block.block_type == BlockType::Air {
                            continue;
                        }
                        let is_current_block_transparent = block.is_transparent();
                        let default_block_color = match block.block_type {
                            BlockType::Dirt => [0.5, 0.25, 0.05],
                            BlockType::Grass => [0.0, 0.8, 0.1],
                            BlockType::Bedrock => [0.5, 0.5, 0.5],
                            BlockType::OakLog => [0.5, 0.5, 0.5],
                            BlockType::OakLeaves => [0.5, 0.5, 0.5],
                            BlockType::Air => unreachable!(),
                        };
                        let current_block_world_center = glam::Vec3::new(
                            chunk_world_origin_x + lx as f32 + 0.5,
                            ly as f32 + 0.5,
                            chunk_world_origin_z + lz as f32 + 0.5,
                        );
                        let face_definitions: [(CubeFace, (i32, i32, i32)); 6] = [
                            (CubeFace::Front, (0, 0, -1)), (CubeFace::Back, (0, 0, 1)),
                            (CubeFace::Right, (1, 0, 0)), (CubeFace::Left, (-1, 0, 0)),
                            (CubeFace::Top, (0, 1, 0)), (CubeFace::Bottom, (0, -1, 0)),
                        ];
                        for (face_type, offset) in face_definitions.iter() {
                            let neighbor_world_bx =
                                chunk_world_origin_x as i32 + lx as i32 + offset.0;
                            let neighbor_world_by = ly as i32 + offset.1;
                            let neighbor_world_bz =
                                chunk_world_origin_z as i32 + lz as i32 + offset.2;

                            let mut is_face_visible = true;
                            if neighbor_world_by >= 0 && neighbor_world_by < CHUNK_HEIGHT as i32 {
                                if let Some(neighbor_block) = self.world.get_block_at_world(
                                    neighbor_world_bx as f32,
                                    neighbor_world_by as f32,
                                    neighbor_world_bz as f32,
                                ) {
                                    if neighbor_block.is_solid() && !neighbor_block.is_transparent() {
                                        is_face_visible = false;
                                    }
                                }
                            }

                            if is_face_visible {
                                if !is_current_block_transparent {
                                    let vertices_template = face_type.get_vertices_template();
                                    let local_indices = face_type.get_local_indices();
                                    const ATLAS_COLS: f32 = 16.0;
                                    const ATLAS_ROWS: f32 = 39.0;
                                    let tex_size_x = 1.0 / ATLAS_COLS;
                                    let tex_size_y = 1.0 / ATLAS_ROWS;
                                    let all_face_atlas_indices = block.get_texture_atlas_indices();
                                    let mut current_vertex_color = default_block_color;
                                    let face_specific_atlas_indices: [f32; 2] = match face_type {
                                        CubeFace::Front => all_face_atlas_indices[0],
                                        CubeFace::Back => all_face_atlas_indices[1],
                                        CubeFace::Right => all_face_atlas_indices[2],
                                        CubeFace::Left => all_face_atlas_indices[3],
                                        CubeFace::Top => all_face_atlas_indices[4],
                                        CubeFace::Bottom => all_face_atlas_indices[5],
                                    };
                                    match block.block_type {
                                        BlockType::Grass => {
                                            if *face_type == CubeFace::Top { current_vertex_color = [0.1, 0.9, 0.1]; }
                                            else if *face_type == CubeFace::Bottom { current_vertex_color = [0.5, 0.25, 0.05];}
                                            else { current_vertex_color = [0.0, 0.8, 0.1]; }
                                        }
                                        _ => {}
                                    }
                                    let u_min = face_specific_atlas_indices[0] * tex_size_x;
                                    let v_min = face_specific_atlas_indices[1] * tex_size_y;
                                    let u_max = u_min + tex_size_x;
                                    let v_max = v_min + tex_size_y;
                                    let uvs_for_bl_br_tr_tl_order = [[u_min, v_max], [u_max, v_max], [u_max, v_min], [u_min, v_min]];
                                    let uvs_for_bl_tl_tr_br_order = [[u_min, v_max], [u_min, v_min], [u_max, v_min], [u_max, v_max]];
                                    let selected_face_uvs = match face_type {
                                        CubeFace::Front | CubeFace::Right | CubeFace::Left | CubeFace::Bottom => &uvs_for_bl_tl_tr_br_order,
                                        CubeFace::Back | CubeFace::Top => &uvs_for_bl_br_tr_tl_order,
                                    };
                                    for (i, v_template) in vertices_template.iter().enumerate() {
                                        opaque_vertices.push(Vertex {
                                            position: (current_block_world_center + glam::Vec3::from(v_template.position)).into(),
                                            color: current_vertex_color,
                                            uv: selected_face_uvs[i],
                                            tree_id: 0,
                                        });
                                    }
                                    for local_idx in local_indices {
                                        opaque_indices.push(opaque_vertex_offset + local_idx);
                                    }
                                    opaque_vertex_offset += vertices_template.len() as u16;
                                }
                            }
                        }
                        if is_current_block_transparent {
                            transparent_block_render_list.push(TransparentBlockData {
                                block: *block,
                                lx, ly, lz,
                                world_center: current_block_world_center,
                            });
                        }
                    }
                }
            }
        }

        let player_camera_pos = self.player.position + glam::Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0);
        transparent_block_render_list.sort_by(|a, b| {
            let dist_a = player_camera_pos.distance_squared(a.world_center);
            let dist_b = player_camera_pos.distance_squared(b.world_center);
            dist_b.partial_cmp(&dist_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        for t_block_data in transparent_block_render_list {
            let block = &t_block_data.block;
            let _lx = t_block_data.lx;
            let _ly = t_block_data.ly;
            let _lz = t_block_data.lz;
            let current_block_world_center = t_block_data.world_center;
            let base_vertex_color = match block.block_type {
                BlockType::OakLeaves => [0.1, 0.9, 0.2],
                _ => [0.5, 0.5, 0.5],
            };
            let face_definitions: [(CubeFace, (i32, i32, i32)); 6] = [
                (CubeFace::Front, (0, 0, -1)), (CubeFace::Back, (0, 0, 1)),
                (CubeFace::Right, (1, 0, 0)), (CubeFace::Left, (-1, 0, 0)),
                (CubeFace::Top, (0, 1, 0)), (CubeFace::Bottom, (0, -1, 0)),
            ];
            for (face_type, _offset) in face_definitions.iter() {
                let mut is_face_visible_for_transparent = true;
                let neighbor_check_offset = match face_type {
                    CubeFace::Front => (0,0,-1), CubeFace::Back => (0,0,1),
                    CubeFace::Right => (1,0,0), CubeFace::Left => (-1,0,0),
                    CubeFace::Top => (0,1,0), CubeFace::Bottom => (0,-1,0),
                };
                let neighbor_world_bx_transparent = (chunk_world_origin_x + t_block_data.lx as f32) as i32 + neighbor_check_offset.0;
                let neighbor_world_by_transparent = t_block_data.ly as i32 + neighbor_check_offset.1;
                let neighbor_world_bz_transparent = (chunk_world_origin_z + t_block_data.lz as f32) as i32 + neighbor_check_offset.2;

                if neighbor_world_by_transparent >= 0 && neighbor_world_by_transparent < CHUNK_HEIGHT as i32 {
                    if let Some(neighbor_block_transparent) = self.world.get_block_at_world(
                        neighbor_world_bx_transparent as f32,
                        neighbor_world_by_transparent as f32,
                        neighbor_world_bz_transparent as f32
                    ) {
                        if neighbor_block_transparent.is_solid() && !neighbor_block_transparent.is_transparent() {
                            is_face_visible_for_transparent = false;
                        }
                    }
                }

                if !is_face_visible_for_transparent {
                    continue;
                }

                let vertices_template = face_type.get_vertices_template();
                let local_indices = face_type.get_local_indices();
                const ATLAS_COLS: f32 = 16.0;
                const ATLAS_ROWS: f32 = 39.0;
                let tex_size_x = 1.0 / ATLAS_COLS;
                let tex_size_y = 1.0 / ATLAS_ROWS;
                let all_face_atlas_indices = block.get_texture_atlas_indices();
                let current_vertex_color = base_vertex_color;
                let face_specific_atlas_indices: [f32; 2] = match face_type {
                    CubeFace::Front => all_face_atlas_indices[0], CubeFace::Back => all_face_atlas_indices[1],
                    CubeFace::Right => all_face_atlas_indices[2], CubeFace::Left => all_face_atlas_indices[3],
                    CubeFace::Top => all_face_atlas_indices[4], CubeFace::Bottom => all_face_atlas_indices[5],
                };
                let u_min = face_specific_atlas_indices[0] * tex_size_x;
                let v_min = face_specific_atlas_indices[1] * tex_size_y;
                let u_max = u_min + tex_size_x;
                let v_max = v_min + tex_size_y;
                let uvs_for_bl_br_tr_tl_order = [[u_min, v_max], [u_max, v_max], [u_max, v_min], [u_min, v_min]];
                let uvs_for_bl_tl_tr_br_order = [[u_min, v_max], [u_min, v_min], [u_max, v_min], [u_max, v_max]];
                let selected_face_uvs = match face_type {
                    CubeFace::Front | CubeFace::Right | CubeFace::Left | CubeFace::Bottom => &uvs_for_bl_tl_tr_br_order,
                    CubeFace::Back | CubeFace::Top => &uvs_for_bl_br_tr_tl_order,
                };
                for (i, v_template) in vertices_template.iter().enumerate() {
                    let current_tree_id = if block.block_type == BlockType::OakLeaves {
                        block.tree_id.unwrap_or(0)
                    } else { 0 };
                    transparent_vertices.push(Vertex {
                        position: (current_block_world_center + glam::Vec3::from(v_template.position)).into(),
                        color: current_vertex_color,
                        uv: selected_face_uvs[i],
                        tree_id: current_tree_id,
                    });
                }
                for local_idx in local_indices {
                    transparent_indices.push(transparent_vertex_offset + local_idx);
                }
                transparent_vertex_offset += vertices_template.len() as u16;
            }
        }

        use wgpu::util::DeviceExt;
        let mut opaque_buffers: Option<ChunkRenderBuffers> = None;
        if !opaque_vertices.is_empty() && !opaque_indices.is_empty() {
            let vertex_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Opaque Chunk VB ({}, {})", chunk_cx, chunk_cz)),
                        contents: bytemuck::cast_slice(&opaque_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            let index_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Opaque Chunk IB ({}, {})", chunk_cx, chunk_cz)),
                        contents: bytemuck::cast_slice(&opaque_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });
            opaque_buffers = Some(ChunkRenderBuffers {
                vertex_buffer,
                index_buffer,
                num_indices: opaque_indices.len() as u32,
            });
        }
        let mut transparent_buffers: Option<ChunkRenderBuffers> = None;
        if !transparent_vertices.is_empty() && !transparent_indices.is_empty() {
            let vertex_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Transparent Chunk VB ({}, {})", chunk_cx, chunk_cz)),
                        contents: bytemuck::cast_slice(&transparent_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            let index_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Transparent Chunk IB ({}, {})", chunk_cx, chunk_cz)),
                        contents: bytemuck::cast_slice(&transparent_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });
            transparent_buffers = Some(ChunkRenderBuffers {
                vertex_buffer,
                index_buffer,
                num_indices: transparent_indices.len() as u32,
            });
        }
        if opaque_buffers.is_some() || transparent_buffers.is_some() {
            let (final_camera_pos, final_camera_yaw) = if transparent_buffers.is_some() {
                (Some(self.player.position), Some(self.player.yaw))
            } else {
                (None, None)
            };

            self.chunk_render_data.insert(
                (chunk_cx, chunk_cz),
                ChunkRenderData {
                    opaque_buffers,
                    transparent_buffers,
                    last_transparent_mesh_camera_pos: final_camera_pos,
                    last_transparent_mesh_camera_yaw: final_camera_yaw,
                },
            );
        } else {
            self.chunk_render_data.remove(&(chunk_cx, chunk_cz));
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            let depth_texture_desc = wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("Depth Texture (Resized)"),
                view_formats: &[],
            };
            self.depth_texture = self.device.create_texture(&depth_texture_desc);
            self.depth_texture_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.surface.configure(&self.device, &self.config);
            self.debug_overlay
                .resize(new_size.width, new_size.height, &self.queue);
            self.crosshair.resize(new_size, &self.queue);
        }
    }

    pub fn process_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.player.process_mouse_movement(delta_x, delta_y);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match key_code {
                    KeyCode::KeyW | KeyCode::ArrowUp => { self.player.movement_intention.forward = is_pressed; true }
                    KeyCode::KeyS | KeyCode::ArrowDown => { self.player.movement_intention.backward = is_pressed; true }
                    KeyCode::KeyA | KeyCode::ArrowLeft => { self.player.movement_intention.left = is_pressed; true }
                    KeyCode::KeyD | KeyCode::ArrowRight => { self.player.movement_intention.right = is_pressed; true }
                    KeyCode::Space => { self.player.movement_intention.jump = is_pressed; true }
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => false,
                    KeyCode::Escape => false,
                    KeyCode::F3 => { if is_pressed { self.debug_overlay.toggle_visibility(); } true }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        self.handle_block_interactions();
        let dt_secs = 1.0 / 60.0;
        let player_pos = self.player.position;
        let current_chunk_x = (player_pos.x / CHUNK_WIDTH as f32).floor() as i32;
        let current_chunk_z = (player_pos.z / CHUNK_DEPTH as f32).floor() as i32;
        let mut new_active_chunk_coords = Vec::new();
        let render_distance = 1;
        for dx in -render_distance..=render_distance {
            for dz in -render_distance..=render_distance {
                let target_cx = current_chunk_x + dx;
                let target_cz = current_chunk_z + dz;
                new_active_chunk_coords.push((target_cx, target_cz));
                let _ = self.world.get_or_create_chunk(target_cx, target_cz);
            }
        }
        self.active_chunk_coords = new_active_chunk_coords;

        let mut coords_to_mesh: Vec<(i32, i32)> = Vec::new();
        for &(cx, cz) in &self.active_chunk_coords {
            if !self.chunk_render_data.contains_key(&(cx, cz)) {
                coords_to_mesh.push((cx, cz));
            } else {
                if let Some(render_data) = self.chunk_render_data.get(&(cx, cz)) {
                    if render_data.transparent_buffers.is_some() {
                        let mut needs_remesh = false;
                        if let Some(last_pos) = render_data.last_transparent_mesh_camera_pos {
                            if self.player.position.distance_squared(last_pos) > TRANSPARENT_REMESH_DISTANCE_SQUARED_THRESHOLD {
                                needs_remesh = true;
                            }
                        } else {
                            needs_remesh = true;
                        }
                        if !needs_remesh {
                            if let Some(last_yaw) = render_data.last_transparent_mesh_camera_yaw {
                                let yaw_diff = (self.player.yaw - last_yaw).abs();
                                let mut normalized_yaw_diff = yaw_diff;
                                if normalized_yaw_diff > std::f32::consts::PI {
                                    normalized_yaw_diff = 2.0 * std::f32::consts::PI - normalized_yaw_diff;
                                }
                                if normalized_yaw_diff > TRANSPARENT_REMESH_YAW_THRESHOLD_RADIANS {
                                    needs_remesh = true;
                                }
                            } else {
                                needs_remesh = true;
                            }
                        }
                        if needs_remesh {
                            coords_to_mesh.push((cx, cz));
                        }
                    }
                }
            }
        }
        coords_to_mesh.sort_unstable();
        coords_to_mesh.dedup();
        for (cx, cz) in &coords_to_mesh {
            self.build_or_rebuild_chunk_mesh(*cx, *cz);
        }

        self.player.update_physics_and_collision(dt_secs, &self.world);
        const RAYCAST_MAX_DISTANCE: f32 = 5.0;
        self.selected_block =
            crate::raycast::cast_ray(&self.player, &self.world, RAYCAST_MAX_DISTANCE);
        if let Some((block_pos, _)) = self.selected_block {
            self.wireframe_renderer.update_selection(Some(block_pos));
        } else {
            self.wireframe_renderer.update_selection(None);
        }
        let camera_eye = self.player.position + glam::Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0);
        let camera_front = glam::Vec3::new(
            self.player.yaw.cos() * self.player.pitch.cos(),
            self.player.pitch.sin(),
            self.player.yaw.sin() * self.player.pitch.cos(),
        ).normalize();
        let camera_target = camera_eye + camera_front;
        let view_matrix = Mat4::look_at_rh(camera_eye, camera_target, glam::Vec3::Y);
        let aspect_ratio = self.config.width as f32 / self.config.height as f32;
        let fovy_radians = 45.0f32.to_radians();
        let znear = 0.1;
        let zfar = 1000.0;
        let projection_matrix = Mat4::perspective_rh(fovy_radians, aspect_ratio, znear, zfar);
        let view_proj_matrix = projection_matrix * view_matrix;
        self.camera_uniform.view_proj = view_proj_matrix.to_cols_array_2d();
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.debug_overlay.update(self.player.position);
        self.input_state.clear_frame_state();
    }

    fn handle_block_interactions(&mut self) {
        if self.input_state.left_mouse_pressed_this_frame {
            if let Some((block_pos, _face)) = self.selected_block {
                match self.world.set_block(block_pos, BlockType::Air) {
                    Ok(chunk_coord) => {
                        self.chunk_render_data.remove(&chunk_coord);
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1);
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0 + 1, chunk_coord.1);
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0 - 1, chunk_coord.1);
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1 + 1);
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1 - 1);
                    }
                    Err(e) => { eprintln!("Error removing block: {}", e); }
                }
            }
        }
        if self.input_state.right_mouse_pressed_this_frame {
            if let Some((selected_block_pos, hit_face)) = self.selected_block {
                let mut offset = IVec3::ZERO;
                match hit_face {
                    BlockFace::PosX => offset.x = 1, BlockFace::NegX => offset.x = -1,
                    BlockFace::PosY => offset.y = 1, BlockFace::NegY => offset.y = -1,
                    BlockFace::PosZ => offset.z = 1, BlockFace::NegZ => offset.z = -1,
                }
                let new_block_pos = selected_block_pos + offset;
                let player_aabb = self.player.get_world_bounding_box();
                let new_block_aabb = AABB {
                    min: new_block_pos.as_vec3(),
                    max: new_block_pos.as_vec3() + glam::Vec3::ONE,
                };
                if !player_aabb.intersects(&new_block_aabb) {
                    match self.world.set_block(new_block_pos, BlockType::Grass) {
                        Ok(chunk_coord) => {
                            self.chunk_render_data.remove(&chunk_coord);
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1);
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0 + 1, chunk_coord.1);
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0 - 1, chunk_coord.1);
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1 + 1);
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1 - 1);
                        }
                        Err(e) => { eprintln!("Error placing block: {}", e); }
                    }
                }
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let Err(e) = self.debug_overlay.prepare(&self.device, &self.queue) {
            eprintln!("Failed to prepare debug overlay: {:?}", e);
        }
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            for chunk_coord in &self.active_chunk_coords {
                if let Some(chunk_data) = self.chunk_render_data.get(chunk_coord) {
                    if let Some(ref opaque_buffers) = chunk_data.opaque_buffers {
                        if opaque_buffers.num_indices > 0 {
                            render_pass.set_vertex_buffer(0, opaque_buffers.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(opaque_buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                            render_pass.draw_indexed(0..opaque_buffers.num_indices, 0, 0..1);
                        }
                    }
                }
            }
            self.wireframe_renderer.draw(&mut render_pass, &self.queue, &self.world);
            render_pass.set_pipeline(&self.transparent_render_pipeline);
            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            let mut sorted_transparent_chunks = self.active_chunk_coords.clone();
            let player_pos = self.player.position;
            sorted_transparent_chunks.sort_by(|a, b| {
                let pos_a = glam::Vec3::new(
                    (a.0 as f32 + 0.5) * CHUNK_WIDTH as f32,
                    CHUNK_HEIGHT as f32 / 2.0,
                    (a.1 as f32 + 0.5) * CHUNK_DEPTH as f32,
                );
                let pos_b = glam::Vec3::new(
                    (b.0 as f32 + 0.5) * CHUNK_WIDTH as f32,
                    CHUNK_HEIGHT as f32 / 2.0,
                    (b.1 as f32 + 0.5) * CHUNK_DEPTH as f32,
                );
                let dist_a = player_pos.distance_squared(pos_a);
                let dist_b = player_pos.distance_squared(pos_b);
                dist_b.partial_cmp(&dist_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            for chunk_coord in &sorted_transparent_chunks {
                if let Some(chunk_data) = self.chunk_render_data.get(chunk_coord) {
                    if let Some(ref transparent_buffers) = chunk_data.transparent_buffers {
                        if transparent_buffers.num_indices > 0 {
                            render_pass.set_vertex_buffer(0, transparent_buffers.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(transparent_buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                            render_pass.draw_indexed(0..transparent_buffers.num_indices, 0, 0..1);
                        }
                    }
                }
            }
        }
        {
            let mut crosshair_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Crosshair Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });
            self.crosshair.draw(&mut crosshair_render_pass);
        }
        {
            let mut debug_text_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Debug Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None, occlusion_query_set: None,
            });
            self.debug_overlay.render(&mut debug_text_render_pass);
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    pollster::block_on(run());
}
// ... (rest of main.rs, if any) ...

[end of engine/src/main.rs]

[end of engine/src/main.rs]

[end of engine/src/main.rs]
