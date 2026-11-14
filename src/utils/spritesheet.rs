use macroquad::prelude::*;

use super::assets::load_texture;

pub struct Spritesheet {
    texture: Texture2D,
    tile_size: f32,
}

impl Spritesheet {
    pub fn from_png(bytes: &[u8], tile_size: f32) -> Self {
        Self {
            texture: load_texture(bytes),
            tile_size,
        }
    }

    pub fn texture(&self) -> &Texture2D {
        &self.texture
    }

    pub fn tile_size(&self) -> f32 {
        self.tile_size
    }

    pub fn source(tile_size: f32, col: usize, row: usize) -> Rect {
        Rect::new(
            col as f32 * tile_size,
            row as f32 * tile_size,
            tile_size,
            tile_size,
        )
    }

    pub fn draw_tile(&self, col: usize, row: usize, x: f32, y: f32) {
        self.draw_tile_tinted(col, row, x, y, WHITE);
    }

    pub fn draw_tile_tinted(&self, col: usize, row: usize, x: f32, y: f32, tint: Color) {
        draw_texture_ex(
            &self.texture,
            x,
            y,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(self.tile_size, self.tile_size)),
                source: Some(Self::source(self.tile_size, col, row)),
                ..Default::default()
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_is_the_tile_rect_at_col_and_row() {
        assert_eq!(Spritesheet::source(16.0, 0, 0), Rect::new(0.0, 0.0, 16.0, 16.0));
        assert_eq!(Spritesheet::source(16.0, 3, 2), Rect::new(48.0, 32.0, 16.0, 16.0));
    }
}
