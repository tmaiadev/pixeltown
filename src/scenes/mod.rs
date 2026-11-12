use crate::input::Input;

pub mod gameplay;
pub mod intro;
pub mod menu;

pub trait Scene {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Scene>>;
    fn draw(&self);
}
