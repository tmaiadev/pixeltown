use crate::input::Input;

pub mod gameplay;
pub mod intro;
pub mod load_game;
pub mod menu;
pub mod terrain;

pub trait Scene {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Scene>>;
    fn draw(&self, input: &Input);
}
