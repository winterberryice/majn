mod block;
mod chunk;
mod cube_geometry;
mod camera;
mod instance;
pub mod physics;
pub mod player;
mod debug_overlay;
mod world; // Add world module
mod raycast; // Add raycast module
mod wireframe_renderer; // Add wireframe_renderer module

use std::sync::Arc; // Added for Arc<Window>
use wgpu::Trace;
use winit::{
    application::ApplicationHandler, // Added for ApplicationHandler
    event::*,
    event_loop::{EventLoop, ActiveEventLoop, ControlFlow}, // Added ControlFlow
    window::{Window, WindowId}, // WindowId might be needed by ApplicationHandler methods
    keyboard::{KeyCode, PhysicalKey},
};

// Struct to hold application state and wgpu state
struct App { // Removed lifetime 'a
    window: Option<Arc<Window>>, // Changed to Option<Arc<Window>>
    state: Option<State>,      // Changed to Option<State> (State will also not have 'a)
    mouse_grabbed: bool,       // Added to track mouse grab state
    last_mouse_position: Option<winit::dpi::PhysicalPosition<f64>>, // Added to track last mouse position
}

impl App { // Removed lifetime 'a
    fn new() -> Self {
        Self {
            window: None,
            state: None,
            mouse_grabbed: false,
            last_mouse_position: None,
        }
    }

    // Helper method to manage mouse grab and cursor visibility
    fn set_mouse_grab(&mut self, grab: bool) {
        if let Some(window) = self.window.as_ref() {
            if grab {
                window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                    .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
                    .unwrap_or_else(|e| eprintln!("Failed to grab cursor: {:?}", e));
                window.set_cursor_visible(false);
            } else {
                window.set_cursor_grab(winit::window::CursorGrabMode::None)
                    .unwrap_or_else(|e| eprintln!("Failed to release cursor: {:?}", e));
                window.set_cursor_visible(true);
            }
            self.mouse_grabbed = grab;
        }
    }

