use macroquad::prelude::*;

use crate::map::MAP_SIZE;
use crate::utils::perspective::Perspective;

pub const TILE_SIZE: f32 = 16.0;
pub const MAX_ZOOM: f32 = 2.0;
const ZOOM_LEVELS: [f32; 4] = [0.5, 1.0, 1.5, 2.0];
const ZOOM_EPSILON: f32 = 1e-4;
const ZOOM_SMOOTHING: f32 = 14.0;
const PAN_SPEED: f32 = 600.0;
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
    pub fn new(viewport: Vec2) -> Self {
        let zoom = min_zoom(viewport);
        Self {
            center: Vec2::splat(MAP_PIXELS / 2.0),
            zoom,
            target_zoom: zoom,
        }
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn update(&mut self, dt: f32, viewport: Vec2, motion: Motion) {
        let min_zoom = min_zoom(viewport);
        if motion.zoom_step != 0 {
            self.target_zoom = step_zoom(self.target_zoom, motion.zoom_step, min_zoom);
        }
        self.target_zoom = self.target_zoom.clamp(min_zoom, MAX_ZOOM);
        if motion.zoom_delta != 0.0 {
            self.zoom = (self.zoom * motion.zoom_delta.exp()).clamp(min_zoom, MAX_ZOOM);
            self.target_zoom = self.zoom;
        } else {
            let blend = 1.0 - (-ZOOM_SMOOTHING * dt).exp();
            self.zoom += (self.target_zoom - self.zoom) * blend;
        }

        self.center += motion.pan * PAN_SPEED * dt / self.zoom;
        self.center -= motion.drag / self.zoom;
        self.center = clamp_center(self.center, viewport / self.zoom);
    }

    pub fn camera_2d(&self, viewport: Vec2) -> Camera2D {
        Camera2D {
            zoom: 2.0 * self.zoom / viewport,
            target: self.view_center(),
            ..Default::default()
        }
    }

    pub fn view_center(&self) -> Vec2 {
        (self.center * self.zoom).round() / self.zoom
    }

    pub fn perspective(&self, viewport: Vec2) -> Perspective {
        Perspective::new(self.view_center(), viewport / self.zoom)
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

fn min_zoom(viewport: Vec2) -> f32 {
    viewport.max_element() / MAP_PIXELS
}

fn step_zoom(zoom: f32, direction: i32, min_zoom: f32) -> f32 {
    let levels = std::iter::once(min_zoom).chain(ZOOM_LEVELS.into_iter().filter(|level| *level > min_zoom));
    if direction > 0 {
        levels.filter(|level| *level > zoom + ZOOM_EPSILON).fold(MAX_ZOOM, f32::min)
    } else {
        levels.filter(|level| *level < zoom - ZOOM_EPSILON).fold(min_zoom, f32::max)
    }
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
    fn starts_zoomed_out_to_show_the_whole_map() {
        let camera = Camera::new(VIEWPORT);
        assert_eq!(camera.zoom(), min_zoom(VIEWPORT));
    }

    #[test]
    fn starts_centered_on_the_map() {
        let camera = Camera::new(VIEWPORT);
        let offset = (camera.view_center() - Vec2::splat(MAP_PIXELS / 2.0)).abs();
        assert!(offset.max_element() <= 0.5 / camera.zoom());
    }

    #[test]
    fn panning_stops_at_every_edge() {
        let mut camera = Camera::new(VIEWPORT);
        camera.zoom = 1.0;
        camera.target_zoom = 1.0;
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
        let mut camera = Camera::new(VIEWPORT);
        camera.zoom = 1.0;
        camera.target_zoom = 1.0;
        moved(&mut camera, Motion { drag: vec2(20.0, 0.0), ..Default::default() });
        assert!(camera.view_center().x < MAP_PIXELS / 2.0);
    }

    #[test]
    fn zoom_steps_follow_fixed_levels_and_respect_limits() {
        let min = min_zoom(VIEWPORT);
        assert_eq!(step_zoom(1.0, 1, min), 1.5);
        assert_eq!(step_zoom(1.0, -1, min), 0.5);
        assert_eq!(step_zoom(0.5, -1, min), min);
        assert_eq!(step_zoom(min, 1, min), 0.5);
        assert_eq!(step_zoom(MAX_ZOOM, 1, min), MAX_ZOOM);
        assert_eq!(step_zoom(min, -1, min), min);
        assert_eq!(step_zoom(1.1, 1, min), 1.5);
        assert_eq!(step_zoom(1.1, -1, min), 1.0);
    }

    #[test]
    fn min_zoom_fits_the_larger_viewport_side_to_the_map() {
        assert_eq!(min_zoom(VIEWPORT) * MAP_PIXELS, VIEWPORT.x);
        assert_eq!(min_zoom(vec2(300.0, 900.0)) * MAP_PIXELS, 900.0);
    }

    #[test]
    fn fluid_zoom_is_clamped() {
        let mut camera = Camera::new(VIEWPORT);
        moved(&mut camera, Motion { zoom_delta: 10.0, ..Default::default() });
        assert_eq!(camera.zoom(), MAX_ZOOM);
        moved(&mut camera, Motion { zoom_delta: -20.0, ..Default::default() });
        assert_eq!(camera.zoom(), min_zoom(VIEWPORT));
    }

    #[test]
    fn step_zoom_animates_towards_the_target() {
        let mut camera = Camera::new(VIEWPORT);
        camera.zoom = 1.0;
        camera.target_zoom = 1.0;
        camera.update(0.016, VIEWPORT, Motion { zoom_step: 1, ..Default::default() });
        assert!(camera.zoom() > 1.0 && camera.zoom() < 1.5);
        for _ in 0..120 {
            camera.update(0.016, VIEWPORT, Motion::default());
        }
        assert!((camera.zoom() - 1.5).abs() < 0.001);
    }

    #[test]
    fn a_viewport_larger_than_the_map_centers_it() {
        let mut camera = Camera::new(VIEWPORT);
        let wide = vec2(1200.0, 480.0);
        camera.update(0.0, wide, Motion { zoom_delta: -10.0, ..Default::default() });
        camera.update(1.0, wide, Motion { pan: vec2(1.0, 0.0), ..Default::default() });
        assert_eq!(camera.view_center().x, MAP_PIXELS / 2.0);
    }

    #[test]
    fn visible_tiles_cover_the_viewport_and_stay_in_bounds() {
        let camera = Camera::new(VIEWPORT);
        let (x0, y0, x1, y1) = camera.visible_tiles(VIEWPORT);
        assert!(x0 < MAP_SIZE / 2 && x1 > MAP_SIZE / 2);
        assert!(y0 < MAP_SIZE / 2 && y1 > MAP_SIZE / 2);

        let mut far = Camera::new(VIEWPORT);
        far.update(0.0, VIEWPORT, Motion { zoom_delta: -10.0, ..Default::default() });
        let (x0, y0, x1, y1) = far.visible_tiles(VIEWPORT);
        assert_eq!((x0, x1), (0, MAP_SIZE));
        assert!(y1 - y0 < MAP_SIZE);
    }
}
