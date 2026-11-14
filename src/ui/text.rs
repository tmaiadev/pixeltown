use macroquad::prelude::*;

pub fn draw_text_centered(font: &Font, text: &str, center: Vec2, size: u16, color: Color) {
    let width = measure_text(text, Some(font), size, 1.0).width;
    draw_text_left(font, text, vec2(center.x - width / 2.0, center.y), size, color);
}

// `anchor` is the right edge, vertically centered on the text.
pub fn draw_text_right(font: &Font, text: &str, anchor: Vec2, size: u16, color: Color) {
    let width = measure_text(text, Some(font), size, 1.0).width;
    draw_text_left(font, text, vec2(anchor.x - width, anchor.y), size, color);
}

// `anchor` is the left edge, vertically centered on the text.
pub fn draw_text_left(font: &Font, text: &str, anchor: Vec2, size: u16, color: Color) {
    let dims = measure_text(text, Some(font), size, 1.0);
    draw_text_ex(
        text,
        anchor.x.round(),
        (anchor.y + dims.height / 2.0).round(),
        TextParams {
            font: Some(font),
            font_size: size,
            color,
            ..Default::default()
        },
    );
}

pub fn truncate(font: &Font, text: &str, size: u16, max_width: f32) -> String {
    let fits = |t: &str| measure_text(t, Some(font), size, 1.0).width <= max_width;
    if fits(text) {
        return text.to_owned();
    }
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let shortened = format!("{}..", chars.iter().collect::<String>());
        if fits(&shortened) {
            return shortened;
        }
    }
    String::new()
}
