use macroquad::prelude::*;

use super::Action;

const BINDINGS: [(KeyCode, Action); 7] = [
    (KeyCode::Up, Action::Up),
    (KeyCode::W, Action::Up),
    (KeyCode::Down, Action::Down),
    (KeyCode::S, Action::Down),
    (KeyCode::Enter, Action::Confirm),
    (KeyCode::Space, Action::Confirm),
    (KeyCode::Escape, Action::Cancel),
];

const ZOOM_IN_KEYS: [KeyCode; 2] = [KeyCode::Equal, KeyCode::KpAdd];
const ZOOM_OUT_KEYS: [KeyCode; 2] = [KeyCode::Minus, KeyCode::KpSubtract];

pub fn pressed_actions() -> impl Iterator<Item = Action> {
    let tab = is_key_pressed(KeyCode::Tab).then(|| {
        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
            Action::Left
        } else {
            Action::Right
        }
    });

    BINDINGS
        .into_iter()
        .filter(|(key, _)| is_key_pressed(*key))
        .map(|(_, action)| action)
        .chain(tab)
}

pub fn any_key_pressed() -> bool {
    get_last_key_pressed().is_some()
}

pub fn pan() -> Vec2 {
    let down = |keys: [KeyCode; 2]| keys.into_iter().any(is_key_down);
    vec2(
        axis(down([KeyCode::A, KeyCode::Left]), down([KeyCode::D, KeyCode::Right])),
        axis(down([KeyCode::W, KeyCode::Up]), down([KeyCode::S, KeyCode::Down])),
    )
}

pub fn zoom_step() -> i32 {
    let pressed = |keys: [KeyCode; 2]| keys.into_iter().any(is_key_pressed);
    zoom_direction(pressed(ZOOM_IN_KEYS), pressed(ZOOM_OUT_KEYS))
}

fn axis(negative: bool, positive: bool) -> f32 {
    positive as i32 as f32 - negative as i32 as f32
}

fn zoom_direction(zoom_in: bool, zoom_out: bool) -> i32 {
    zoom_in as i32 - zoom_out as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposing_keys_cancel_out() {
        assert_eq!(axis(true, false), -1.0);
        assert_eq!(axis(false, true), 1.0);
        assert_eq!(axis(true, true), 0.0);
        assert_eq!(axis(false, false), 0.0);
    }

    #[test]
    fn zoom_keys_resolve_to_a_single_step() {
        assert_eq!(zoom_direction(true, false), 1);
        assert_eq!(zoom_direction(false, true), -1);
        assert_eq!(zoom_direction(true, true), 0);
    }
}
