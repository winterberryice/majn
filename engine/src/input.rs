#[derive(Debug, Default)]
pub struct InputState {
    pub left_mouse_pressed_this_frame: bool,
    pub right_mouse_pressed_this_frame: bool,
    left_mouse_was_pressed_event: bool,
    right_mouse_was_pressed_event: bool,
    pub cursor_position: (f32, f32),
}

impl InputState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn on_mouse_input(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
        _inventory_open: bool,
    ) {
        let is_pressed = state == winit::event::ElementState::Pressed;
        match button {
            winit::event::MouseButton::Left => {
                if is_pressed && !self.left_mouse_was_pressed_event {
                    self.left_mouse_pressed_this_frame = true;
                }
                self.left_mouse_was_pressed_event = is_pressed;
            }
            winit::event::MouseButton::Right => {
                if is_pressed && !self.right_mouse_was_pressed_event {
                    self.right_mouse_pressed_this_frame = true;
                }
                self.right_mouse_was_pressed_event = is_pressed;
            }
            _ => {}
        }
    }

    pub fn on_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.cursor_position = (position.x as f32, position.y as f32);
    }

    pub fn clear_frame_state(&mut self) {
        self.left_mouse_pressed_this_frame = false;
        self.right_mouse_pressed_this_frame = false;
    }
}