    // New method to handle window events, adapted from ApplicationHandler::window_event
    fn handle_window_event(&mut self, event: WindowEvent, active_loop: &ActiveEventLoop) { // Renamed elwt to active_loop for clarity
        // --- Phase 1: Handle events that might change self.mouse_grabbed or cause an early exit ---
        // This phase operates on `&mut self` but NOT `&mut self.state` yet.
        let mut event_consumed_by_grab_logic = false;

        match event {
            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } if key_event.physical_key == PhysicalKey::Code(KeyCode::Escape) && key_event.state == ElementState::Pressed => {
                if self.mouse_grabbed {
                    self.set_mouse_grab(false); // Modifies self directly
                    event_consumed_by_grab_logic = true;
                } else {
                    active_loop.exit(); // Uses active_loop
                    return; // Early return, no further processing of this event
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, .. } => {
                if !self.mouse_grabbed {
                    self.set_mouse_grab(true); // Modifies self directly
                    // Optionally, mark as consumed if clicking to grab shouldn't also trigger game actions
                    // event_consumed_by_grab_logic = true;
                }
            }
            _ => {}
        }

        // --- Phase 2: Process event with State ---
        // Now, we can safely borrow `self.state.as_mut()` if the event wasn't fully handled by grab logic
        // or if grab logic doesn't preclude state processing.

        // We need `state` for most other event handling.
        let state = match self.state.as_mut() {
            Some(s) => s,
            // If state is None (e.g., after Resumed failed or before it ran),
            // most window events can't be processed meaningfully.
            None => return,
        };

        let mut event_handled_by_state_input = false;
        // Only pass event to state.input if not the Escape key press that was consumed by grab logic
        if !(event_consumed_by_grab_logic && matches!(event, WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::Escape), state: ElementState::Pressed, .. }, .. })) {
            event_handled_by_state_input = state.input(&event);
        }

        let mut cursor_moved_while_grabbed = false;
        if self.mouse_grabbed {
            if let WindowEvent::CursorMoved { position, .. } = event {
                // This specific handling of CursorMoved when grabbed might be redundant
                // if DeviceEvent::MouseMotion is the primary source of camera updates.
                // However, if other UI elements depend on CursorMoved even when grabbed, this is relevant.
                // For now, we mark it as potentially handled to prevent default processing if grabbed.
                let mut mouse_delta = (0.0, 0.0); // This delta is local to CursorMoved, DeviceEvent provides its own
                if let Some(last_pos) = self.last_mouse_position {
                    mouse_delta.0 = position.x - last_pos.x;
                    mouse_delta.1 = position.y - last_pos.y;
                }
                self.last_mouse_position = Some(position);
                // state.process_mouse_motion(mouse_delta.0, mouse_delta.1); // This is usually done by DeviceEvent
                cursor_moved_while_grabbed = true; // If grabbed, consider CursorMoved handled at this level
            }
        }


        // --- Phase 3: Default event handling for non-consumed events ---
        // These are events that weren't an Escape toggle, a click-to-grab,
        // weren't consumed by state.input(), and weren't a CursorMoved while grabbed.
        if !event_consumed_by_grab_logic && !event_handled_by_state_input && !cursor_moved_while_grabbed {
            match event {
                WindowEvent::CloseRequested => {
                    active_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    // This is the correct place for rendering logic triggered by the system.
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size), // Use existing size
                        Err(wgpu::SurfaceError::OutOfMemory) => active_loop.exit(),
                        Err(e) => eprintln!("Error rendering: {:?}", e),
                    }
                }
                // Other WindowEvents like ScaleFactorChanged, ThemeChanged, etc.
                // can be handled here if needed.
                _ => {}
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);

        if self.window.is_none() {
            let window_attributes = Window::default_attributes().with_title("Hello WGPU with ApplicationHandler!");
            let window_arc = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(Arc::clone(&window_arc));
            let initial_size = window_arc.inner_size();

            let state = pollster::block_on(State::new(Arc::clone(&window_arc), initial_size));
            self.state = Some(state);
            self.set_mouse_grab(true); // Initial mouse grab
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
        _event_loop: &ActiveEventLoop, // Prefixed with underscore as it's not used
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
            // Handle other device events if needed
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) { // Prefixed with underscore
        // This corresponds to the old Event::MainEventsCleared or Event::AboutToWait
        // Request a redraw continuously for animation, if the window exists.
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        // Note: RedrawRequested events will be handled in window_event
        // Rendering logic (update, render) will be triggered by WindowEvent::RedrawRequested
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) { // Prefixed with underscore
        // Corresponds to the old Event::LoopDestroyed or Event::LoopExiting
        println!("ApplicationHandler: Event loop is exiting. Cleaning up.");
        // Explicitly drop state and window if necessary, though Arc and Option should handle it.
        // self.state = None;
        // self.window = None;
    }

    // We might need to implement other methods like `new_events` or `memory_warning`
    // if specific behaviors are needed for those, but for now, the defaults are fine.
}

// Represents a single point on a shape.
// bytemuck is used to safely cast our struct into a slice of bytes that the GPU can understand.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex { // Made public
    pub position: [f32; 3], // Made public
    pub color: [f32; 3],    // Made public
}

impl Vertex {
    // This describes the memory layout of a single vertex to the shader.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> { // Made public
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // Corresponds to `layout(location = 0)` in shader
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1, // Corresponds to `layout(location = 1)` in shader
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// Our vertices for a triangle with colored corners.
// const VERTICES: &[Vertex] = &[
//     Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },    // Top (Red)
//     Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] }, // Bottom-left (Green)
//     Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },  // Bottom-right (Blue)
// ];
// use crate::cube_geometry;
use crate::camera::CameraUniform; // Camera and CameraController removed
use crate::chunk::{CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH}; // Import chunk dimensions, Chunk itself is not directly used in State anymore
use crate::cube_geometry::CubeFace; // Import CubeFace
use crate::player::Player; // Import Player
use crate::physics::PLAYER_EYE_HEIGHT; // For camera positioning
use glam::Mat4; // For view/projection matrix calculation
use crate::debug_overlay::DebugOverlay;
use crate::world::World;
use crate::raycast::BlockFace;
use glam::IVec3;
use crate::wireframe_renderer::WireframeRenderer; // Import WireframeRenderer
use std::collections::HashMap; // For storing chunk render data

// Struct to hold GPU buffers for a single chunk's mesh
struct ChunkRenderData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

// The State struct holds all of our wgpu-related objects.
struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    // Player replaces Camera and CameraController
    player: Player,

