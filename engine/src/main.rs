mod block;
mod camera;
mod chunk;
mod cube_geometry;
mod debug_overlay;
mod input;
pub mod physics;
pub mod player;
mod raycast; // Add raycast module
mod texture;
mod ui; // Added for Crosshair
mod wireframe_renderer;
mod world; // Add world module // Add wireframe_renderer module

use std::sync::Arc; // Added for Arc<Window>
use wgpu::Trace;
use winit::{
    application::ApplicationHandler, // Added for ApplicationHandler
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, // Added ControlFlow
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId}, // WindowId might be needed by ApplicationHandler methods
};

// Struct to hold application state and wgpu state
struct App {
    // Removed lifetime 'a
    window: Option<Arc<Window>>, // Changed to Option<Arc<Window>>
    state: Option<State>,        // Changed to Option<State> (State will also not have 'a)
    // input_state: input::InputState, // REMOVED: Will be part of State
    mouse_grabbed: bool, // Added to track mouse grab state
    last_mouse_position: Option<winit::dpi::PhysicalPosition<f64>>, // Added to track last mouse position
}

impl App {
    // Removed lifetime 'a
    fn new() -> Self {
        Self {
            window: None,
            state: None,
            // input_state: input::InputState::new(), // REMOVED
            mouse_grabbed: false,
            last_mouse_position: None,
        }
    }

    // Helper method to manage mouse grab and cursor visibility
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

    // New method to handle window events, adapted from ApplicationHandler::window_event
    fn handle_window_event(&mut self, event: WindowEvent, active_loop: &ActiveEventLoop) {
        // Renamed elwt to active_loop for clarity
        // --- Phase 1: Handle events that might change self.mouse_grabbed or cause an early exit ---
        // This phase operates on `&mut self` but NOT `&mut self.state` yet.
        let mut event_consumed_by_grab_logic = false;

        match event {
            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } if key_event.physical_key == PhysicalKey::Code(KeyCode::Escape)
                && key_event.state == ElementState::Pressed =>
            {
                if self.mouse_grabbed {
                    self.set_mouse_grab(false); // Modifies self directly
                    event_consumed_by_grab_logic = true;
                } else {
                    active_loop.exit(); // Uses active_loop
                    return; // Early return, no further processing of this event
                }
            }
            WindowEvent::MouseInput {
                button,
                state: mouse_element_state,
                ..
            } => {
                // Capture button and state, renamed state to mouse_element_state to avoid conflict
                // Pass mouse input to State's InputState handler
                if let Some(s) = self.state.as_mut() {
                    s.input_state.on_mouse_input(button, mouse_element_state);
                }

                if mouse_element_state == ElementState::Pressed {
                    // Existing logic for mouse grab
                    if !self.mouse_grabbed {
                        self.set_mouse_grab(true); // Modifies self directly
                        // event_consumed_by_grab_logic = true; // Potentially consume this click for grabbing only
                    }
                }
            }
            _ => {}
        }

        // --- Phase 2: Process event with State ---
        // Now, we can safely borrow `self.state.as_mut()` if the event wasn't fully handled by grab logic
        // or if grab logic doesn't preclude state processing.

        // We need `state` for most other event handling.
        let state = match self.state.as_mut() {
            // state needs to be mutable here
            Some(s) => s,
            // If state is None (e.g., after Resumed failed or before it ran),
            // most window events can't be processed meaningfully.
            None => return,
        };

        let mut event_handled_by_state_input = false;
        // Only pass event to state.input if not the Escape key press that was consumed by grab logic
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
                    // This is the correct place for rendering logic triggered by the system.
                    // We need to borrow input_state mutably here, separate from state.
                    // This is tricky because state also comes from self.
                    // Let's try to call update with input_state from self.
                    // This might require restructuring if Rust's borrow checker complains.
                    // For now, let's assume we can pass `&mut self.input_state`.

                    // The issue: `state` is `&mut self.state.unwrap()`.
                    // `self.input_state` is another field of `self`.
                    // We cannot have two mutable borrows of `self` or parts of `self` simultaneously
                    // if `state.update` takes `&mut self`, and we pass `&mut self.input_state`.
                    // However, `state.update` takes `&mut self` (referring to `State` instance)
                    // and `&mut input_state` as a separate argument.

                    // Let's try to get `input_state` first, then `state`.
                    // This won't work as `self.state.as_mut()` borrows `self` mutably.

                    // The most straightforward way is to temporarily take `input_state` out of `self`,
                    // then call `state.update()`, then put `input_state` back.
                    // This is not ideal.
                    // A better way: `State::update` should not take `&mut self` if it also needs `&mut InputState` from `App`.
                    // Or, `InputState` becomes part of `State`.

