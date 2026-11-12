use gilrs::{Axis, Button, EventType, GamepadId, Gilrs};
use macroquad::prelude::{vec2, Vec2};

use super::Action;

const STICK_THRESHOLD: f32 = 0.5;
const PAN_DEADZONE: f32 = 0.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadKind {
    Nintendo,
    Xbox,
    PlayStation,
    Generic,
}

impl GamepadKind {
    fn from_name(name: &str) -> Self {
        let name = name.to_lowercase();
        let has = |needles: &[&str]| needles.iter().any(|n| name.contains(n));
        if has(&["nintendo", "switch", "joy-con"]) {
            Self::Nintendo
        } else if has(&["xbox", "x-box", "xinput"]) {
            Self::Xbox
        } else if has(&["playstation", "dualshock", "dualsense", "ps4", "ps5"]) {
            Self::PlayStation
        } else {
            Self::Generic
        }
    }
}

#[derive(Default)]
pub struct Poll {
    pub actions: Vec<Action>,
    pub any_pressed: bool,
    pub active: Option<GamepadKind>,
    pub pan: Vec2,
    pub zoom_step: i32,
}

pub struct Gamepads {
    gilrs: Option<Gilrs>,
    stick_action: Option<Action>,
    stick_x_action: Option<Action>,
    right_stick: Vec2,
}

impl Gamepads {
    pub fn new() -> Self {
        Self {
            gilrs: Gilrs::new().ok(),
            stick_action: None,
            stick_x_action: None,
            right_stick: Vec2::ZERO,
        }
    }

    pub fn poll(&mut self) -> Poll {
        let mut poll = Poll::default();
        let Some(gilrs) = self.gilrs.as_mut() else {
            return poll;
        };

        while let Some(event) = gilrs.next_event() {
            let kind = kind_of(gilrs, event.id);
            match event.event {
                EventType::ButtonPressed(button, _) => {
                    poll.any_pressed = true;
                    poll.active = Some(kind);
                    poll.actions.extend(button_action(button));
                    poll.zoom_step += button_zoom(button);
                }
                EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                    let action = stick_action(value);
                    if action.is_some() && action != self.stick_action {
                        poll.active = Some(kind);
                        poll.actions.extend(action);
                    }
                    self.stick_action = action;
                }
                EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                    let action = stick_x_action(value);
                    if action.is_some() && action != self.stick_x_action {
                        poll.active = Some(kind);
                        poll.actions.extend(action);
                    }
                    self.stick_x_action = action;
                }
                EventType::AxisChanged(axis @ (Axis::RightStickX | Axis::RightStickY), value, _) => {
                    match axis {
                        Axis::RightStickX => self.right_stick.x = value,
                        _ => self.right_stick.y = -value,
                    }
                    if apply_deadzone(self.right_stick) != Vec2::ZERO {
                        poll.active = Some(kind);
                    }
                }
                _ => {}
            }
        }
        poll.pan = apply_deadzone(self.right_stick);
        poll
    }
}

fn kind_of(gilrs: &Gilrs, id: GamepadId) -> GamepadKind {
    GamepadKind::from_name(gilrs.gamepad(id).name())
}

fn button_action(button: Button) -> Option<Action> {
    match button {
        Button::DPadUp => Some(Action::Up),
        Button::DPadDown => Some(Action::Down),
        Button::DPadLeft => Some(Action::Left),
        Button::DPadRight => Some(Action::Right),
        Button::South | Button::Start => Some(Action::Confirm),
        Button::East => Some(Action::Cancel),
        _ => None,
    }
}

fn button_zoom(button: Button) -> i32 {
    match button {
        Button::RightTrigger2 => 1,
        Button::LeftTrigger2 => -1,
        _ => 0,
    }
}

fn stick_x_action(value: f32) -> Option<Action> {
    if value > STICK_THRESHOLD {
        Some(Action::Right)
    } else if value < -STICK_THRESHOLD {
        Some(Action::Left)
    } else {
        None
    }
}

fn apply_deadzone(stick: Vec2) -> Vec2 {
    vec2(dead(stick.x), dead(stick.y))
}

fn dead(value: f32) -> f32 {
    if value.abs() < PAN_DEADZONE {
        0.0
    } else {
        value
    }
}

fn stick_action(value: f32) -> Option<Action> {
    if value > STICK_THRESHOLD {
        Some(Action::Up)
    } else if value < -STICK_THRESHOLD {
        Some(Action::Down)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_gamepad_by_name() {
        assert_eq!(
            GamepadKind::from_name("Nintendo Switch Pro Controller"),
            GamepadKind::Nintendo
        );
        assert_eq!(
            GamepadKind::from_name("Xbox Wireless Controller"),
            GamepadKind::Xbox
        );
        assert_eq!(
            GamepadKind::from_name("DualSense Wireless Controller"),
            GamepadKind::PlayStation
        );
        assert_eq!(GamepadKind::from_name("USB Gamepad"), GamepadKind::Generic);
    }

    #[test]
    fn maps_buttons_and_stick_to_actions() {
        assert_eq!(button_action(Button::DPadUp), Some(Action::Up));
        assert_eq!(button_action(Button::South), Some(Action::Confirm));
        assert_eq!(button_action(Button::North), None);
        assert_eq!(stick_action(0.9), Some(Action::Up));
        assert_eq!(stick_action(-0.9), Some(Action::Down));
        assert_eq!(stick_action(0.1), None);
    }

    #[test]
    fn maps_horizontal_navigation_and_cancel() {
        assert_eq!(button_action(Button::DPadLeft), Some(Action::Left));
        assert_eq!(button_action(Button::DPadRight), Some(Action::Right));
        assert_eq!(button_action(Button::East), Some(Action::Cancel));
        assert_eq!(stick_x_action(0.9), Some(Action::Right));
        assert_eq!(stick_x_action(-0.9), Some(Action::Left));
        assert_eq!(stick_x_action(0.2), None);
    }

    #[test]
    fn triggers_zoom_and_small_stick_drift_is_ignored() {
        assert_eq!(button_zoom(Button::RightTrigger2), 1);
        assert_eq!(button_zoom(Button::LeftTrigger2), -1);
        assert_eq!(button_zoom(Button::South), 0);
        assert_eq!(apply_deadzone(vec2(0.1, -0.05)), Vec2::ZERO);
        assert_eq!(apply_deadzone(vec2(0.8, 0.1)), vec2(0.8, 0.0));
    }
}
