use macroquad::prelude::*;

use crate::screen::internal_width;

const SIZE: u16 = 8;
const MARGIN: f32 = 4.0;

pub fn draw_fps(font: &Font) {
    let text = format!("{} FPS", get_fps());
    let dims = measure_text(&text, Some(font), SIZE, 1.0);
    draw_text_ex(
        &text,
        (internal_width() - dims.width - MARGIN).round(),
        (MARGIN + dims.height).round(),
        TextParams {
            font: Some(font),
            font_size: SIZE,
            color: WHITE,
            ..Default::default()
        },
    );
}
