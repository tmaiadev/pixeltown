use macroquad::prelude::*;

pub fn draw_text_centered(font: &Font, text: &str, center: Vec2, size: u16, color: Color) {
    let dims = measure_text(text, Some(font), size, 1.0);
    draw_text_ex(
        text,
        (center.x - dims.width / 2.0).round(),
        (center.y + dims.height / 2.0).round(),
        TextParams {
            font: Some(font),
            font_size: size,
            color,
            ..Default::default()
        },
    );
}
