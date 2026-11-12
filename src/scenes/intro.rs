use macroquad::prelude::*;

use super::{menu::MenuScene, Scene};
use crate::ui::text::draw_text_centered;
use crate::utils::animation::Animation;
use crate::{internal_width, INTERNAL_HEIGHT};

const LOGO_SCALE: f32 = 2.0;

const PRESENTS_FADE_IN: f32 = 1.0;
const PRESENTS_HOLD: f32 = 1.5;
const PRESENTS_FADE_OUT: f32 = 1.0;
const PRESENTS_WAIT: f32 = 0.5;

const LOGO_FADE_IN: f32 = 1.5;
const LOGO_HOLD: f32 = 2.0;
const LOGO_FADE_OUT: f32 = 1.0;

fn fade_alpha(elapsed: f32, fade_in: f32, hold: f32, fade_out: f32) -> f32 {
    if elapsed < fade_in {
        elapsed / fade_in
    } else if elapsed < fade_in + hold {
        1.0
    } else if elapsed < fade_in + hold + fade_out {
        1.0 - (elapsed - fade_in - hold) / fade_out
    } else {
        0.0
    }
}

pub struct IntroScene {
    font: Font,
    logo: Texture2D,
    presents: Animation,
    logo_anim: Animation,
}

impl IntroScene {
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
            presents: Animation::new(
                PRESENTS_FADE_IN + PRESENTS_HOLD + PRESENTS_FADE_OUT + PRESENTS_WAIT,
            )
            .autostart(),
            logo_anim: Animation::new(LOGO_FADE_IN + LOGO_HOLD + LOGO_FADE_OUT).autostart(),
        }
    }
}

impl Scene for IntroScene {
    fn update(&mut self) -> Option<Box<dyn Scene>> {
        let skip = get_last_key_pressed().is_some() || is_mouse_button_pressed(MouseButton::Left);
        let dt = get_frame_time();

        if !self.presents.has_ended() {
            self.presents.update(dt);
        } else {
            self.logo_anim.update(dt);
        }

        if skip || self.logo_anim.has_ended() {
            return Some(Box::new(MenuScene::new()));
        }
        None
    }

    fn draw(&self) {
        clear_background(BLACK);
        let center = vec2(internal_width() / 2.0, INTERNAL_HEIGHT / 2.0);

        if !self.presents.has_ended() {
            let alpha = fade_alpha(
                self.presents.elapsed(),
                PRESENTS_FADE_IN,
                PRESENTS_HOLD,
                PRESENTS_FADE_OUT,
            );
            draw_text_centered(
                &self.font,
                "@tmaiadev presents",
                center,
                16,
                Color::new(1.0, 1.0, 1.0, alpha),
            );
        } else {
            let alpha = fade_alpha(
                self.logo_anim.elapsed(),
                LOGO_FADE_IN,
                LOGO_HOLD,
                LOGO_FADE_OUT,
            );
            let size = self.logo.size() * LOGO_SCALE;
            draw_texture_ex(
                &self.logo,
                (center.x - size.x / 2.0).round(),
                (center.y - size.y / 2.0).round(),
                Color::new(1.0, 1.0, 1.0, alpha),
                DrawTextureParams {
                    dest_size: Some(size),
                    ..Default::default()
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade_alpha_ramps_holds_and_ramps_down() {
        assert_eq!(fade_alpha(0.0, 1.0, 1.0, 1.0), 0.0);
        assert_eq!(fade_alpha(0.5, 1.0, 1.0, 1.0), 0.5);
        assert_eq!(fade_alpha(1.5, 1.0, 1.0, 1.0), 1.0);
        assert_eq!(fade_alpha(2.5, 1.0, 1.0, 1.0), 0.5);
        assert_eq!(fade_alpha(3.5, 1.0, 1.0, 1.0), 0.0);
    }
}
