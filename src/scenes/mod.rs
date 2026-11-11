pub mod gameplay;

pub trait Scene {
    fn update(&mut self);
    fn draw(&self);
}
