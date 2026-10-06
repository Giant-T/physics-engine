use nalgebra::Vector2;
use winit::keyboard::KeyCode;

pub struct InputState {
    mouse_delta: Vector2<f32>,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_space_pressed: bool,
    is_ctrl_pressed: bool,
}

impl InputState {
    pub fn reset_mouse_delta(&mut self) {
        self.mouse_delta = Vector2::zeros();
    }

    pub fn update_keys(&mut self, code: KeyCode, is_pressed: bool) {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
            }
            KeyCode::Space => {
                self.is_space_pressed = is_pressed;
            }
            KeyCode::ControlLeft => {
                self.is_ctrl_pressed = is_pressed;
            }
            _ => {}
        }
    }

    pub fn mouse_delta(&self) -> Vector2<f32> {
        self.mouse_delta
    }

    pub fn is_forward_pressed(&self) -> bool {
        self.is_forward_pressed
    }

    pub fn is_backward_pressed(&self) -> bool {
        self.is_backward_pressed
    }

    pub fn is_left_pressed(&self) -> bool {
        self.is_left_pressed
    }

    pub fn is_right_pressed(&self) -> bool {
        self.is_right_pressed
    }

    pub fn is_space_pressed(&self) -> bool {
        self.is_space_pressed
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.is_ctrl_pressed
    }

    pub fn handle_mouse_move(&mut self, movement: Vector2<f32>) {
        self.mouse_delta += movement;
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_delta: Vector2::zeros(),
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_space_pressed: false,
            is_ctrl_pressed: false,
        }
    }
}
