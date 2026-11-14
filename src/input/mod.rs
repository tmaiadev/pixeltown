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

impl Action {
    pub const ALL: [Action; 6] = [
        Action::Up,
        Action::Down,
        Action::Left,
        Action::Right,
        Action::Confirm,
        Action::Cancel,
    ];
}

const ACTION_COUNT: usize = Action::ALL.len();

#[derive(Default)]
struct Frame {
    actions: [bool; ACTION_COUNT],
    nav_actions: [bool; ACTION_COUNT],
    any_pressed: bool,
    pointer: Option<Vec2>,
    pointer_moved: bool,
    pointer_clicked: bool,
    pan: Vec2,
    drag: Vec2,
    zoom_step: i32,
    zoom_delta: f32,
    scroll: f32,
    grid_toggled: bool,
}

pub struct Input {
    gamepads: Gamepads,
    gestures: Gestures,
    last_device: Device,
    last_mouse: Vec2,
    frame: Frame,
}

impl Input {
    pub fn new() -> Self {
        Self {
            gamepads: Gamepads::new(),
            gestures: Gestures::default(),
            last_device: Device::KeyboardMouse,
            last_mouse: internal_mouse_position(),
            frame: Frame::default(),
        }
    }

    pub fn update(&mut self) {
        self.frame = Frame::default();

        let mouse_used = self.poll_mouse();
        let keyboard_used = self.poll_keyboard();
        if mouse_used || keyboard_used {
            self.last_device = Device::KeyboardMouse;
        }
        if self.poll_touch() {
            self.last_device = Device::Touch;
        }
        if let Some(kind) = self.poll_gamepads() {
            self.last_device = Device::Gamepad(kind);
        }
    }

    fn poll_mouse(&mut self) -> bool {
        let mut used = false;
        let mouse = internal_mouse_position();
        self.frame.pointer = Some(mouse);
        if mouse != self.last_mouse {
            if is_mouse_button_down(MouseButton::Right) {
                self.frame.drag = mouse - self.last_mouse;
            }
            self.last_mouse = mouse;
            self.frame.pointer_moved = true;
            used = true;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            self.frame.pointer_clicked = true;
            self.frame.any_pressed = true;
            used = true;
        }
        let (_, wheel) = mouse_wheel();
        if wheel != 0.0 {
            let wheel = wheel.clamp(-WHEEL_LIMIT, WHEEL_LIMIT);
            self.frame.zoom_delta -= wheel * WHEEL_SENSITIVITY;
            self.frame.scroll -= wheel.signum();
            used = true;
        }
        used
    }

    fn poll_keyboard(&mut self) -> bool {
        let mut used = false;
        self.frame.grid_toggled = keyboard::grid_toggled();

        let pan = keyboard::pan();
        if pan != Vec2::ZERO {
            self.frame.pan += pan;
            used = true;
        }
        let zoom = keyboard::zoom_step();
        if zoom != 0 {
            self.frame.zoom_step += zoom;
            used = true;
        }
        for action in keyboard::pressed_nav_actions() {
            self.press_nav(action);
            used = true;
        }
        for action in keyboard::pressed_wasd_actions() {
            self.press(action);
            used = true;
        }
        if keyboard::any_key_pressed() {
            self.frame.any_pressed = true;
            used = true;
        }
        used
    }

    fn poll_touch(&mut self) -> bool {
        let mut used = false;
        if let Some(tap) = touch::tap_position() {
            self.frame.pointer = Some(tap);
            self.frame.pointer_clicked = true;
            self.frame.any_pressed = true;
            used = true;
        }
        if let Some(gesture) = self.gestures.poll() {
            self.frame.drag += gesture.drag;
            self.frame.zoom_delta += gesture.zoom;
            used = true;
        }
        used
    }

    fn poll_gamepads(&mut self) -> Option<GamepadKind> {
        let pad = self.gamepads.poll();
        self.frame.pan = (self.frame.pan + pad.pan).clamp_length_max(1.0);
        self.frame.zoom_step += pad.zoom_step;
        for action in pad.actions {
            self.press_nav(action);
        }
        if pad.any_pressed {
            self.frame.any_pressed = true;
        }
        pad.active
    }

    // WASD also pans the terrain camera, so it must not move UI focus there.
    fn press(&mut self, action: Action) {
        self.frame.actions[action as usize] = true;
        self.frame.any_pressed = true;
    }

    fn press_nav(&mut self, action: Action) {
        self.press(action);
        self.frame.nav_actions[action as usize] = true;
    }

    pub fn last_device(&self) -> Device {
        self.last_device
    }

    pub fn pressed(&self, action: Action) -> bool {
        self.frame.actions[action as usize]
    }

    pub fn pressed_nav(&self, action: Action) -> bool {
        self.frame.nav_actions[action as usize]
    }

    pub fn any_pressed(&self) -> bool {
        self.frame.any_pressed
    }

    pub fn pointer(&self) -> Option<Vec2> {
        self.frame.pointer
    }

    pub fn pointer_moved(&self) -> bool {
        self.frame.pointer_moved
    }

    pub fn pointer_clicked(&self) -> bool {
        self.frame.pointer_clicked
    }

    pub fn pan(&self) -> Vec2 {
        self.frame.pan
    }

    pub fn drag(&self) -> Vec2 {
        self.frame.drag
    }

    pub fn zoom_step(&self) -> i32 {
        self.frame.zoom_step.clamp(-1, 1)
    }

    pub fn zoom_delta(&self) -> f32 {
        self.frame.zoom_delta
    }

    // Wheel notches this frame, positive scrolls down.
    pub fn scroll(&self) -> f32 {
        self.frame.scroll
    }

    pub fn grid_toggled(&self) -> bool {
        self.frame.grid_toggled
    }
}

fn to_internal(screen: Vec2) -> Vec2 {
    screen * (INTERNAL_HEIGHT / screen_height())
}

fn internal_mouse_position() -> Vec2 {
    to_internal(mouse_position().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_discriminants_match_their_index_in_all() {
        for (i, action) in Action::ALL.into_iter().enumerate() {
            assert_eq!(action as usize, i);
        }
    }
}
