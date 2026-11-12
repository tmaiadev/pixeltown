use macroquad::prelude::*;

use crate::INTERNAL_HEIGHT;

pub fn tap_position() -> Option<Vec2> {
    touches()
        .into_iter()
        .find(|t| t.phase == TouchPhase::Started)
        .map(|t| t.position * (INTERNAL_HEIGHT / screen_height()))
}