                    // Let's make InputState part of State for simplicity.
                    // This requires changes in:
                    // 1. App struct: remove input_state
                    // 2. App::new(): remove input_state init
                    // 3. App::handle_window_event (MouseInput): call state.input_state.on_mouse_input()
                    // 4. State struct: add input_state field
                    // 5. State::new(): init input_state
                    // 6. State::update(): access self.input_state directly
                    // This seems like a more Rusty way to handle ownership.

                    // === REVISED PLAN FOR THIS STEP ===
                    // 1. Move InputState ownership to the State struct.
                    // 2. Update App event handling to call state.input_state.on_mouse_input().
                    // 3. Update State::update() to use its own input_state.

                    // For now, I will proceed with the original plan and see if the borrow checker complains.
                    // If it does, I will refactor to move InputState into State.
                    // The current signature of state.update is `fn update(&mut self, input_state: &mut input::InputState)`
                    // `state` here is `&mut State`.
                    // `self` in `App::handle_window_event` is `&mut App`.
                    // So we'd be calling `state.update(&mut self.input_state)`.
                    // This means `state` (which is `self.state.as_mut().unwrap()`) is one mutable borrow.
                    // `&mut self.input_state` is another mutable borrow from `self`.
                    // This is a conflict.

                    // Refactoring to move InputState into State is the way.

                    // --- START REFACTOR ---
                    // This change is larger than just this location.
                    // I will make the necessary changes across files for this refactor.
                    // I will start by removing input_state from App and adding it to State.

