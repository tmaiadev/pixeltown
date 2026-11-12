use macroquad::prelude::*;

use super::Scene;
use crate::input::Input;

pub struct GameplayScene;

impl Scene for GameplayScene {
    fn update(&mut self, _input: &Input) -> Option<Box<dyn Scene>> {
        None
    }

    fn draw(&self) {
        clear_background(BLACK);
    }
}
