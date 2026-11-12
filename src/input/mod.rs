mod gamepad;
mod keyboard;
mod touch;

use macroquad::prelude::*;

use crate::INTERNAL_HEIGHT;
use gamepad::Gamepads;
pub use gamepad::GamepadKind;

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
    Confirm,
}

const ACTION_COUNT: usize = 3;

pub struct Input {
    gamepads: Gamepads,
    last_device: Device,
    actions: [bool; ACTION_COUNT],
    any_pressed: bool,
    pointer: Option<Vec2>,
    last_mouse: Vec2,
    pointer_moved: bool,
    pointer_clicked: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            gamepads: Gamepads::new(),
            last_device: Device::KeyboardMouse,
            actions: [false; ACTION_COUNT],
            any_pressed: false,
            pointer: None,
            last_mouse: internal_mouse_position(),
            pointer_moved: false,
            pointer_clicked: false,
        }
    }

    pub fn update(&mut self) {
        self.actions = [false; ACTION_COUNT];
        self.any_pressed = false;
        self.pointer_moved = false;
        self.pointer_clicked = false;

        let mouse = internal_mouse_position();
        self.pointer = Some(mouse);
        if mouse != self.last_mouse {
            self.last_mouse = mouse;
            self.pointer_moved = true;
            self.last_device = Device::KeyboardMouse;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            self.pointer_clicked = true;
            self.any_pressed = true;
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

        let pad = self.gamepads.poll();
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
        for action in [Action::Up, Action::Down, Action::Confirm] {
            assert!((action as usize) < ACTION_COUNT);
        }
    }
}
