mod block;
mod chunk;
mod cube_geometry;
mod camera;
mod instance;

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
// use crate::cube_geometry; // Removed: `mod cube_geometry;` makes it available.
use crate::camera::{Camera, CameraController, CameraUniform}; // Import Camera, CameraController, and CameraUniform
use crate::chunk::Chunk; // Import Chunk
use crate::instance::InstanceRaw; // Import InstanceRaw

// The State struct holds all of our wgpu-related objects.
struct State { // Removed lifetime 'a
    surface: wgpu::Surface<'static>, // Changed to 'static
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    // vertex_buffer: wgpu::Buffer, // Old triangle vertex buffer
    // num_vertices: u32,           // Old number of triangle vertices

    cube_vertex_buffer: wgpu::Buffer,
    cube_index_buffer: wgpu::Buffer,
    num_cube_indices: u32,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController, // Ensuring this field is present

    chunk: Chunk, // Add a chunk
    instance_data: Vec<InstanceRaw>,
    instance_buffer: wgpu::Buffer,

    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    // window: Arc<Window>, // Removed: No longer needed in State
}

impl State { // Removed lifetime 'a
    async fn new(window_surface_target: Arc<Window>, initial_size: winit::dpi::PhysicalSize<u32>) -> Self { // Takes Arc<Window> for surface, initial_size
        // let size = window.inner_size(); // Now passed as initial_size

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface using Arc<Window> for Surface<'static>
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
                    // API Change: The fields were renamed from 'features' and 'limits'
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: wgpu::MemoryHints::default(),
                    trace: Trace::Off, // Guessed variant Trace::Off
                },
                // None, // trace_path was removed
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0]; // Choose a supported format

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: initial_size.width, // Use initial_size
            height: initial_size.height, // Use initial_size
            present_mode: wgpu::PresentMode::Fifo, // V-sync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            // API Change: This new field is required
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create camera bind group layout FIRST
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
                bind_group_layouts: &[&camera_bind_group_layout], // USE THE CAMERA BIND GROUP LAYOUT
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout), // This now uses the layout with the camera BGL
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceRaw::desc()], // Add InstanceRaw desc
                // API Change: This new field is required
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
                // API Change: This new field is required
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // Counter-clockwise triangles are front-facing
                cull_mode: Some(wgpu::Face::Back), // Cull back-facing triangles
                // Setting this to None requires Features::POLYGON_MODE_LINE or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float, // Must match depth_texture format
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // Standard depth comparison
                stencil: wgpu::StencilState::default(), // No stencil for now
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            // API Change: This new field is required
            cache: None,
        });

        use wgpu::util::DeviceExt;
        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(VERTICES), // VERTICES is commented out
        //     usage: wgpu::BufferUsages::VERTEX,
        // });
        // let num_vertices = VERTICES.len() as u32; // VERTICES is commented out

        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(cube_geometry::cube_vertices()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let cube_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(cube_geometry::cube_indices()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_cube_indices = cube_geometry::cube_indices().len() as u32;

        let camera = Camera::new(
            // Adjust camera position for better view of the 16x32x16 chunk
            glam::Vec3::new(
                0.0,
                crate::chunk::CHUNK_HEIGHT as f32 / 1.5, // Higher up
                crate::chunk::CHUNK_DEPTH as f32 * 2.0 // Further back
            ),
            glam::Vec3::new(0.0, crate::chunk::CHUNK_HEIGHT as f32 / 2.0 - 5.0 , 0.0), // Target: look towards the center of the chunk mass
            glam::Vec3::Y,                  // up: standard Y up
            config.width as f32 / config.height as f32,
            45.0, // fovy_degrees
            0.1,  // znear
            100.0, // zfar
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        // Set mouse_sensitivity to 0.0 to disable mouse camera control
        let camera_controller = CameraController::new(10.0, 0.0);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // camera_bind_group_layout is now created earlier, before render_pipeline_layout
        // let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }
        //     ],
        //     label: Some("camera_bind_group_layout"),
        // });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout, // This refers to the earlier created layout
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let mut chunk = Chunk::new();
        chunk.generate_terrain(); // Populate with dirt and grass

        // Initial instance data and buffer
        // Max instances = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH
        let max_instances = crate::chunk::CHUNK_WIDTH * crate::chunk::CHUNK_HEIGHT * crate::chunk::CHUNK_DEPTH;
        // Ensure instance_data Vec has enough capacity
        let instance_data: Vec<InstanceRaw> = Vec::with_capacity(max_instances);

        let instance_buffer_size = (max_instances * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: instance_buffer_size, // Initial size, can be larger if needed or resized
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false, // Data will be copied via queue.write_buffer
        });

        let depth_texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float, // Or Depth24PlusStencil8, etc.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // Removed | wgpu::TextureUsages::TEXTURE_BINDING as not currently sampled
            label: Some("Depth Texture"),
            view_formats: &[],
        };
        let depth_texture = device.create_texture(&depth_texture_desc);
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            surface,
            device,
            queue,
            config,
            size: initial_size, // Store initial_size
            render_pipeline,
            // vertex_buffer,
            // num_vertices,
            cube_vertex_buffer,
            cube_index_buffer,
            num_cube_indices,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            chunk,
            instance_data,
            instance_buffer,
            depth_texture,
            depth_texture_view,
            // window, // Removed
        }
    }

    // Removed: pub fn window(&self) -> &Window
    // State no longer directly holds the window Arc. App does.

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.camera.aspect = self.config.width as f32 / self.config.height as f32; // Update camera aspect ratio

            // Recreate depth texture for new size
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
        }
    }

    // Method to specifically handle mouse motion
    pub fn process_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        // Access camera_controller from State
        if self.camera_controller.movement.mouse_sensitivity > 0.0 {
            self.camera_controller.process_mouse_movement(delta_x, delta_y);
        }
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
                        self.camera_controller.movement.is_forward_pressed = is_pressed;
                        true // Event handled
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.camera_controller.movement.is_backward_pressed = is_pressed;
                        true // Event handled
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.camera_controller.movement.is_left_pressed = is_pressed;
                        true // Event handled
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.camera_controller.movement.is_right_pressed = is_pressed;
                        true // Event handled
                    }
                    KeyCode::Space => {
                        self.camera_controller.movement.is_up_pressed = is_pressed;
                        true // Event handled
                    }
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                        self.camera_controller.movement.is_down_pressed = is_pressed;
                        true // Event handled
                    }
                    // Let App handle Escape for mouse grab toggle / exit
                    KeyCode::Escape => false, // Event not handled by State, let App handle it
                    _ => false, // Event not handled
                }
            }
            // CursorMoved is now handled by App, which calls state.process_mouse_motion directly.
            // So, we don't need to handle CursorMoved here in state.input for camera purposes.
            // WindowEvent::CursorMoved { .. } => {
            //     true // Indicate event was seen, even if handled by App calling process_mouse_motion
            // }
            _ => false, // Event not handled by State's general input processing
        }
    }

    fn update(&mut self) {
        // Update camera based on controller
        // For dt, we'd ideally pass the actual delta time from the game loop.
        // For now, let's use a fixed placeholder or calculate it if possible.
        // If App::about_to_wait is called regularly (e.g., for each frame),
        // we could store a `last_update_time` in `State` and calculate `dt`.
        // For simplicity here, let's assume a fixed `dt` for now, e.g., 1/60th of a second.
        let dt = std::time::Duration::from_secs_f32(1.0 / 60.0); // Placeholder
        self.camera_controller.update_camera(&mut self.camera, dt);

        // Update camera UBO
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        // Generate instance data from chunk
        self.instance_data.clear();
        let chunk_offset = glam::Vec3::new(
            -(crate::chunk::CHUNK_WIDTH as f32 / 2.0),
            0.0, // Or -(crate::chunk::CHUNK_HEIGHT as f32 / 2.0) if you want to center Y
            -(crate::chunk::CHUNK_DEPTH as f32 / 2.0)
        );

        for x in 0..crate::chunk::CHUNK_WIDTH {
            for y in 0..crate::chunk::CHUNK_HEIGHT {
                for z in 0..crate::chunk::CHUNK_DEPTH {
                    if let Some(block) = self.chunk.get_block(x, y, z) {
                        if block.is_solid() { // Only render solid blocks
                            let position = glam::Vec3::new(x as f32, y as f32, z as f32) + chunk_offset;
                            let model_matrix = glam::Mat4::from_translation(position);

                            // Determine color based on block type
                            let color = match block.block_type {
                                crate::block::BlockType::Dirt => [0.5, 0.25, 0.05], // Brown
                                crate::block::BlockType::Grass => [0.0, 0.8, 0.1],  // Green
                                crate::block::BlockType::Air => [0.0, 0.0, 0.0],    // Should not happen due to is_solid
                                // Add other block types here
                            };
                            self.instance_data.push(InstanceRaw::new(model_matrix, color));
                        }
                    }
                }
            }
        }
        if !self.instance_data.is_empty() {
            self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&self.instance_data));
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                        load: wgpu::LoadOp::Clear(1.0), // Clear depth to 1.0 (far plane)
                        store: wgpu::StoreOp::Store, // Store the depth buffer
                    }),
                    stencil_ops: None, // No stencil operations for now
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]); // Set camera UBO at group 0
            render_pass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..)); // Per-vertex data
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..)); // Per-instance data
            render_pass.set_index_buffer(self.cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            if self.instance_data.is_empty() {
                // If there's nothing to draw, don't call draw_indexed.
                // This can happen if the chunk is all air or if instance_data failed to populate.
            } else {
                render_pass.draw_indexed(0..self.num_cube_indices, 0, 0..self.instance_data.len() as u32);
            }
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