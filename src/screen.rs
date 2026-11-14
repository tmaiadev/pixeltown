use macroquad::prelude::*;

pub const INTERNAL_HEIGHT: f32 = 480.0;

pub fn internal_width() -> f32 {
    INTERNAL_HEIGHT * screen_width() / screen_height()
}

pub fn viewport() -> Vec2 {
    vec2(internal_width(), INTERNAL_HEIGHT)
}

pub fn set_internal_camera() {
    let width = internal_width();
    set_camera(&Camera2D {
        zoom: vec2(2.0 / width, 2.0 / INTERNAL_HEIGHT),
        target: vec2(width / 2.0, INTERNAL_HEIGHT / 2.0),
        ..Default::default()
    });
}

// Converts window pixels to internal-resolution pixels.
pub fn to_internal(screen: Vec2) -> Vec2 {
    screen * (INTERNAL_HEIGHT / screen_height())
}
