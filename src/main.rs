mod cli;
mod input;
mod map;
mod map_view;
mod scenes;
mod screen;
mod ui;
mod utils;

use macroquad::prelude::*;
use input::Input;
use cli::{Args, SceneName};
use scenes::Scene;
use screen::set_internal_camera;

fn window_conf() -> Conf {
    Conf {
        window_title: "Pixeltown".to_owned(),
        fullscreen: !Args::from_env().windowed,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
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
            set_internal_camera();
            ui::fps::draw_fps(font);
        }
        next_frame().await;
    }
}
