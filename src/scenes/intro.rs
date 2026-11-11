use macroquad::prelude::*;

use super::{gameplay::GameplayScene, Scene};
use crate::ui::text::draw_text_centered;
use crate::utils::animation::{Step, Timeline};
use crate::{internal_width, INTERNAL_HEIGHT};

const LOGO_SCALE: f32 = 2.0;

pub struct IntroScene {
    font: Font,
    logo: Texture2D,
    presents: Timeline,
    logo_timeline: Timeline,
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
            presents: Timeline::new(vec![
                Step::FadeIn(1.0),
                Step::Hold(1.5),
                Step::FadeOut(1.0),
                Step::Wait(0.5),
            ]),
            logo_timeline: Timeline::new(vec![
                Step::FadeIn(1.5),
                Step::Hold(2.0),
                Step::FadeOut(1.0),
            ]),
        }
    }
}

impl Scene for IntroScene {
    fn update(&mut self) -> Option<Box<dyn Scene>> {
        let skip = get_last_key_pressed().is_some() || is_mouse_button_pressed(MouseButton::Left);
        let dt = get_frame_time();

        if !self.presents.is_done() {
            self.presents.update(dt);
        } else {
            self.logo_timeline.update(dt);
        }

        if skip || self.logo_timeline.is_done() {
            return Some(Box::new(GameplayScene));
        }
        None
    }

    fn draw(&self) {
        clear_background(BLACK);
        let center = vec2(internal_width() / 2.0, INTERNAL_HEIGHT / 2.0);

        if !self.presents.is_done() {
            draw_text_centered(
                &self.font,
                "@tmaiadev presents",
                center,
                16,
                Color::new(1.0, 1.0, 1.0, self.presents.alpha()),
            );
        } else {
            let size = self.logo.size() * LOGO_SCALE;
            draw_texture_ex(
                &self.logo,
                (center.x - size.x / 2.0).round(),
                (center.y - size.y / 2.0).round(),
                Color::new(1.0, 1.0, 1.0, self.logo_timeline.alpha()),
                DrawTextureParams {
                    dest_size: Some(size),
                    ..Default::default()
                },
            );
        }
    }
}
