mod gamepad;
mod keyboard;
mod touch;

use macroquad::prelude::*;

use crate::INTERNAL_HEIGHT;
use gamepad::Gamepads;
pub use gamepad::GamepadKind;
use touch::Gestures;

const WHEEL_LIMIT: f32 = 3.0;
const WHEEL_SENSITIVITY: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    KeyboardMouse,
    Touch,
    Gamepad(GamepadKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
}

const ACTION_COUNT: usize = 6;

pub struct Input {
    gamepads: Gamepads,
    gestures: Gestures,
    last_device: Device,
    actions: [bool; ACTION_COUNT],
    any_pressed: bool,
    pointer: Option<Vec2>,
    last_mouse: Vec2,
    pointer_moved: bool,
    pointer_clicked: bool,
    pan: Vec2,
    drag: Vec2,
    zoom_step: i32,
    zoom_delta: f32,
}

impl Input {
    pub fn new() -> Self {
        Self {
            gamepads: Gamepads::new(),
            gestures: Gestures::default(),
            last_device: Device::KeyboardMouse,
            actions: [false; ACTION_COUNT],
            any_pressed: false,
            pointer: None,
            last_mouse: internal_mouse_position(),
            pointer_moved: false,
            pointer_clicked: false,
            pan: Vec2::ZERO,
            drag: Vec2::ZERO,
            zoom_step: 0,
            zoom_delta: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.actions = [false; ACTION_COUNT];
        self.any_pressed = false;
        self.pointer_moved = false;
        self.pointer_clicked = false;
        self.pan = Vec2::ZERO;
        self.drag = Vec2::ZERO;
        self.zoom_step = 0;
        self.zoom_delta = 0.0;

        let mouse = internal_mouse_position();
        self.pointer = Some(mouse);
        if mouse != self.last_mouse {
            if is_mouse_button_down(MouseButton::Right) {
                self.drag = mouse - self.last_mouse;
            }
            self.last_mouse = mouse;
            self.pointer_moved = true;
            self.last_device = Device::KeyboardMouse;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            self.pointer_clicked = true;
            self.any_pressed = true;
            self.last_device = Device::KeyboardMouse;
        }

        let (_, wheel) = mouse_wheel();
        if wheel != 0.0 {
            self.zoom_delta += wheel.clamp(-WHEEL_LIMIT, WHEEL_LIMIT) * WHEEL_SENSITIVITY;
            self.last_device = Device::KeyboardMouse;
        }

        let key_pan = keyboard::pan();
        if key_pan != Vec2::ZERO {
            self.pan += key_pan;
            self.last_device = Device::KeyboardMouse;
        }
        let key_zoom = keyboard::zoom_step();
        if key_zoom != 0 {
            self.zoom_step += key_zoom;
            self.last_device = Device::KeyboardMouse;
        }

        for action in keyboard::pressed_actions() {
            self.press(action);
            self.last_device = Device::KeyboardMouse;
        }
        if keyboard::any_key_pressed() {
            self.any_pressed = true;
            self.last_device = Device::KeyboardMouse;
        }

        if let Some(tap) = touch::tap_position() {
            self.pointer = Some(tap);
            self.pointer_clicked = true;
            self.any_pressed = true;
            self.last_device = Device::Touch;
        }

        if let Some(gesture) = self.gestures.poll() {
            self.drag += gesture.drag;
            self.zoom_delta += gesture.zoom;
            self.last_device = Device::Touch;
        }

        let pad = self.gamepads.poll();
        self.pan += pad.pan;
        self.zoom_step += pad.zoom_step;
        self.pan = self.pan.clamp_length_max(1.0);
        for action in pad.actions {
            self.press(action);
        }
        if pad.any_pressed {
            self.any_pressed = true;
        }
        if let Some(kind) = pad.active {
            self.last_device = Device::Gamepad(kind);
        }
    }

    fn press(&mut self, action: Action) {
        self.actions[action as usize] = true;
        self.any_pressed = true;
    }

    pub fn last_device(&self) -> Device {
        self.last_device
    }

    pub fn pressed(&self, action: Action) -> bool {
        self.actions[action as usize]
    }

    pub fn any_pressed(&self) -> bool {
        self.any_pressed
    }

    pub fn pointer(&self) -> Option<Vec2> {
        self.pointer
    }

    pub fn pointer_moved(&self) -> bool {
        self.pointer_moved
    }

    pub fn pointer_clicked(&self) -> bool {
        self.pointer_clicked
    }

    pub fn pan(&self) -> Vec2 {
        self.pan
    }

    pub fn drag(&self) -> Vec2 {
        self.drag
    }

    pub fn zoom_step(&self) -> i32 {
        self.zoom_step.clamp(-1, 1)
    }

    pub fn zoom_delta(&self) -> f32 {
        self.zoom_delta
    }
}

fn internal_mouse_position() -> Vec2 {
    let (x, y) = mouse_position();
    vec2(x, y) * (INTERNAL_HEIGHT / screen_height())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_indices_fit_in_state_array() {
        for action in [
            Action::Up,
            Action::Down,
            Action::Left,
            Action::Right,
            Action::Confirm,
            Action::Cancel,
        ] {
            assert!((action as usize) < ACTION_COUNT);
        }
    }
}
