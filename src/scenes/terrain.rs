mod camera;
mod dock;

use macroquad::miniquad::date;
use macroquad::prelude::*;

use super::{gameplay::GameplayScene, Scene};
use crate::input::{Action, Device, Input};
use crate::map::{autotile, rng::Rng, Map, TerrainType, MAP_SIZE};
use crate::utils::animation::Animation;
use crate::utils::assets::load_font;
use crate::utils::frame_animation::FrameAnimation;
use crate::utils::spritesheet::Spritesheet;
use crate::{internal_width, set_internal_camera, INTERNAL_HEIGHT};
use camera::{Camera, Motion, TILE_SIZE};
use dock::{DockButton, DockView};

const FADE_IN: f32 = 1.0;
const FADE_OUT: f32 = 0.8;
const WATER_FPS: f32 = 4.0;
const GRID_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.25);
const NAVIGATION: [Action; 4] = [Action::Up, Action::Down, Action::Left, Action::Right];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    FadingIn,
    Editing,
    FadingOut,
}

pub struct TerrainScene {
    font: Font,
    sheet: Spritesheet,
    water: FrameAnimation,
    map: Map,
    cells: Vec<autotile::Tile>,
    selected: TerrainType,
    camera: Camera,
    rng: Rng,
    phase: Phase,
    fade_in: Animation,
    fade_out: Animation,
    focus: usize,
    focus_visible: bool,
    device: Device,
    pointer: Option<Vec2>,
    grid_visible: bool,
}

impl TerrainScene {
    pub fn new() -> Self {
        let sheet = Spritesheet::from_png(
            include_bytes!("../../assets/sprites/terrain.png"),
            TILE_SIZE,
        );

        let mut rng = Rng::new((date::now() * 1000.0) as u64);
        let selected = TerrainType::Inland;
        let map = Map::generate(selected, rng.next_u64());
        let cells = autotile::tile_cells(&map);

        Self {
            font: load_font(),
            sheet,
            water: FrameAnimation::new(autotile::WATER_FRAMES, WATER_FPS),
            map,
            cells,
            selected,
            camera: Camera::new(vec2(internal_width(), INTERNAL_HEIGHT)),
            rng,
            phase: Phase::FadingIn,
            fade_in: Animation::new(FADE_IN).autostart(),
            fade_out: Animation::new(FADE_OUT),
            focus: dock::GENERATE,
            focus_visible: false,
            device: Device::KeyboardMouse,
            pointer: None,
            grid_visible: false,
        }
    }

    fn generate(&mut self) {
        self.map = Map::generate(self.selected, self.rng.next_u64());
        self.cells = autotile::tile_cells(&self.map);
    }

    fn activate(&mut self, button: usize) {
        match DockButton::at(button) {
            DockButton::Terrain(kind) => {
                self.selected = kind;
                self.generate();
            }
            DockButton::Generate => self.generate(),
            DockButton::Accept => {
                self.phase = Phase::FadingOut;
                self.fade_out.start();
            }
        }
    }

    fn handle_dock(&mut self, input: &Input, viewport: Vec2) {
        for action in NAVIGATION {
            if input.pressed_nav(action) {
                self.focus = dock::move_focus(self.focus, action);
                self.focus_visible = true;
            }
        }
        if input.pointer_moved() || input.last_device() == Device::Touch {
            self.focus_visible = false;
        }

        if input.pressed(Action::Confirm) {
            self.activate(self.focus);
        }
        if input.pointer_clicked() {
            let dock = dock::layout(viewport.x, viewport.y);
            if let Some(i) = input.pointer().and_then(|p| dock.button_at(p)) {
                self.focus = i;
                self.activate(i);
            }
        }
    }

    fn draw_map(&self, viewport: Vec2) {
        set_camera(&self.camera.camera_2d(viewport));

        let visible = self.camera.visible_tiles(viewport);
        let (x0, y0, x1, y1) = visible;
        let frame = self.water.frame();
        for y in y0..y1 {
            for x in x0..x1 {
                let tile = self.cells[y * MAP_SIZE + x];
                let (col, row) = autotile::animated_cell(tile, frame, x + y);
                self.sheet.draw_tile(col, row, x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
            }
        }

        if self.grid_visible {
            draw_grid(visible, self.camera.zoom());
        }
    }

    fn draw_ui(&self, viewport: Vec2, alpha: f32) {
        let dock = dock::layout(viewport.x, viewport.y);
        let hovered = match self.device {
            Device::KeyboardMouse if !self.focus_visible => {
                self.pointer.and_then(|p| dock.button_at(p))
            }
            _ => None,
        };
        let view = DockView {
            font: &self.font,
            selected: self.selected,
            hovered,
            focus: self.focus_visible.then_some(self.focus),
            alpha,
        };
        dock::draw(&dock, &view);
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

impl Scene for TerrainScene {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Scene>> {
        let dt = get_frame_time();
        let viewport = vec2(internal_width(), INTERNAL_HEIGHT);
        self.device = input.last_device();
        self.pointer = input.pointer();

        match self.phase {
            Phase::FadingIn => {
                self.fade_in.update(dt);
                if self.fade_in.has_ended() {
                    self.phase = Phase::Editing;
                }
            }
            Phase::FadingOut => {
                self.fade_out.update(dt);
                if self.fade_out.has_ended() {
                    return Some(Box::new(GameplayScene));
                }
            }
            Phase::Editing => {}
        }

        let motion = if self.phase == Phase::FadingOut {
            Motion::default()
        } else {
            Motion {
                pan: input.pan(),
                drag: input.drag(),
                zoom_step: input.zoom_step(),
                zoom_delta: input.zoom_delta(),
            }
        };
        self.camera.update(dt, viewport, motion);
        self.water.update(dt);

        if self.phase == Phase::Editing {
            if input.grid_toggled() {
                self.grid_visible = !self.grid_visible;
            }
            self.handle_dock(input, viewport);
        }
        None
    }

    fn draw(&self) {
        clear_background(BLACK);
        let viewport = vec2(internal_width(), INTERNAL_HEIGHT);
        self.draw_map(viewport);

        set_internal_camera();
        let ui_alpha = match self.phase {
            Phase::FadingOut => 1.0 - self.fade_out.progress(),
            _ => 1.0,
        };
        if ui_alpha > 0.0 {
            self.draw_ui(viewport, ui_alpha);
        }

        if self.phase == Phase::FadingIn {
            let alpha = 1.0 - self.fade_in.progress();
            draw_rectangle(0.0, 0.0, viewport.x, viewport.y, Color::new(0.0, 0.0, 0.0, alpha));
        }
    }
}
