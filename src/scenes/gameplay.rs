use macroquad::prelude::*;

use super::Scene;

pub struct GameplayScene;

impl Scene for GameplayScene {
    fn update(&mut self) {}

    fn draw(&self) {
        clear_background(BLACK);
    }
}
