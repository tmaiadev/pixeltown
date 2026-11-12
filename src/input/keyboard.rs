use macroquad::prelude::*;

use super::Action;

const BINDINGS: [(KeyCode, Action); 6] = [
    (KeyCode::Up, Action::Up),
    (KeyCode::W, Action::Up),
    (KeyCode::Down, Action::Down),
    (KeyCode::S, Action::Down),
    (KeyCode::Enter, Action::Confirm),
    (KeyCode::Space, Action::Confirm),
];

pub fn pressed_actions() -> impl Iterator<Item = Action> {
    BINDINGS
        .into_iter()
        .filter(|(key, _)| is_key_pressed(*key))
        .map(|(_, action)| action)
}

pub fn any_key_pressed() -> bool {
    get_last_key_pressed().is_some()
}
