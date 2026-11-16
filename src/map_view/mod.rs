mod buildings;
pub mod camera;
mod chunks;
mod power_lines;
mod trees;

use macroquad::prelude::*;

use crate::map::autotile::{self, GroundSprite, SpriteCell};
use crate::map::buildings::Building;
use crate::map::{Map, MAP_SIZE};
use crate::utils::frame_animation::FrameAnimation;
use crate::utils::spritesheet::Spritesheet;
use camera::{Camera, TileRect, TILE_SIZE};
use chunks::{ChunkCache, CHUNKS};

const WATER_FPS: f32 = 4.0;
const GRID_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.25);
const BRIDGE_HEIGHT: f32 = 3.0;
const BUILD_OK: Color = Color::new(0.45, 0.75, 1.0, 0.45);
const BUILD_BLOCKED: Color = Color::new(1.0, 0.2, 0.2, 0.45);
const FOREST_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.27);

pub struct MapView {
    sheet: Spritesheet,
    nature: Spritesheet,
    roads_sheet: Spritesheet,
    buildings_sheet: Spritesheet,
    water: FrameAnimation,
    cells: Vec<GroundSprite>,
    features: Vec<Option<SpriteCell>>,
    forests: Vec<Option<SpriteCell>>,
    roads: Vec<Option<SpriteCell>>,
    bridge_trusses: Vec<Option<SpriteCell>>,
    trees: Vec<autotile::TreeCell>,
    poles: Vec<autotile::WireCell>,
    forest_trunks: Vec<(u16, u16)>,
    buildings: Vec<Building>,
    clock: f32,
    ground: ChunkCache,
    canopy: ChunkCache,
    bridges: ChunkCache,
}

impl MapView {
    pub fn new(map: &Map) -> Self {
        let mut view = Self {
            sheet: Spritesheet::from_png(include_bytes!("../../assets/sprites/terrain.png"), TILE_SIZE),
            nature: Spritesheet::from_png(include_bytes!("../../assets/sprites/nature.png"), TILE_SIZE),
            roads_sheet: Spritesheet::from_png(include_bytes!("../../assets/sprites/roads.png"), TILE_SIZE),
            buildings_sheet: Spritesheet::from_png(include_bytes!("../../assets/sprites/buildings.png"), TILE_SIZE),
            water: FrameAnimation::new(autotile::WATER_FRAMES, WATER_FPS),
            cells: Vec::new(),
            features: Vec::new(),
            forests: Vec::new(),
            roads: Vec::new(),
            bridge_trusses: Vec::new(),
            trees: Vec::new(),
            poles: Vec::new(),
            forest_trunks: Vec::new(),
            buildings: Vec::new(),
            clock: 0.0,
            ground: ChunkCache::new(),
            canopy: ChunkCache::new(),
            bridges: ChunkCache::new(),
        };
        view.set_map(map);
        view
    }

    pub fn set_map(&mut self, map: &Map) {
        self.cells = autotile::ground_sprites(map);
        self.features = autotile::feature_cells(map);
        self.forests = autotile::forest_cells(map);
        self.roads = autotile::road_cells(map);
        self.bridge_trusses = autotile::bridge_truss_cells(map);
        self.trees = autotile::tree_cells(map);
        self.poles = autotile::wire_cells(map);
        self.forest_trunks = autotile::forest_trunks(map);
        self.buildings = map.buildings();
        self.bake(0..CHUNKS * CHUNKS);
    }

    // Refreshes only what building on `tiles` can change, rebaking just the chunks around them.
    pub fn update_tiles(&mut self, map: &Map, tiles: &[(i32, i32)]) {
        self.features = autotile::feature_cells(map);
        self.forests = autotile::forest_cells(map);
        self.roads = autotile::road_cells(map);
        self.bridge_trusses = autotile::bridge_truss_cells(map);
        self.trees = autotile::tree_cells(map);
        self.poles = autotile::wire_cells(map);
        self.forest_trunks = autotile::forest_trunks(map);
        self.buildings = map.buildings();
        let mut dirty: Vec<usize> = tiles.iter().flat_map(|&(x, y)| chunks::chunks_around(x, y)).collect();
        dirty.sort_unstable();
        dirty.dedup();
        self.bake(dirty);
    }

