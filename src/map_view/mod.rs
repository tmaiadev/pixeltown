pub mod camera;
mod chunks;
mod trees;

use macroquad::prelude::*;

use crate::map::{autotile, Map, MAP_SIZE};
use crate::utils::frame_animation::FrameAnimation;
use crate::utils::spritesheet::Spritesheet;
use camera::{Camera, TILE_SIZE};
use chunks::ChunkCache;

const WATER_FPS: f32 = 4.0;
const GRID_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.25);
const FOREST_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.27);

pub struct MapView {
    sheet: Spritesheet,
    nature: Spritesheet,
    water: FrameAnimation,
    cells: Vec<autotile::Tile>,
    features: Vec<Option<(u8, u8)>>,
    forests: Vec<Option<(u8, u8)>>,
    trees: Vec<autotile::TreeCell>,
    forest_trunks: Vec<(u16, u16)>,
    ground: ChunkCache,
    canopy: ChunkCache,
}

impl MapView {
    pub fn new(map: &Map) -> Self {
        let mut view = Self {
            sheet: Spritesheet::from_png(include_bytes!("../../assets/sprites/terrain.png"), TILE_SIZE),
            nature: Spritesheet::from_png(include_bytes!("../../assets/sprites/nature.png"), TILE_SIZE),
            water: FrameAnimation::new(autotile::WATER_FRAMES, WATER_FPS),
            cells: Vec::new(),
            features: Vec::new(),
            forests: Vec::new(),
            trees: Vec::new(),
            forest_trunks: Vec::new(),
            ground: ChunkCache::new(),
            canopy: ChunkCache::new(),
        };
        view.set_map(map);
        view
    }

    pub fn set_map(&mut self, map: &Map) {
        self.cells = autotile::tile_cells(map);
        self.features = autotile::feature_cells(map);
        self.forests = autotile::forest_cells(map);
        self.trees = autotile::tree_cells(map);
        self.forest_trunks = autotile::forest_trunks(map);
        self.bake();
    }

    pub fn update(&mut self, dt: f32) {
        self.water.update(dt);
    }

    // Animated tiles are left transparent in the ground layer and drawn every frame.
    fn bake(&self) {
        self.ground.render(|chunk| {
            let tiles = || chunks::chunk_tiles(chunk).filter(|&(_, _, i)| !self.cells[i].is_animated());
            for (x, y, i) in tiles() {
                let (col, row) = autotile::animated_cell(self.cells[i], 0, 0);
                self.sheet.draw_tile(col, row, x, y);
            }
            for (x, y, i) in tiles() {
                if let Some((col, row)) = self.features[i] {
                    self.nature.draw_tile(col as usize, row as usize, x, y);
                }
                if let Some((col, row)) = self.forests[i] {
                    self.nature.draw_tile_tinted(col as usize, row as usize, x, y, FOREST_SHADOW);
                }
            }
        });
        self.canopy.render(|chunk| {
            for (x, y, i) in chunks::chunk_tiles(chunk) {
                if let Some((col, row)) = self.forests[i] {
                    self.nature.draw_tile(col as usize, row as usize, x, y);
                }
            }
        });
    }

    pub fn draw(&self, camera: &Camera, viewport: Vec2, grid_visible: bool) {
        set_camera(&camera.camera_2d(viewport));

        let visible = camera.visible_tiles(viewport);
        let (x0, y0, x1, y1) = visible;
        self.ground.draw(visible);

        let frame = self.water.frame();
        let animated = || {
            (y0..y1)
                .flat_map(move |y| (x0..x1).map(move |x| (x, y, y * MAP_SIZE + x)))
                .filter(|&(_, _, i)| self.cells[i].is_animated())
        };
        // Each pass sticks to one spritesheet so consecutive tiles batch into one draw call.
        for (x, y, i) in animated() {
            let (col, row) = autotile::animated_cell(self.cells[i], frame, x + y);
            self.sheet.draw_tile(col, row, x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
        }
        for (x, y, i) in animated() {
            if let Some((col, row)) = self.features[i] {
                self.nature.draw_tile(col as usize, row as usize, x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
            }
        }

        let perspective = camera.perspective(viewport);
        trees::draw_shadows(&self.nature, &self.trees, visible);
        trees::draw_trunks(&self.nature, &self.forest_trunks, visible, &perspective);
        self.canopy.draw_raised(visible, &perspective, trees::TREE_HEIGHT);
        trees::draw_trees(&self.nature, &self.trees, visible, &perspective);

        if grid_visible {
            draw_grid(visible, camera.zoom());
        }
    }
}

fn draw_grid((x0, y0, x1, y1): (usize, usize, usize, usize), zoom: f32) {
    let thickness = 1.0 / zoom;
    let (left, right) = (x0 as f32 * TILE_SIZE, x1 as f32 * TILE_SIZE);
    let (top, bottom) = (y0 as f32 * TILE_SIZE, y1 as f32 * TILE_SIZE);
    for x in x0..=x1 {
        let px = x as f32 * TILE_SIZE;
        draw_line(px, top, px, bottom, thickness, GRID_COLOR);
    }
    for y in y0..=y1 {
        let py = y as f32 * TILE_SIZE;
        draw_line(left, py, right, py, thickness, GRID_COLOR);
    }
}
