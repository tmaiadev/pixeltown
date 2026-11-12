use macroquad::prelude::*;

use super::{gameplay::GameplayScene, Scene};
use crate::ui::text::draw_text_centered;
use crate::utils::animation::Animation;
use crate::{internal_width, INTERNAL_HEIGHT};

const FADE_IN: f32 = 1.0;
const LOGO_SCALE: f32 = 2.0;
const LOGO_TOP: f32 = 80.0;
const ITEMS_TOP: f32 = 290.0;
const ITEM_SPACING: f32 = 32.0;
const ITEM_HALF_WIDTH: f32 = 140.0;
const ITEM_SIZE: u16 = 16;
const CREDIT_SIZE: u16 = 8;
const CREDIT_BOTTOM_MARGIN: f32 = 24.0;
const CURSOR_OFFSET: f32 = 24.0;

const ITEMS: [&str; 4] = ["Start New Game", "Load Game", "Options", "Exit"];

fn wrap_selection(current: usize, delta: isize, len: usize) -> usize {
    (current as isize + delta).rem_euclid(len as isize) as usize
}

fn item_y(index: usize) -> f32 {
    ITEMS_TOP + index as f32 * ITEM_SPACING
}

fn item_at(point: Vec2, center_x: f32) -> Option<usize> {
    if (point.x - center_x).abs() > ITEM_HALF_WIDTH {
        return None;
    }
    (0..ITEMS.len()).find(|&i| (point.y - item_y(i)).abs() <= ITEM_SPACING / 2.0)
}

fn internal_mouse_position() -> Vec2 {
    let (x, y) = mouse_position();
    vec2(x, y) * (INTERNAL_HEIGHT / screen_height())
}

pub struct MenuScene {
    font: Font,
    logo: Texture2D,
    fade: Animation,
    selected: usize,
    last_mouse: Vec2,
}

impl MenuScene {
    pub fn new() -> Self {
        let mut font =
            load_ttf_font_from_bytes(include_bytes!("../../assets/fonts/PressStart2P.ttf"))
                .expect("valid font");
        font.set_filter(FilterMode::Nearest);

        let logo = Texture2D::from_file_with_format(
            include_bytes!("../../assets/sprites/logo.png"),
            Some(ImageFormat::Png),
        );
        logo.set_filter(FilterMode::Nearest);

        Self {
            font,
            logo,
            fade: Animation::new(FADE_IN).autostart(),
            selected: 0,
            last_mouse: internal_mouse_position(),
        }
    }

    fn activate(&self) -> Option<Box<dyn Scene>> {
        match self.selected {
            0 => Some(Box::new(GameplayScene)),
            3 => std::process::exit(0),
            _ => None,
        }
    }
}

impl Scene for MenuScene {
    fn update(&mut self) -> Option<Box<dyn Scene>> {
        self.fade.update(get_frame_time());
        if !self.fade.has_ended() {
            return None;
        }

        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.selected = wrap_selection(self.selected, -1, ITEMS.len());
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.selected = wrap_selection(self.selected, 1, ITEMS.len());
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            return self.activate();
        }

        let mouse = internal_mouse_position();
        let hovered = item_at(mouse, internal_width() / 2.0);
        if mouse != self.last_mouse {
            self.last_mouse = mouse;
            if let Some(i) = hovered {
                self.selected = i;
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) && hovered.is_some() {
            return self.activate();
        }
        None
    }

    fn draw(&self) {
        clear_background(BLACK);
        let center_x = internal_width() / 2.0;
        let alpha = self.fade.progress();
        let tint = Color::new(1.0, 1.0, 1.0, alpha);

        let size = self.logo.size() * LOGO_SCALE;
        draw_texture_ex(
            &self.logo,
            (center_x - size.x / 2.0).round(),
            LOGO_TOP,
            tint,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );

        for (i, label) in ITEMS.iter().enumerate() {
            let y = item_y(i);
            draw_text_centered(&self.font, label, vec2(center_x, y), ITEM_SIZE, tint);
            if i == self.selected {
                let width = measure_text(label, Some(&self.font), ITEM_SIZE, 1.0).width;
                let cursor_x = center_x - width / 2.0 - CURSOR_OFFSET;
                draw_text_centered(&self.font, ">", vec2(cursor_x, y), ITEM_SIZE, tint);
            }
        }

        draw_text_centered(
            &self.font,
            "created by @tmaiadev",
            vec2(center_x, INTERNAL_HEIGHT - CREDIT_BOTTOM_MARGIN),
            CREDIT_SIZE,
            tint,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_selection_wraps_in_both_directions() {
        assert_eq!(wrap_selection(0, -1, 4), 3);
        assert_eq!(wrap_selection(3, 1, 4), 0);
        assert_eq!(wrap_selection(1, 1, 4), 2);
    }

    #[test]
    fn item_at_hits_rows_and_misses_outside() {
        assert_eq!(item_at(vec2(100.0, item_y(2)), 100.0), Some(2));
        assert_eq!(item_at(vec2(100.0, item_y(0) - ITEM_SPACING), 100.0), None);
        assert_eq!(
            item_at(vec2(100.0 + ITEM_HALF_WIDTH + 1.0, item_y(1)), 100.0),
            None
        );
    }
}