    // CameraUniform and related GPU resources are still needed for rendering
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    // chunk: Chunk, // Replaced by world
    world: World,
    // New fields for managing multiple chunk meshes
    // chunk_vertex_buffer: Option<wgpu::Buffer>, // Removed
    // chunk_index_buffer: Option<wgpu::Buffer>, // Removed
    // num_chunk_indices: u32, // Removed
    chunk_render_data: HashMap<(i32, i32), ChunkRenderData>, // Stores VBs/IBs per chunk coord
    active_chunk_coords: Vec<(i32, i32)>, // Coords of chunks to render

    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    debug_overlay: DebugOverlay,
    wireframe_renderer: WireframeRenderer, // For selected block outline

    selected_block: Option<(IVec3, BlockFace)>, // For raycasting result
}

impl State {
    async fn new(window_surface_target: Arc<Window>, initial_size: winit::dpi::PhysicalSize<u32>) -> Self {
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
                // None, // trace_path - Removed as it's an extra argument
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

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("camera_bind_group_layout"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                // Only Vertex::desc() now, no InstanceRaw::desc()
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back), // Still use back-face culling GPU-side
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

        // Player setup
        let initial_player_position = glam::Vec3::new(
            CHUNK_WIDTH as f32 / 2.0,
            (CHUNK_HEIGHT / 2) as f32 + 2.0, // Start slightly above the surface
            CHUNK_DEPTH as f32 / 2.0,
        );
        let initial_yaw = -std::f32::consts::FRAC_PI_2; // Look along -Z
        let initial_pitch = 0.0;
        let mouse_sensitivity = 0.003; // Same as old CameraController

        let player = Player::new(
            initial_player_position,
            initial_yaw,
            initial_pitch,
            mouse_sensitivity,
        );

        // CameraUniform setup - still needed, but will be updated by Player's state
        // We need a temporary Camera struct or matrix to initialize it the first time.
        // Or, we can update it in the first call to State::update().
        // For now, let's initialize it with identity, it will be updated before first render.
        let camera_uniform = CameraUniform::new();
        // camera_uniform.update_view_proj(&camera); // This will be done in State::update

        use wgpu::util::DeviceExt; // For create_buffer_init
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        // World initialization
        let world = World::new(); // Create an empty world
        // Initial chunk generation and meshing will happen in the first `State::update()` call.

        // Depth texture
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
        let wireframe_renderer = WireframeRenderer::new(&device, &config, &camera_bind_group_layout);

        let state = Self {
            surface,
            device,
            queue,
            config,
            size: initial_size,
            render_pipeline,
            player, // Player is now part of State
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            // camera and camera_controller removed
            // chunk, // Replaced by world
            world,
            // chunk_vertex_buffer: None, // Removed
            // chunk_index_buffer: None, // Removed
            // num_chunk_indices: 0, // Removed
            chunk_render_data: HashMap::new(),
            active_chunk_coords: Vec::new(),
            depth_texture,
            depth_texture_view,
            debug_overlay,
            wireframe_renderer,
            selected_block: None,
        };

        // Initial chunk loading and meshing will be handled by the first `update` call.
        // state.build_chunk_mesh(); // Old call for single chunk

        state
    }

