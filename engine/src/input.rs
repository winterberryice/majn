#[derive(Debug, Default)]
pub struct InputState {
    pub left_mouse_pressed_this_frame: bool,
    pub right_mouse_pressed_this_frame: bool,
    // Internal state to track if a button was pressed in the *previous* frame's event processing pass.
    // This helps ensure one event translates to one action, even if events arrive faster than frames.
    left_mouse_was_pressed_event: bool,
    right_mouse_was_pressed_event: bool,
}

impl InputState {
    pub fn new() -> Self {
        Default::default()
    }

    // Called from the winit event loop for MouseInput events
    pub fn on_mouse_input(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
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

    // Called at the end of each frame/update cycle to reset the per-frame flags.
    pub fn clear_frame_state(&mut self) {
        self.left_mouse_pressed_this_frame = false;
        self.right_mouse_pressed_this_frame = false;
    }

    // Optional: if we want to reset the "was_pressed_event" on focus loss or similar
    // pub fn reset_all_presses(&mut self) {
    //     self.left_mouse_pressed_this_frame = false;
    //     self.right_mouse_pressed_this_frame = false;
    //     self.left_mouse_was_pressed_event = false;
    //     self.right_mouse_was_pressed_event = false;
    // }
}
