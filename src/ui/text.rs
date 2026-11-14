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

// Greedy word wrap; a word wider than `max_width` gets its own line.
pub fn wrap(text: &str, max_width: f32, measure: impl Fn(&str) -> f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if measure(&format!("{line} {word}")) <= max_width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_breaks_between_words() {
        let measure = |t: &str| t.len() as f32;
        assert_eq!(wrap("delete this save?", 11.0, measure), ["delete this", "save?"]);
        assert_eq!(wrap("delete this save?", 100.0, measure), ["delete this save?"]);
        assert_eq!(wrap("unbreakable a", 4.0, measure), ["unbreakable", "a"]);
        assert!(wrap("", 10.0, measure).is_empty());
    }
}
