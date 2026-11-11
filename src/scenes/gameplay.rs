use macroquad::prelude::*;

use super::Scene;

pub struct GameplayScene;

impl Scene for GameplayScene {
    fn update(&mut self) -> Option<Box<dyn Scene>> {
        None
    }

    fn draw(&self) {
        clear_background(BLACK);
    }
}
