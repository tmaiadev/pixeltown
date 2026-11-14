use macroquad::prelude::*;

use super::{menu::MenuScene, Scene};
use crate::input::Input;
use crate::ui::image::draw_texture_scaled;
use crate::ui::text::draw_text_centered;
use crate::utils::animation::Animation;
use crate::utils::assets::{load_font, load_logo};
use crate::screen::{internal_width, INTERNAL_HEIGHT};

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
    presents_fade: Animation,
    logo_fade: Animation,
}

impl IntroScene {
    pub fn new() -> Self {
        Self {
            font: load_font(),
            logo: load_logo(),
            presents_fade: Animation::new(
                PRESENTS_FADE_IN + PRESENTS_HOLD + PRESENTS_FADE_OUT + PRESENTS_WAIT,
            )
            .autostart(),
            logo_fade: Animation::new(LOGO_FADE_IN + LOGO_HOLD + LOGO_FADE_OUT).autostart(),
        }
    }
}

impl Scene for IntroScene {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Scene>> {
        let dt = get_frame_time();

        if !self.presents_fade.has_ended() {
            self.presents_fade.update(dt);
        } else {
            self.logo_fade.update(dt);
        }

        if input.any_pressed() || self.logo_fade.has_ended() {
            return Some(Box::new(MenuScene::new()));
        }
        None
    }

    fn draw(&self, _input: &Input) {
        clear_background(BLACK);
        let center = vec2(internal_width() / 2.0, INTERNAL_HEIGHT / 2.0);

        if !self.presents_fade.has_ended() {
            let alpha = fade_alpha(
                self.presents_fade.elapsed(),
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
                self.logo_fade.elapsed(),
                LOGO_FADE_IN,
                LOGO_HOLD,
                LOGO_FADE_OUT,
            );
            let top_left = (center - self.logo.size() * LOGO_SCALE / 2.0).round();
            draw_texture_scaled(&self.logo, top_left, LOGO_SCALE, Color::new(1.0, 1.0, 1.0, alpha));
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
