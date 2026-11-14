use std::path::Path;

use macroquad::miniquad::date;
use macroquad::prelude::*;

use super::Scene;
use crate::input::Input;
use crate::map::save::{self, SaveError};
use crate::map::{Map, TerrainType};
use crate::map_view::camera::{Camera, Motion};
use crate::map_view::MapView;
use crate::{internal_width, INTERNAL_HEIGHT};

pub struct GameplayScene {
    view: MapView,
    camera: Camera,
}

impl GameplayScene {
    pub const ZOOM: f32 = 1.0;

    pub fn new(map: &Map) -> Self {
        let viewport = vec2(internal_width(), INTERNAL_HEIGHT);
        Self {
            view: MapView::new(map),
            camera: Camera::centered(Self::ZOOM, viewport),
        }
    }

    pub fn load(path: &Path) -> Result<Self, SaveError> {
        Ok(Self::new(&save::read(path)?))
    }

    // Development entry point: picks up the newest save, or a fresh map when there is none.
    pub fn resume() -> Self {
        let loaded = save::latest().map(|path| Self::load(&path));
        match loaded {
            Some(Ok(scene)) => scene,
            Some(Err(err)) => {
                eprintln!("error: could not load latest save: {err}");
                Self::fresh()
            }
            None => Self::fresh(),
        }
    }

    fn fresh() -> Self {
        Self::new(&Map::generate(TerrainType::Inland, (date::now() * 1000.0) as u64))
    }
}

impl Scene for GameplayScene {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Scene>> {
        let dt = get_frame_time();
        let viewport = vec2(internal_width(), INTERNAL_HEIGHT);
        let motion = Motion {
            pan: input.pan(),
            drag: input.drag(),
            zoom_step: input.zoom_step(),
            zoom_delta: input.zoom_delta(),
        };
        self.camera.update(dt, viewport, motion);
        self.view.update(dt);
        None
    }

    fn draw(&self) {
        clear_background(BLACK);
        let viewport = vec2(internal_width(), INTERNAL_HEIGHT);
        self.view.draw(&self.camera, viewport, false);
    }
}
