use macroquad::miniquad::date;

pub fn time_seed() -> u64 {
    (date::now() * 1000.0) as u64
}