    // fn build_chunk_mesh(&mut self) { // Old signature for single chunk
    // New signature: takes chunk coords, and accesses World to get chunk data
    fn build_or_rebuild_chunk_mesh(&mut self, chunk_cx: i32, chunk_cz: i32) {
        let mut chunk_mesh_vertices: Vec<Vertex> = Vec::new();
        let mut chunk_mesh_indices: Vec<u16> = Vec::new();
        let mut current_vertex_offset: u16 = 0;

        // Get the specific chunk from the world
        // We might need a mutable borrow of world here if get_chunk isn't sufficient
        // or if we want to generate the chunk if it doesn't exist (though get_or_create_chunk is for that)
        // For mesh building, we assume the chunk exists and has its block data.
        let chunk_opt = self.world.get_chunk(chunk_cx, chunk_cz);
        if chunk_opt.is_none() {
            // This case should ideally be handled by the caller:
            // ensure chunk exists and is generated before trying to mesh it.
            // For now, if it doesn't exist, we can't mesh it.
            // Or, we could use get_or_create_chunk if world was mut.
            // Let's assume `update` ensures chunks are loaded before calling this.
            eprintln!("Attempted to build mesh for non-existent or non-generated chunk ({}, {})", chunk_cx, chunk_cz);
            // Remove any existing render data if we decide to "unload" it visually
            self.chunk_render_data.remove(&(chunk_cx, chunk_cz));
            return;
        }
        let chunk = chunk_opt.unwrap(); // Safe due to check above

        // Vertex positions need to be relative to the chunk's origin in world space.
        // A block at local chunk coords (lx, ly, lz) for a chunk at (cx, cz)
        // will have its base corner at world coords:
        // (cx * CHUNK_WIDTH + lx, ly, cz * CHUNK_DEPTH + lz)
        let chunk_world_origin_x = chunk_cx as f32 * CHUNK_WIDTH as f32;
        let chunk_world_origin_z = chunk_cz as f32 * CHUNK_DEPTH as f32;


        for lx in 0..CHUNK_WIDTH {
            for ly in 0..CHUNK_HEIGHT {
                for lz in 0..CHUNK_DEPTH {
                    if let Some(block) = chunk.get_block(lx, ly, lz) {
                        if block.is_solid() {
                            let block_color = match block.block_type {
                                crate::block::BlockType::Dirt => [0.5, 0.25, 0.05],
                                crate::block::BlockType::Grass => [0.0, 0.8, 0.1],
                                crate::block::BlockType::Air => continue,
                            };

                            // Calculate the world center of the current block
                            // Local center is (lx+0.5, ly+0.5, lz+0.5)
                            // World center is (chunk_world_origin_x + lx + 0.5, ly + 0.5, chunk_world_origin_z + lz + 0.5)
                            let current_block_world_center = glam::Vec3::new(
                                chunk_world_origin_x + lx as f32 + 0.5,
                                ly as f32 + 0.5, // Y is absolute block coordinate
                                chunk_world_origin_z + lz as f32 + 0.5
                            );

                            // Face culling logic: Check neighbors
                            // Neighbors can be in the same chunk or adjacent chunks.
                            let face_definitions: [(CubeFace, (i32, i32, i32)); 6] = [
                                (CubeFace::Front,  (0, 0, -1)), // Relative to current block: (lx, ly, lz-1)
                                (CubeFace::Back,   (0, 0, 1)),  // (lx, ly, lz+1)
                                (CubeFace::Right,  (1, 0, 0)),  // (lx+1, ly, lz)
                                (CubeFace::Left,   (-1, 0, 0)), // (lx-1, ly, lz)
                                (CubeFace::Top,    (0, 1, 0)),  // (lx, ly+1, lz)
                                (CubeFace::Bottom, (0, -1, 0)), // (lx, ly-1, lz)
                            ];

                            for (face_type, offset) in face_definitions.iter() {
                                // Calculate absolute world coordinates of the neighbor block to check
                                let neighbor_world_bx = chunk_world_origin_x as i32 + lx as i32 + offset.0;
                                let neighbor_world_by = ly as i32 + offset.1; // Y is absolute
                                let neighbor_world_bz = chunk_world_origin_z as i32 + lz as i32 + offset.2;

                                let mut is_face_visible = true;
                                // Check if neighbor is outside world bounds (e.g. y < 0 or y >= CHUNK_HEIGHT)
                                if neighbor_world_by < 0 || neighbor_world_by >= CHUNK_HEIGHT as i32 {
                                    // Neighbor is outside vertical build limit, face is visible
                                } else {
                                    // Query the world for the neighbor block
                                    if let Some(neighbor_block) = self.world.get_block_at_world(
                                        neighbor_world_bx as f32,
                                        neighbor_world_by as f32,
                                        neighbor_world_bz as f32
                                    ) {
                                        if neighbor_block.is_solid() {
                                            is_face_visible = false;
                                        }
                                    }
                                    // If neighbor_block is None (e.g. chunk not loaded), face is visible.
                                }


                                if is_face_visible {
                                    let vertices_template = face_type.get_vertices_template();
                                    let local_indices = face_type.get_local_indices();

                                    for v_template in vertices_template {
                                        chunk_mesh_vertices.push(Vertex {
                                            // Vertex positions are already relative to block center.
                                            // We add the block's world center to get final world vertex positions.
                                            position: (current_block_world_center + glam::Vec3::from(v_template.position)).into(),
                                            color: block_color,
                                        });
                                    }
                                    for local_idx in local_indices {
                                        chunk_mesh_indices.push(current_vertex_offset + local_idx);
                                    }
                                    current_vertex_offset += vertices_template.len() as u16;
                                }
                            }
                        }
                    }
                }
            }
        }

        if !chunk_mesh_vertices.is_empty() && !chunk_mesh_indices.is_empty() {
            use wgpu::util::DeviceExt;
            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Chunk VB ({}, {})", chunk_cx, chunk_cz)),
                contents: bytemuck::cast_slice(&chunk_mesh_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Chunk IB ({}, {})", chunk_cx, chunk_cz)),
                contents: bytemuck::cast_slice(&chunk_mesh_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            self.chunk_render_data.insert((chunk_cx, chunk_cz), ChunkRenderData {
                vertex_buffer,
                index_buffer,
                num_indices: chunk_mesh_indices.len() as u32,
            });
        } else {
            // No visible faces, or chunk is empty. Remove existing render data if any.
            self.chunk_render_data.remove(&(chunk_cx, chunk_cz));
        }
    }


    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            // self.camera.aspect = self.config.width as f32 / self.config.height as f32; // Removed, self.camera no longer exists

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
            self.depth_texture_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

            self.surface.configure(&self.device, &self.config);
            self.debug_overlay.resize(new_size.width, new_size.height, &self.queue); // Added &self.queue
        }
    }

