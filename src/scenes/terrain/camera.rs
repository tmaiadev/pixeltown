use macroquad::prelude::*;

use crate::map::MAP_SIZE;

pub const TILE_SIZE: f32 = 16.0;
pub const MIN_ZOOM: f32 = 0.25;
pub const MAX_ZOOM: f32 = 2.0;
const ZOOM_STEP: f32 = 0.25;
const ZOOM_SMOOTHING: f32 = 14.0;
const PAN_SPEED: f32 = 240.0;
const MAP_PIXELS: f32 = MAP_SIZE as f32 * TILE_SIZE;

#[derive(Clone, Copy, Default)]
pub struct Motion {
    pub pan: Vec2,
    pub drag: Vec2,
    pub zoom_step: i32,
    pub zoom_delta: f32,
}

pub struct Camera {
    center: Vec2,
    zoom: f32,
    target_zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            center: Vec2::splat(MAP_PIXELS / 2.0),
            zoom: 1.0,
            target_zoom: 1.0,
        }
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn update(&mut self, dt: f32, viewport: Vec2, motion: Motion) {
        if motion.zoom_step != 0 {
            self.target_zoom = step_zoom(self.target_zoom, motion.zoom_step);
        }
        if motion.zoom_delta != 0.0 {
            self.zoom = (self.zoom * motion.zoom_delta.exp()).clamp(MIN_ZOOM, MAX_ZOOM);
            self.target_zoom = self.zoom;
        } else {
            let blend = 1.0 - (-ZOOM_SMOOTHING * dt).exp();
            self.zoom += (self.target_zoom - self.zoom) * blend;
        }

        self.center += motion.pan * PAN_SPEED * dt / self.zoom;
        self.center -= motion.drag / self.zoom;
        self.center = clamp_center(self.center, viewport / self.zoom);
    }

    pub fn view_center(&self) -> Vec2 {
        (self.center * self.zoom).round() / self.zoom
    }

    pub fn visible_tiles(&self, viewport: Vec2) -> (usize, usize, usize, usize) {
        let half = viewport / self.zoom / 2.0;
        let center = self.view_center();
        let to_tile = |world: f32, max: usize| ((world / TILE_SIZE).floor().max(0.0) as usize).min(max);
        (
            to_tile(center.x - half.x, MAP_SIZE),
            to_tile(center.y - half.y, MAP_SIZE),
            to_tile(center.x + half.x, MAP_SIZE - 1) + 1,
            to_tile(center.y + half.y, MAP_SIZE - 1) + 1,
        )
    }
}

fn step_zoom(zoom: f32, direction: i32) -> f32 {
    let steps = (zoom / ZOOM_STEP).round() as i32 + direction.signum();
    (steps as f32 * ZOOM_STEP).clamp(MIN_ZOOM, MAX_ZOOM)
}

fn clamp_axis(center: f32, view: f32) -> f32 {
    if view >= MAP_PIXELS {
        MAP_PIXELS / 2.0
    } else {
        center.clamp(view / 2.0, MAP_PIXELS - view / 2.0)
    }
}

fn clamp_center(center: Vec2, view: Vec2) -> Vec2 {
    vec2(clamp_axis(center.x, view.x), clamp_axis(center.y, view.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Vec2 = Vec2::new(853.0, 480.0);

    fn moved(camera: &mut Camera, motion: Motion) {
        camera.update(1.0, VIEWPORT, motion);
    }

    #[test]
    fn starts_centered_on_the_map() {
        let camera = Camera::new();
        assert_eq!(camera.view_center(), Vec2::splat(MAP_PIXELS / 2.0));
    }

    #[test]
    fn panning_stops_at_every_edge() {
        let mut camera = Camera::new();
        for direction in [vec2(-1.0, 0.0), vec2(1.0, 0.0), vec2(0.0, -1.0), vec2(0.0, 1.0)] {
            for _ in 0..200 {
                moved(&mut camera, Motion { pan: direction, ..Default::default() });
            }
            let half = VIEWPORT / camera.zoom() / 2.0;
            let center = camera.view_center();
            assert!(center.x - half.x >= -1.0 && center.x + half.x <= MAP_PIXELS + 1.0);
            assert!(center.y - half.y >= -1.0 && center.y + half.y <= MAP_PIXELS + 1.0);
        }
    }

    #[test]
    fn dragging_moves_the_map_with_the_pointer() {
        let mut camera = Camera::new();
        moved(&mut camera, Motion { drag: vec2(20.0, 0.0), ..Default::default() });
        assert!(camera.view_center().x < MAP_PIXELS / 2.0);
    }

    #[test]
    fn zoom_steps_snap_to_quarters_and_respect_limits() {
        assert_eq!(step_zoom(1.0, 1), 1.25);
        assert_eq!(step_zoom(1.0, -1), 0.75);
        assert_eq!(step_zoom(MAX_ZOOM, 1), MAX_ZOOM);
        assert_eq!(step_zoom(MIN_ZOOM, -1), MIN_ZOOM);
        assert_eq!(step_zoom(1.1, 1), 1.25);
    }

    #[test]
    fn fluid_zoom_is_clamped() {
        let mut camera = Camera::new();
        moved(&mut camera, Motion { zoom_delta: 10.0, ..Default::default() });
        assert_eq!(camera.zoom(), MAX_ZOOM);
        moved(&mut camera, Motion { zoom_delta: -20.0, ..Default::default() });
        assert_eq!(camera.zoom(), MIN_ZOOM);
    }

    #[test]
    fn step_zoom_animates_towards_the_target() {
        let mut camera = Camera::new();
        camera.update(0.016, VIEWPORT, Motion { zoom_step: 1, ..Default::default() });
        assert!(camera.zoom() > 1.0 && camera.zoom() < 1.25);
        for _ in 0..120 {
            camera.update(0.016, VIEWPORT, Motion::default());
        }
        assert!((camera.zoom() - 1.25).abs() < 0.001);
    }

    #[test]
    fn a_viewport_larger_than_the_map_centers_it() {
        let mut camera = Camera::new();
        let wide = vec2(1200.0, 480.0);
        camera.update(0.0, wide, Motion { zoom_delta: -10.0, ..Default::default() });
        camera.update(1.0, wide, Motion { pan: vec2(1.0, 0.0), ..Default::default() });
        assert_eq!(camera.view_center().x, MAP_PIXELS / 2.0);
    }

    #[test]
    fn visible_tiles_cover_the_viewport_and_stay_in_bounds() {
        let camera = Camera::new();
        let (x0, y0, x1, y1) = camera.visible_tiles(VIEWPORT);
        assert!(x0 < MAP_SIZE / 2 && x1 > MAP_SIZE / 2);
        assert!(y0 < MAP_SIZE / 2 && y1 > MAP_SIZE / 2);

        let mut far = Camera::new();
        far.update(0.0, VIEWPORT, Motion { zoom_delta: -10.0, ..Default::default() });
        let (x0, y0, x1, y1) = far.visible_tiles(VIEWPORT);
        assert!((x1 - x0).abs_diff(214) <= 2);
        assert!((y1 - y0).abs_diff(120) <= 2);
        assert!(x1 <= MAP_SIZE && y1 <= MAP_SIZE);
    }
}
