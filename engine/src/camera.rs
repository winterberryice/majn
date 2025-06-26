use glam::{Mat4, Vec3};

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32, // Field of view in Y, in radians
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        aspect: f32,
        fovy_degrees: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            fovy: fovy_degrees.to_radians(),
            znear,
            zfar,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

pub struct CameraMovement {
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub mouse_sensitivity: f32,
    pub speed: f32,
}

impl CameraMovement {
    pub fn new(speed: f32, mouse_sensitivity: f32) -> Self {
        Self {
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            mouse_sensitivity,
            speed,
        }
    }
}

pub struct CameraController {
    pub movement: CameraMovement,
    yaw: f32,   // Rotation around the Y axis (vertical)
    pitch: f32, // Rotation around the X axis (horizontal)
}

impl CameraController {
    pub fn new(speed: f32, mouse_sensitivity: f32) -> Self {
        Self {
            movement: CameraMovement::new(speed, mouse_sensitivity),
            yaw: -90.0f32.to_radians(), // Initialize to look along -Z
            pitch: 0.0,
        }
    }

    pub fn process_mouse_movement(&mut self, delta_x: f64, delta_y: f64) {
        let delta_x = delta_x as f32 * self.movement.mouse_sensitivity;
        let delta_y = delta_y as f32 * self.movement.mouse_sensitivity;

        self.yaw += delta_x;
        self.pitch -= delta_y;

        self.pitch = self.pitch.clamp(-89.0f32.to_radians(), 89.0f32.to_radians());
    }

    pub fn update_camera(&self, camera: &mut Camera, dt: std::time::Duration) {
        let dt_secs = dt.as_secs_f32();

        // Calculate movement direction based on current orientation BEFORE changing eye position
        let view_direction = (camera.target - camera.eye).normalize_or_zero();
        let right_direction = view_direction.cross(camera.up).normalize_or_zero();

        let mut move_vector = Vec3::ZERO;
        if self.movement.is_forward_pressed {
            move_vector += view_direction;
        }
        if self.movement.is_backward_pressed {
            move_vector -= view_direction;
        }
        if self.movement.is_left_pressed {
            move_vector -= right_direction;
        }
        if self.movement.is_right_pressed {
            move_vector += right_direction;
        }

        // Normalize combined horizontal movement vector to prevent faster diagonal movement
        if move_vector.length_squared() > 0.0 {
             camera.eye += move_vector.normalize() * self.movement.speed * dt_secs;
        }

        // Vertical movement (fly mode) - along world Y axis
        if self.movement.is_up_pressed {
            camera.eye.y += self.movement.speed * dt_secs;
        }
        if self.movement.is_down_pressed {
            camera.eye.y -= self.movement.speed * dt_secs;
        }

        // Update target based on yaw and pitch (from mouse movement)
        let new_forward_direction = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize_or_zero(); // Use normalize_or_zero for safety
        camera.target = camera.eye + new_forward_direction;
    }
}
