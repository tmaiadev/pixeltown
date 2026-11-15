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
pub const MAP_CENTER: Vec2 = Vec2::new(MAP_PIXELS / 2.0, MAP_PIXELS / 2.0);

// Tiles from (x0, y0) inclusive to (x1, y1) exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileRect {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

impl TileRect {
    pub fn grow(self, margin: usize) -> Self {
        Self {
            x0: self.x0.saturating_sub(margin),
            y0: self.y0.saturating_sub(margin),
            x1: (self.x1 + margin).min(MAP_SIZE),
            y1: (self.y1 + margin).min(MAP_SIZE),
        }
    }
}

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

    pub fn centered(zoom: f32, viewport: Vec2) -> Self {
        let mut camera = Self::new(viewport);
        camera.set_view(MAP_CENTER, zoom, viewport);
        camera
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn view(&self) -> (Vec2, f32) {
        (self.center, self.zoom)
    }

    pub fn set_view(&mut self, center: Vec2, zoom: f32, viewport: Vec2) {
        self.zoom = zoom.clamp(min_zoom(viewport), MAX_ZOOM);
        self.target_zoom = self.zoom;
        self.center = clamp_center(center, viewport / self.zoom);
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

    // Projects onto the whole screen with the view fitted to `area`, the part the map is visible in.
    // Snapping the view's corner, not its center, keeps odd-sized areas from clipping the map edge.
    pub fn camera_2d(&self, screen: Vec2, area: Rect) -> Camera2D {
        let corner = ((self.center - area.size() / 2.0 / self.zoom) * self.zoom).round() / self.zoom;
        Camera2D {
            zoom: 2.0 * self.zoom / screen,
            target: corner + (screen / 2.0 - area.point()) / self.zoom,
            ..Default::default()
        }
    }

    // Tile under a screen point, if the point is inside `area` and the map.
    pub fn tile_at(&self, point: Vec2, area: Rect) -> Option<(i32, i32)> {
        if !area.contains(point) {
            return None;
        }
        let corner = ((self.center - area.size() / 2.0 / self.zoom) * self.zoom).round() / self.zoom;
        let world = corner + (point - area.point()) / self.zoom;
        let (x, y) = ((world.x / TILE_SIZE).floor() as i32, (world.y / TILE_SIZE).floor() as i32);
        crate::map::Map::in_bounds(x, y).then_some((x, y))
    }

    // Screen position of a tile's top-left corner.
    pub fn tile_origin(&self, tile: (i32, i32), area: Rect) -> Vec2 {
        let corner = ((self.center - area.size() / 2.0 / self.zoom) * self.zoom).round() / self.zoom;
        area.point() + (vec2(tile.0 as f32, tile.1 as f32) * TILE_SIZE - corner) * self.zoom
    }

    // Moves the view just enough to keep the tile fully visible.
    pub fn reveal(&mut self, tile: (i32, i32), viewport: Vec2) {
        let half = viewport / self.zoom / 2.0;
        let min = vec2(tile.0 as f32, tile.1 as f32) * TILE_SIZE;
        let max = min + Vec2::splat(TILE_SIZE);
        let center = self.view_center();
        let shift = (min - (center - half)).min(Vec2::ZERO) + (max - (center + half)).max(Vec2::ZERO);
        self.center = clamp_center(self.center + shift, viewport / self.zoom);
    }

    pub fn view_center(&self) -> Vec2 {
        (self.center * self.zoom).round() / self.zoom
    }

    pub fn perspective(&self, viewport: Vec2) -> Perspective {
        Perspective::new(self.view_center(), viewport / self.zoom)
    }

    pub fn visible_tiles(&self, viewport: Vec2) -> TileRect {
        let half = viewport / self.zoom / 2.0;
        let center = self.view_center();
        let to_tile = |world: f32, max: usize| ((world / TILE_SIZE).floor().max(0.0) as usize).min(max);
        TileRect {
            x0: to_tile(center.x - half.x, MAP_SIZE),
            y0: to_tile(center.y - half.y, MAP_SIZE),
            x1: to_tile(center.x + half.x, MAP_SIZE - 1) + 1,
            y1: to_tile(center.y + half.y, MAP_SIZE - 1) + 1,
        }
    }
}

// Zoom interpolates geometrically so the scale changes at a steady rate on screen.
pub fn glide(from: (Vec2, f32), to: (Vec2, f32), t: f32) -> (Vec2, f32) {
    let t = t * t * (3.0 - 2.0 * t);
    (from.0.lerp(to.0, t), from.1 * (to.1 / from.1).powf(t))
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
    fn centered_camera_sits_on_the_map_center_at_the_given_zoom() {
        let camera = Camera::centered(1.0, VIEWPORT);
        assert_eq!(camera.view(), (MAP_CENTER, 1.0));
        assert_eq!(Camera::centered(0.0, VIEWPORT).zoom(), min_zoom(VIEWPORT));
    }

    #[test]
    fn glide_eases_from_start_to_end() {
        let from = (Vec2::ZERO, 0.25);
        let to = (MAP_CENTER, 1.0);
        assert_eq!(glide(from, to, 0.0), from);
        let end = glide(from, to, 1.0);
        assert!((end.0 - to.0).length() < 1e-3 && (end.1 - to.1).abs() < 1e-6);
        let (center, zoom) = glide(from, to, 0.5);
        assert_eq!(center, MAP_CENTER / 2.0);
        assert!((zoom - 0.5).abs() < 1e-6);
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
        let TileRect { x0, y0, x1, y1 } = camera.visible_tiles(VIEWPORT);
        assert!(x0 < MAP_SIZE / 2 && x1 > MAP_SIZE / 2);
        assert!(y0 < MAP_SIZE / 2 && y1 > MAP_SIZE / 2);

        let mut far = Camera::new(VIEWPORT);
        far.update(0.0, VIEWPORT, Motion { zoom_delta: -10.0, ..Default::default() });
        let TileRect { x0, y0, x1, y1 } = far.visible_tiles(VIEWPORT);
        assert_eq!((x0, x1), (0, MAP_SIZE));
        assert!(y1 - y0 < MAP_SIZE);
    }

    #[test]
    fn panned_to_a_corner_the_map_corner_meets_the_area_corner() {
        for area in [Rect::new(0.0, 16.0, 853.0, 427.0), Rect::new(0.0, 16.0, 852.0, 428.0)] {
            let mut camera = Camera::new(area.size());
            let projection = |camera: &Camera| camera.camera_2d(VIEWPORT, area);
            let to_screen = |camera: &Camera, world: Vec2| {
                (world - projection(camera).target) * camera.zoom() + VIEWPORT / 2.0
            };
            camera.set_view(Vec2::ZERO, 1.0, area.size());
            assert_eq!(to_screen(&camera, Vec2::ZERO), area.point());
            camera.set_view(Vec2::splat(MAP_PIXELS), 1.0, area.size());
            let far = area.point() + area.size();
            assert_eq!(to_screen(&camera, Vec2::splat(MAP_PIXELS)), far);
        }
    }

    #[test]
    fn growing_a_tile_rect_stays_inside_the_map() {
        let edge = TileRect { x0: 0, y0: 3, x1: MAP_SIZE, y1: 10 };
        assert_eq!(edge.grow(2), TileRect { x0: 0, y0: 1, x1: MAP_SIZE, y1: 12 });
    }

    #[test]
    fn tile_at_inverts_the_view_and_rejects_points_outside() {
        let area = Rect::new(0.0, 16.0, 480.0, 300.0);
        let camera = Camera::centered(1.0, area.size());
        let center = area.center();
        assert_eq!(camera.tile_at(center, area), Some((128, 128)));
        assert_eq!(camera.tile_at(vec2(0.0, 0.0), area), None);
        let origin = camera.tile_origin((128, 128), area);
        assert_eq!(camera.tile_at(origin + Vec2::splat(1.0), area), Some((128, 128)));
    }
}
