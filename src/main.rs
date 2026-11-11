mod scenes;

use macroquad::prelude::*;
use scenes::{gameplay::GameplayScene, Scene};

const INTERNAL_HEIGHT: f32 = 480.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Pixeltown".to_owned(),
        fullscreen: true,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

fn set_internal_camera() {
    let width = INTERNAL_HEIGHT * screen_width() / screen_height();
    set_camera(&Camera2D {
        zoom: vec2(2.0 / width, 2.0 / INTERNAL_HEIGHT),
        target: vec2(width / 2.0, INTERNAL_HEIGHT / 2.0),
        ..Default::default()
    });
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut scene: Box<dyn Scene> = Box::new(GameplayScene);

    loop {
        set_internal_camera();
        scene.update();
        scene.draw();
        next_frame().await;
    }
}
