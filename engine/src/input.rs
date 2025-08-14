use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode as VirtualKeyCode, PhysicalKey};

#[derive(Debug, Default)]
pub struct InputState {
    pub left_mouse_pressed_this_frame: bool,
    pub right_mouse_pressed_this_frame: bool,
    pub left_mouse_released_this_frame: bool,
    pub right_mouse_released_this_frame: bool,
    pub left_mouse_is_down: bool,
    pub right_mouse_is_down: bool,
    left_mouse_was_pressed_event: bool,
    right_mouse_was_pressed_event: bool,
    pub cursor_position: (f32, f32),
    pub keys_pressed_this_frame: HashSet<VirtualKeyCode>,
    pub keys_held: HashSet<VirtualKeyCode>,
    pub mouse_wheel_delta: f32,
}

impl InputState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn on_mouse_button(
        &mut self,
        button: MouseButton,
        state: ElementState,
        _inventory_open: bool,
    ) {
        let is_pressed = state == ElementState::Pressed;
        match button {
            MouseButton::Left => {
                if is_pressed && !self.left_mouse_was_pressed_event {
                    self.left_mouse_pressed_this_frame = true;
                }
                if !is_pressed && self.left_mouse_is_down {
                    self.left_mouse_released_this_frame = true;
                }
                self.left_mouse_is_down = is_pressed;
                self.left_mouse_was_pressed_event = is_pressed;
            }
            MouseButton::Right => {
                if is_pressed && !self.right_mouse_was_pressed_event {
                    self.right_mouse_pressed_this_frame = true;
                }
                if !is_pressed && self.right_mouse_is_down {
                    self.right_mouse_released_this_frame = true;
                }
                self.right_mouse_is_down = is_pressed;
                self.right_mouse_was_pressed_event = is_pressed;
            }
            _ => {}
        }
    }

    pub fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                self.mouse_wheel_delta += y;
            }
            MouseScrollDelta::PixelDelta(pos) => {
                // You might need to scale this value depending on your needs
                self.mouse_wheel_delta += pos.y as f32;
            }
        }
    }

    pub fn on_keyboard_input(&mut self, input: &KeyEvent) {
        if let PhysicalKey::Code(keycode) = input.physical_key {
            if input.state == ElementState::Pressed {
                if !self.keys_held.contains(&keycode) {
                    self.keys_pressed_this_frame.insert(keycode);
                }
                self.keys_held.insert(keycode);
            } else {
                self.keys_held.remove(&keycode);
            }
        }
    }

    pub fn on_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.cursor_position = (position.x as f32, position.y as f32);
    }

    pub fn clear_frame_state(&mut self) {
        self.left_mouse_pressed_this_frame = false;
        self.right_mouse_pressed_this_frame = false;
        self.left_mouse_released_this_frame = false;
        self.right_mouse_released_this_frame = false;
        self.keys_pressed_this_frame.clear();
        self.mouse_wheel_delta = 0.0;
    }
}
