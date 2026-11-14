mod cli;
mod input;
mod map;
mod scenes;
mod ui;
mod utils;

use macroquad::prelude::*;
use input::Input;
use cli::{Args, SceneName};
use scenes::Scene;

pub const INTERNAL_HEIGHT: f32 = 480.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Pixeltown".to_owned(),
        fullscreen: !Args::from_env().windowed,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

pub fn internal_width() -> f32 {
    INTERNAL_HEIGHT * screen_width() / screen_height()
}

pub fn set_internal_camera() {
    let width = internal_width();
    set_camera(&Camera2D {
        zoom: vec2(2.0 / width, 2.0 / INTERNAL_HEIGHT),
        target: vec2(width / 2.0, INTERNAL_HEIGHT / 2.0),
        ..Default::default()
    });
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Args::from_env();
    let fps_font = args.show_fps.then(utils::assets::load_font);
    let mut scene: Box<dyn Scene> = args.scene.unwrap_or(SceneName::Intro).build();
    let mut input = Input::new();

    loop {
        set_internal_camera();
        input.update();
        if let Some(next) = scene.update(&input) {
            scene = next;
        }
        scene.draw();
        if let Some(font) = &fps_font {
            ui::fps::draw_fps(font);
        }
        next_frame().await;
    }
}