                    // The call will become: state.update() and State::update will internally use self.input_state.
                    state.update(); // State::update will be changed to use its own InputState
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
            let window_attributes =
                Window::default_attributes().with_title("Hello WGPU with ApplicationHandler!");
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Prefixed with underscore
        // This corresponds to the old Event::MainEventsCleared or Event::AboutToWait
        // Request a redraw continuously for animation, if the window exists.
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        // Note: RedrawRequested events will be handled in window_event
        // Rendering logic (update, render) will be triggered by WindowEvent::RedrawRequested
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // Prefixed with underscore
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
pub struct Vertex {
    // Made public
    pub position: [f32; 3], // Made public
    pub color: [f32; 3],    // Made public
    pub uv: [f32; 2],       // Added for texture coordinates
}

impl Vertex {
    // This describes the memory layout of a single vertex to the shader.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        // Made public
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // position
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1, // color
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress, // After position and color
                    shader_location: 2,                                                   // uv
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

use crate::block::BlockType; // For block placement/removal
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
use std::collections::HashMap; // For collision checking

struct ChunkRenderData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
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
    // diffuse_texture: crate::texture::Texture,
    // texture_bind_group_layout: wgpu::BindGroupLayout,
    diffuse_bind_group: wgpu::BindGroup,
    input_state: input::InputState, // Added InputState here
}

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
                    blend: Some(wgpu::BlendState::REPLACE),
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
            // diffuse_texture,
            // texture_bind_group_layout,
            diffuse_bind_group,
            input_state: input::InputState::new(), // Initialize InputState
        }
    }

    fn build_or_rebuild_chunk_mesh(&mut self, chunk_cx: i32, chunk_cz: i32) {
        let mut chunk_mesh_vertices: Vec<Vertex> = Vec::new();
        let mut chunk_mesh_indices: Vec<u16> = Vec::new();
        let mut current_vertex_offset: u16 = 0;

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
                        if block.is_solid() {
                            // Default block_color, might be overridden for specific types/faces
                            let default_block_color = match block.block_type {
                                crate::block::BlockType::Dirt => [0.5, 0.25, 0.05],
                                crate::block::BlockType::Grass => [0.0, 0.8, 0.1], // Default grass color
                                crate::block::BlockType::Air => continue,
                            };

                            let current_block_world_center = glam::Vec3::new(
                                chunk_world_origin_x + lx as f32 + 0.5,
                                ly as f32 + 0.5,
                                chunk_world_origin_z + lz as f32 + 0.5,
                            );

                            let face_definitions: [(CubeFace, (i32, i32, i32)); 6] = [
                                (CubeFace::Front, (0, 0, -1)),
                                (CubeFace::Back, (0, 0, 1)),
                                (CubeFace::Right, (1, 0, 0)),
                                (CubeFace::Left, (-1, 0, 0)),
                                (CubeFace::Top, (0, 1, 0)),
                                (CubeFace::Bottom, (0, -1, 0)),
                            ];

                            for (face_type, offset) in face_definitions.iter() {
                                let neighbor_world_bx =
                                    chunk_world_origin_x as i32 + lx as i32 + offset.0;
                                let neighbor_world_by = ly as i32 + offset.1;
                                let neighbor_world_bz =
                                    chunk_world_origin_z as i32 + lz as i32 + offset.2;

                                let mut is_face_visible = true;
                                if neighbor_world_by < 0 || neighbor_world_by >= CHUNK_HEIGHT as i32
                                {
                                } else {
                                    if let Some(neighbor_block) = self.world.get_block_at_world(
                                        neighbor_world_bx as f32,
                                        neighbor_world_by as f32,
                                        neighbor_world_bz as f32,
                                    ) {
                                        if neighbor_block.is_solid() {
                                            is_face_visible = false;
                                        }
                                    }
                                }

                                if is_face_visible {
                                    let vertices_template = face_type.get_vertices_template();
                                    let local_indices = face_type.get_local_indices();

                                    const ATLAS_COLS: f32 = 16.0;
                                    const ATLAS_ROWS: f32 = 16.0;
                                    let tex_size_x = 1.0 / ATLAS_COLS;
                                    let tex_size_y = 1.0 / ATLAS_ROWS;

                                    let tex_coords_idx: (f32, f32); // Removed mut
                                    let mut current_vertex_color: [f32; 3];

                                    match block.block_type {
                                        crate::block::BlockType::Grass => {
                                            current_vertex_color = [0.0, 0.8, 0.1]; // Default for grass sides
                                            match face_type {
                                                CubeFace::Top => {
                                                    tex_coords_idx = (0.0, 0.0); // Grass Top (grayscale)
                                                    current_vertex_color = [0.1, 0.9, 0.1]; // Sentinel for tinting
                                                }
                                                CubeFace::Bottom => {
                                                    tex_coords_idx = (2.0, 0.0); // Dirt texture
                                                    current_vertex_color = [0.5, 0.25, 0.05]; // Standard Dirt color
                                                }
                                                _ => {
                                                    // Sides
                                                    tex_coords_idx = (3.0, 0.0); // Grass Side texture
                                                    // current_vertex_color remains [0.0, 0.8, 0.1] (standard grass color)
                                                }
                                            }
                                        }
                                        crate::block::BlockType::Dirt => {
                                            tex_coords_idx = (2.0, 0.0); // Dirt texture
                                            current_vertex_color = [0.5, 0.25, 0.05]; // Standard Dirt color
                                        }
                                        _ => {
                                            tex_coords_idx = (15.0, 15.0);
                                            current_vertex_color = default_block_color;
                                        }
                                    };

                                    let u_min = tex_coords_idx.0 * tex_size_x;
                                    let v_min = tex_coords_idx.1 * tex_size_y;
                                    let u_max = u_min + tex_size_x;
                                    let v_max = v_min + tex_size_y;

                                    let face_uvs = [
                                        [u_min, v_max],
                                        [u_max, v_max],
                                        [u_max, v_min],
                                        [u_min, v_min],
                                    ];

                                    for (i, v_template) in vertices_template.iter().enumerate() {
                                        chunk_mesh_vertices.push(Vertex {
                                            position: (current_block_world_center
                                                + glam::Vec3::from(v_template.position))
                                            .into(),
                                            color: current_vertex_color,
                                            uv: face_uvs[i],
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
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Chunk VB ({}, {})", chunk_cx, chunk_cz)),
                    contents: bytemuck::cast_slice(&chunk_mesh_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Chunk IB ({}, {})", chunk_cx, chunk_cz)),
                    contents: bytemuck::cast_slice(&chunk_mesh_indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

            self.chunk_render_data.insert(
                (chunk_cx, chunk_cz),
                ChunkRenderData {
                    vertex_buffer,
                    index_buffer,
                    num_indices: chunk_mesh_indices.len() as u32,
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
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.player.movement_intention.forward = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.player.movement_intention.backward = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.player.movement_intention.left = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.player.movement_intention.right = is_pressed;
                        true
                    }
                    KeyCode::Space => {
                        self.player.movement_intention.jump = is_pressed;
                        true
                    }
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => false,
                    KeyCode::Escape => false,
                    KeyCode::F3 => {
                        if is_pressed {
                            self.debug_overlay.toggle_visibility();
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        // Removed input_state from parameters
        // Handle block interactions first
        self.handle_block_interactions(); // Will now use self.input_state

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
                if !self.chunk_render_data.contains_key(&(target_cx, target_cz)) {}
            }
        }
        self.active_chunk_coords = new_active_chunk_coords;

        let mut coords_to_mesh: Vec<(i32, i32)> = Vec::new();
        for &(cx, cz) in &self.active_chunk_coords {
            if !self.chunk_render_data.contains_key(&(cx, cz)) {
                coords_to_mesh.push((cx, cz));
            }
        }
        for (cx, cz) in coords_to_mesh {
            self.build_or_rebuild_chunk_mesh(cx, cz);
        }

        self.player
            .update_physics_and_collision(dt_secs, &self.world);

        const RAYCAST_MAX_DISTANCE: f32 = 5.0;
        self.selected_block =
            crate::raycast::cast_ray(&self.player, &self.world, RAYCAST_MAX_DISTANCE);

        let camera_eye = self.player.position + glam::Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0);
        let camera_front = glam::Vec3::new(
            self.player.yaw.cos() * self.player.pitch.cos(),
            self.player.pitch.sin(),
            self.player.yaw.sin() * self.player.pitch.cos(),
        )
        .normalize();
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

        // Clear per-frame input flags
        self.input_state.clear_frame_state(); // Use self.input_state
    }

    // New function to handle block interactions based on input
    // Changed to use self.input_state
    fn handle_block_interactions(&mut self) {
        // Block Removal (Left-Click)
        if self.input_state.left_mouse_pressed_this_frame {
            if let Some((block_pos, _face)) = self.selected_block {
                match self.world.set_block(block_pos, BlockType::Air) {
                    Ok(chunk_coord) => {
                        // Mark chunk as dirty by removing its render data
                        self.chunk_render_data.remove(&chunk_coord);
                        // Also, if the removed block was on a boundary, the adjacent chunk might need updating
                        // This is complex; for now, just rebuild the primary chunk.
                        // A more robust solution would check neighbors if block is on edge.
                        // We might also need to rebuild neighbors if a block *removal* exposes their faces.
                        // For simplicity, we'll rely on the existing active chunk meshing logic to pick up
                        // changes when the player moves, or we can force rebuilds of neighbors.
                        // Let's check and rebuild neighbors of the modified chunk as well.
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1); // Rebuild the main chunk
                        // And its direct neighbors, as face visibility might change
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0 + 1, chunk_coord.1);
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0 - 1, chunk_coord.1);
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1 + 1);
                        self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1 - 1);
                    }
                    Err(e) => {
                        eprintln!("Error removing block: {}", e);
                    }
                }
            }
        }

        // Block Placement (Right-Click)
        if self.input_state.right_mouse_pressed_this_frame {
            // Use self.input_state
            if let Some((selected_block_pos, hit_face)) = self.selected_block {
                let mut offset = IVec3::ZERO;
                match hit_face {
                    BlockFace::PosX => offset.x = 1,
                    BlockFace::NegX => offset.x = -1,
                    BlockFace::PosY => offset.y = 1,
                    BlockFace::NegY => offset.y = -1,
                    BlockFace::PosZ => offset.z = 1,
                    BlockFace::NegZ => offset.z = -1,
                }
                let new_block_pos = selected_block_pos + offset;

                // Collision Check with player
                let player_aabb = self.player.get_world_bounding_box();
                let new_block_aabb = AABB {
                    min: new_block_pos.as_vec3(),
                    max: new_block_pos.as_vec3() + glam::Vec3::ONE, // Assuming 1x1x1 block
                };

                if player_aabb.intersects(&new_block_aabb) {
                    // eprintln!("Cannot place block: intersects with player.");
                } else {
                    match self.world.set_block(new_block_pos, BlockType::Grass) {
                        Ok(chunk_coord) => {
                            // Mark chunk as dirty by removing its render data
                            self.chunk_render_data.remove(&chunk_coord);
                            // Rebuild the potentially new chunk and its neighbors
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1);
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0 + 1, chunk_coord.1);
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0 - 1, chunk_coord.1);
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1 + 1);
                            self.build_or_rebuild_chunk_mesh(chunk_coord.0, chunk_coord.1 - 1);
                        }
                        Err(e) => {
                            eprintln!("Error placing block: {}", e);
                        }
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
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
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
            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);

            for chunk_coord in &self.active_chunk_coords {
                if let Some(render_data) = self.chunk_render_data.get(chunk_coord) {
                    if render_data.num_indices > 0 {
                        render_pass.set_vertex_buffer(0, render_data.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            render_data.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );
                        render_pass.draw_indexed(0..render_data.num_indices, 0, 0..1);
                    }
                }
            }

            if let Some((block_coords, _)) = self.selected_block {
                self.wireframe_renderer.update_model_matrix(block_coords);
                self.wireframe_renderer.draw(&mut render_pass, &self.queue);
            }
        }

        {
            let mut crosshair_render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Crosshair Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            self.crosshair.draw(&mut crosshair_render_pass);
        }

        {
            let mut debug_text_render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Debug Text Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
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
