use gilrs::{Axis, Button, EventType, GamepadId, Gilrs};

use super::Action;

const STICK_THRESHOLD: f32 = 0.5;

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
}

pub struct Gamepads {
    gilrs: Option<Gilrs>,
    stick_action: Option<Action>,
}

impl Gamepads {
    pub fn new() -> Self {
        Self {
            gilrs: Gilrs::new().ok(),
            stick_action: None,
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
                }
                EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                    let action = stick_action(value);
                    if action.is_some() && action != self.stick_action {
                        poll.active = Some(kind);
                        poll.actions.extend(action);
                    }
                    self.stick_action = action;
                }
                _ => {}
            }
        }
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
        Button::South | Button::Start => Some(Action::Confirm),
        _ => None,
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
}
