use macroquad::prelude::*;

pub fn draw_texture_scaled(texture: &Texture2D, top_left: Vec2, scale: f32, tint: Color) {
    draw_texture_ex(
        texture,
        top_left.x,
        top_left.y,
        tint,
        DrawTextureParams {
            dest_size: Some(texture.size() * scale),
            ..Default::default()
        },
    );
}
