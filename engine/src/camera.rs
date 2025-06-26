use glam::Mat4;

// We'll also define our Uniform struct here for now.
// It needs to be `repr(C)` to ensure predictable memory layout for the shader.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // Store as [[f32; 4]; 4] for bytemuck compatibility
    pub view_proj: [[f32; 4]; 4], // Made public
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    // This method is no longer called directly by State::update in the new setup,
    // as State::update now calculates the view_proj matrix itself and assigns it
    // to camera_uniform.view_proj directly.
    // However, it might be useful if CameraUniform were to be updated from a
    // generic matrix elsewhere, or if we re-introduce a lightweight Camera struct.
    // For now, it's unused by the main loop.
    // pub fn update_view_proj_from_matrix(&mut self, view_proj_matrix: Mat4) {
    //     self.view_proj = view_proj_matrix.to_cols_array_2d();
    // }

    // The old update_view_proj that took a &Camera is no longer relevant
    // as the Camera struct itself is being removed from active use in State.
}
