pub mod gameplay;
pub mod intro;

pub trait Scene {
    fn update(&mut self) -> Option<Box<dyn Scene>>;
    fn draw(&self);
}
