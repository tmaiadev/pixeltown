use macroquad::prelude::*;

use super::spritesheet::Spritesheet;

pub fn load_font() -> Font {
    let mut font = load_ttf_font_from_bytes(include_bytes!("../../assets/fonts/PressStart2P.ttf"))
        .expect("valid font");
    font.set_filter(FilterMode::Nearest);
    font
}

pub fn load_texture(png: &[u8]) -> Texture2D {
    let texture = Texture2D::from_file_with_format(png, Some(ImageFormat::Png));
    texture.set_filter(FilterMode::Nearest);
    texture
}

pub fn load_logo() -> Texture2D {
    load_texture(include_bytes!("../../assets/sprites/logo.png"))
}

pub fn load_icons() -> Spritesheet {
    Spritesheet::from_png(include_bytes!("../../assets/sprites/icons.png"), 16.0)
}