    pub fn update(&mut self, dt: f32) {
        self.water.update(dt);
        self.clock += dt;
    }

    fn bake(&mut self, chunks: impl IntoIterator<Item = usize> + Clone) {
        let (cells, features, forests, roads) = (&self.cells, &self.features, &self.forests, &self.roads);
        let (sheet, nature, roads_sheet) = (&self.sheet, &self.nature, &self.roads_sheet);
        let (buildings, buildings_sheet) = (&self.buildings, &self.buildings_sheet);
        let animated = |chunk| chunks::chunk_tiles(chunk).any(|(_, _, i)| cells[i].is_animated());
        self.ground.render_chunks(
            chunks.clone(),
            |chunk| if animated(chunk) { autotile::WATER_FRAMES } else { 1 },
            |chunk, frame| {
                for (x, y, i) in chunks::chunk_tiles(chunk) {
                    let cell = autotile::animated_cell(cells[i], frame, i % MAP_SIZE + i / MAP_SIZE);
                    sheet.draw_tile(cell, x, y);
                }
                for (x, y, i) in chunks::chunk_tiles(chunk) {
                    if let Some(cell) = features[i] {
                        nature.draw_tile(cell, x, y);
                    }
                    if let Some(cell) = forests[i] {
                        nature.draw_tile_tinted(cell, x, y, FOREST_SHADOW);
                    }
                    if let Some(cell) = roads[i] {
                        roads_sheet.draw_tile(cell, x, y);
                    }
                }
                buildings::draw_yards(buildings_sheet, buildings, chunks::chunk_rect(chunk));
            },
        );
        self.canopy.render_chunks(
            chunks.clone(),
            |_| 1,
            |chunk, _| {
                for (x, y, i) in chunks::chunk_tiles(chunk) {
                    if let Some(cell) = forests[i] {
                        nature.draw_tile(cell, x, y);
                    }
                }
            },
        );
        let (trusses, roads_sheet) = (&self.bridge_trusses, &self.roads_sheet);
        self.bridges.render_chunks(
            chunks,
            |_| 1,
            |chunk, _| {
                for (x, y, i) in chunks::chunk_tiles(chunk) {
                    if let Some(cell) = trusses[i] {
                        roads_sheet.draw_tile(cell, x, y);
                    }
                }
            },
        );
    }

    // Tints tiles, under the map camera set by `draw`.
    pub fn draw_highlight(&self, camera: &Camera, screen: Vec2, area: Rect, tiles: impl IntoIterator<Item = ((i32, i32), bool)>) {
        set_camera(&camera.camera_2d(screen, area));
        for ((x, y), buildable) in tiles {
            let color = if buildable { BUILD_OK } else { BUILD_BLOCKED };
            draw_rectangle(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, TILE_SIZE, TILE_SIZE, color);
        }
    }

    pub fn draw(&self, camera: &Camera, screen: Vec2, area: Rect, grid_visible: bool) {
        set_camera(&camera.camera_2d(screen, area));
        let viewport = area.size();

        let visible = camera.visible_tiles(viewport);
        self.ground.draw(visible, self.water.frame());

        let perspective = camera.perspective(viewport);
        trees::draw_shadows(&self.nature, &self.trees, visible);
        power_lines::draw_shadows(&self.poles, visible);
        buildings::draw_shadows(&self.buildings, visible);
        trees::draw_trunks(&self.nature, &self.forest_trunks, visible, &perspective);
        self.bridges.draw_raised(visible, &perspective, BRIDGE_HEIGHT);
        self.canopy.draw_raised(visible, &perspective, trees::TREE_HEIGHT);
        trees::draw_trees(&self.nature, &self.trees, visible, &perspective);
        buildings::draw(&self.buildings_sheet, &self.buildings, visible, &perspective, self.clock);
        power_lines::draw_poles(&self.poles, visible, &perspective);

        if grid_visible {
            draw_grid(visible, camera.zoom());
        }
    }
}

fn draw_grid(TileRect { x0, y0, x1, y1 }: TileRect, zoom: f32) {
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