    pub fn process_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        // Sensitivity check is handled inside player.process_mouse_movement if needed,
        // or we assume it's always active if called.
        // The player's mouse_sensitivity field is used by its own method.
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
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.player.movement_intention.forward = is_pressed; true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.player.movement_intention.backward = is_pressed; true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.player.movement_intention.left = is_pressed; true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.player.movement_intention.right = is_pressed; true
                    }
                    KeyCode::Space => {
                        // For player, Space is jump. "Up" movement (flying) is removed.
                        self.player.movement_intention.jump = is_pressed; true
                    }
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                        // "Down" movement (flying) is removed. Shift could be for sprinting or crouching later.
                        // For now, it does nothing with the player controller.
                        false // Or handle as sprint/crouch if implemented
                    }
                    KeyCode::Escape => false, // Escape is handled by App for mouse grab
                    KeyCode::F3 => {
                        if is_pressed { // Only toggle on press
                            self.debug_overlay.toggle_visibility();
                        }
                        true // Event handled regardless of press/release to consume it
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        // TODO: Calculate actual delta time instead of fixed value
        let dt_secs = 1.0 / 60.0; // Using f32 directly for physics

        // 1. Determine current and required chunks
        let player_pos = self.player.position;
        let current_chunk_x = (player_pos.x / CHUNK_WIDTH as f32).floor() as i32;
        let current_chunk_z = (player_pos.z / CHUNK_DEPTH as f32).floor() as i32;

        let mut new_active_chunk_coords = Vec::new();
        let render_distance = 1; // Render 1 chunk around current = 3x3 grid (current +/- 1)
        for dx in -render_distance..=render_distance {
            for dz in -render_distance..=render_distance {
                let target_cx = current_chunk_x + dx;
                let target_cz = current_chunk_z + dz;
                new_active_chunk_coords.push((target_cx, target_cz));

                // Ensure chunk data exists (generate if new)
                // Note: get_or_create_chunk needs &mut self.world
                let _ = self.world.get_or_create_chunk(target_cx, target_cz);

                // Ensure mesh exists (build if new or outdated)
                // We only rebuild if it's not in chunk_render_data.
                // A more sophisticated system would track "dirty" chunks.
                if !self.chunk_render_data.contains_key(&(target_cx, target_cz)) {
                    // This needs to borrow `self` mutably, which `world.get_or_create_chunk` also does.
                    // This is tricky. We might need to collect coords first, then iterate again for meshing.
                    // For now, let's assume get_or_create_chunk doesn't invalidate references needed by build_or_rebuild_chunk_mesh
                    // or that build_or_rebuild_chunk_mesh fetches its own chunk reference.
                    // The current build_or_rebuild_chunk_mesh fetches its own immutable chunk reference from self.world.
                    // So, the mutable borrow of self.world for get_or_create_chunk must end before build_or_rebuild_chunk_mesh
                    // borrows self.world immutably.
                    // This implies a two-pass approach or careful management.

                    // Let's try a simplified approach for now:
                    // Pass 1: Ensure all chunks in `world.chunks` are generated.
                    // Pass 2: Iterate `new_active_chunk_coords` and build meshes if not present in `chunk_render_data`.
                    // This is what's implicitly happening as `get_or_create_chunk` is called above,
                    // and then `build_or_rebuild_chunk_mesh` is called below if needed.
                    // The main issue is `build_or_rebuild_chunk_mesh` needs `&mut self` to store render data.
                }
            }
        }

        // Update the list of active chunk coordinates for rendering
        self.active_chunk_coords = new_active_chunk_coords; // No need to clone here if new_active_chunk_coords is already a new Vec

        // Pass 2: Build meshes for newly activated chunks or those needing an update
        let mut coords_to_mesh: Vec<(i32, i32)> = Vec::new();
        for &(cx, cz) in &self.active_chunk_coords { // Iterate over self.active_chunk_coords immutably
            if !self.chunk_render_data.contains_key(&(cx, cz)) {
                coords_to_mesh.push((cx, cz)); // Collect coordinates
            }
        }

        // Now iterate over the separate Vec, allowing mutable borrows of self
        for (cx, cz) in coords_to_mesh {
            // Since world.get_or_create_chunk was called earlier for (cx,cz),
            // the chunk data should exist in self.world.
            // Now call build_or_rebuild_chunk_mesh which takes &mut self.
            self.build_or_rebuild_chunk_mesh(cx, cz);
        }

        // (Future: Unload meshes for chunks no longer in active_chunk_coords)
        // self.chunk_render_data.retain(|coord, _| self.active_chunk_coords.contains(coord));


        // 2. Update player physics and collision (now using self.world)
        self.player.update_physics_and_collision(dt_secs, &self.world);

        // New: Perform raycasting
        const RAYCAST_MAX_DISTANCE: f32 = 5.0;
        self.selected_block = crate::raycast::cast_ray(&self.player, &self.world, RAYCAST_MAX_DISTANCE);
        // For debugging:
        // if let Some((pos, face)) = self.selected_block {
        //     println!("Selected block: {:?} at face {:?}", pos, face);
        // }


        // 3. Update camera view based on player state
        let camera_eye = self.player.position + glam::Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0);

        // Calculate target based on player's yaw and pitch
        // This logic is similar to how CameraController used to calculate its target.
        let camera_front = glam::Vec3::new(
            self.player.yaw.cos() * self.player.pitch.cos(),
            self.player.pitch.sin(),
            self.player.yaw.sin() * self.player.pitch.cos(),
        ).normalize();
        let camera_target = camera_eye + camera_front;

        let view_matrix = Mat4::look_at_rh(camera_eye, camera_target, glam::Vec3::Y);

        // Projection matrix (aspect ratio might need to be updated if window resizes,
        // which is handled by resize() method for config, but perspective_rh needs it too)
        let aspect_ratio = self.config.width as f32 / self.config.height as f32;
        let fovy_radians = 45.0f32.to_radians(); // Example FOV, could be configurable
        let znear = 0.1;
        let zfar = 1000.0; // Make sure this is far enough
        let projection_matrix = Mat4::perspective_rh(fovy_radians, aspect_ratio, znear, zfar);

        let view_proj_matrix = projection_matrix * view_matrix;
        self.camera_uniform.view_proj = view_proj_matrix.to_cols_array_2d();

        // 3. Write updated camera uniform to GPU buffer
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        // Chunk mesh is static for now after initial build.
        // If blocks could change, we would call self.build_chunk_mesh() here or when a change occurs.

        // Update debug overlay
        self.debug_overlay.update(self.player.position);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Prepare debug overlay text before getting the current texture
        // This is important because brush.queue() might involve GPU operations
        if let Err(e) = self.debug_overlay.prepare(&self.device, &self.queue) {
            eprintln!("Failed to prepare debug overlay: {:?}", e);
            // Decide if this error is critical. For a debug overlay, maybe not.
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1, g: 0.2, b: 0.3, a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Iterate over active chunks and render them
            for chunk_coord in &self.active_chunk_coords {
                if let Some(render_data) = self.chunk_render_data.get(chunk_coord) {
                    if render_data.num_indices > 0 {
                        render_pass.set_vertex_buffer(0, render_data.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(render_data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..render_data.num_indices, 0, 0..1);
                    }
                }
            }

            // New: Render selected block wireframe
            if let Some((block_coords, _)) = self.selected_block {
                self.wireframe_renderer.update_model_matrix(block_coords);
                // The camera bind group (group 0) is already set by the main world render pass.
                // The WireframeRenderer's draw method will set its own pipeline and bind group 1.
                self.wireframe_renderer.draw(&mut render_pass, &self.queue);
            }

            // Render debug overlay
            self.debug_overlay.render(&mut render_pass); // Removed error handling

        }

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    // API Change: EventLoop::new() now returns a Result, which we unwrap.
    let event_loop = EventLoop::new().unwrap();
    // Corrected: winit 0.30 window creation - This will move into ApplicationHandler
    // let window = event_loop.create_window(Window::default_attributes().with_title("Hello WGPU!")).unwrap();
    // let mut state = State::new(window.clone()).await; // If window were Arc'd here

    // API Change: The event loop closure now takes different arguments.
    // The `elwt` (Event Loop Window Target) is used to control the loop (e.g., to exit).
    // OLD event_loop.run CALL WILL BE REPLACED BY run_app
    /*
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => { // state would be part of App
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        // API Change: The keyboard event structure is different now.
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => {
                            elwt.exit(); // Corrected: Use elwt.exit() for winit 0.30
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        // API Change: RedrawRequested is now a WindowEvent.
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                // Corrected: Use elwt.exit() for winit 0.30
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            // API Change: AboutToWait is the new place to request a redraw for continuous rendering.
            Event::AboutToWait => {
                // state.window().request_redraw(); // window would be part of App
            }
            _ => {}
        }
    }).unwrap();
    */
    // let mut app = App::new(); // This was the first instance, now removed.
    // event_loop.run_app(&mut app) will return Result, so unwrap or handle
    // The unwrap is fine for an example, but actual error handling might be better.
    // Create the App instance (which now implements ApplicationHandler)
    let mut app = App::new(); // This is the one that's actually used.

    // Run the event loop using run_app
    // run_app takes ownership of the event_loop and mutable reference to the app.
    // It doesn't return until the event loop exits.
    // The .unwrap() here handles potential errors from event_loop.run_app itself,
    // though for winit, run_app typically doesn't return a Result that needs unwrapping
    // unless specific platform extensions are used or if EventLoop::new() failed.
    // The primary error source was EventLoop::new(), which is already unwrap()'d.
    // For winit 0.29/0.30, run_app does not return a Result.
    event_loop.run_app(&mut app).unwrap();
    // If run_app did return a Result<Result<(), Error>, EventLoopError> or similar,
    // more careful error handling might be needed.
    // However, the typical signature is `pub fn run_app<A: ApplicationHandler>(self, app: A) -> Result<(), EventLoopError>`
    // or on some platforms `pub fn run_app<A: ApplicationHandler>(self, app: A)` (no Result).
    // The `unwrap()` is kept here assuming the `EventLoop::run` it replaces also had an unwrap,
    // but it might need removal if `run_app` for the target winit version doesn't return a Result.
    // Checking winit 0.30 docs: `run_app` returns `Result<(), EventLoopError>`. So unwrap is appropriate.

}

fn main() {
    pollster::block_on(run());
}