use macroquad::prelude::*;

use super::Action;

const NAV_BINDINGS: [(KeyCode, Action); 7] = [
    (KeyCode::Up, Action::Up),
    (KeyCode::Down, Action::Down),
    (KeyCode::Left, Action::Left),
    (KeyCode::Right, Action::Right),
    (KeyCode::Enter, Action::Confirm),
    (KeyCode::Space, Action::Confirm),
    (KeyCode::Escape, Action::Cancel),
];

const WASD_BINDINGS: [(KeyCode, Action); 2] = [(KeyCode::W, Action::Up), (KeyCode::S, Action::Down)];

const ZOOM_IN_KEYS: [KeyCode; 2] = [KeyCode::Equal, KeyCode::KpAdd];
const ZOOM_OUT_KEYS: [KeyCode; 2] = [KeyCode::Minus, KeyCode::KpSubtract];

pub fn pressed_nav_actions() -> impl Iterator<Item = Action> {
    let tab = is_key_pressed(KeyCode::Tab).then(|| {
        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
            Action::Left
        } else {
            Action::Right
        }
    });

    pressed(NAV_BINDINGS).chain(tab)
}

pub fn pressed_wasd_actions() -> impl Iterator<Item = Action> {
    pressed(WASD_BINDINGS)
}

fn pressed<const N: usize>(bindings: [(KeyCode, Action); N]) -> impl Iterator<Item = Action> {
    bindings
        .into_iter()
        .filter(|(key, _)| is_key_pressed(*key))
        .map(|(_, action)| action)
}

pub fn any_key_pressed() -> bool {
    get_last_key_pressed().is_some()
}

pub fn pan() -> Vec2 {
    vec2(
        axis(is_key_down(KeyCode::A), is_key_down(KeyCode::D)),
        axis(is_key_down(KeyCode::W), is_key_down(KeyCode::S)),
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
    fn nav_bindings_exclude_wasd() {
        for (key, _) in NAV_BINDINGS {
            assert!(![KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D].contains(&key));
        }
        for (key, _) in WASD_BINDINGS {
            assert!(!NAV_BINDINGS.iter().any(|(nav, _)| *nav == key));
        }
    }

    #[test]
    fn zoom_keys_resolve_to_a_single_step() {
        assert_eq!(zoom_direction(true, false), 1);
        assert_eq!(zoom_direction(false, true), -1);
        assert_eq!(zoom_direction(true, true), 0);
    }
}
