mod toolbar;
mod top_bar;

use std::path::Path;

use chrono::Local;
use macroquad::prelude::*;

use super::Scene;
use crate::input::{Action, Device, Input};
use crate::map::save::{self, SaveError};
use crate::map::{Map, TerrainType};
use crate::map_view::camera::{Camera, Motion};
use crate::map_view::MapView;
use crate::screen::{self, set_internal_camera};
use crate::utils::assets::{load_font, load_icons};
use crate::utils::seed::time_seed;
use crate::utils::spritesheet::Spritesheet;
use toolbar::{Tool, ToolbarView};
use top_bar::{Speed, Stats, TopBarView};

const NAVIGATION: [Action; 4] = [Action::Up, Action::Down, Action::Left, Action::Right];
// Placeholder residence, commerce, farming and industry demand until the simulation drives it.
const DEMAND: [f32; 4] = [1.0, 0.5, 0.7, 0.2];

pub struct GameplayScene {
    font: Font,
    icons: Spritesheet,
    view: MapView,
    camera: Camera,
    stats: Stats,
    tool: Option<Tool>,
    focus: Tool,
    focus_visible: bool,
}

impl GameplayScene {
    pub const ZOOM: f32 = 1.0;

    pub fn new(map: &Map) -> Self {
        let area = map_area(screen::viewport());
        Self {
            font: load_font(),
            icons: load_icons(),
            view: MapView::new(map),
            camera: Camera::centered(Self::ZOOM, area.size()),
            stats: Stats {
                speed: Speed::Normal,
                cash: 200_000,
                date: Local::now().date_naive(),
                population: 1_432,
            },
            tool: None,
            focus: Tool::Roads,
            focus_visible: false,
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
        Self::new(&Map::generate(TerrainType::Inland, time_seed()))
    }

    fn toggle_tool(&mut self, tool: Tool) {
        self.tool = (self.tool != Some(tool)).then_some(tool);
    }

    fn handle_ui(&mut self, input: &Input, viewport: Vec2) {
        let toolbar = toolbar::layout(viewport.x, viewport.y);
        for action in NAVIGATION {
            if input.pressed_nav(action) {
                self.focus = toolbar::move_focus(&toolbar, self.focus, action);
                self.focus_visible = true;
            }
        }
        if input.pointer_moved() || input.last_device() == Device::Touch {
            self.focus_visible = false;
        }

        if input.pressed(Action::Confirm) {
            self.toggle_tool(self.focus);
        }
        if input.pressed(Action::Cancel) {
            self.tool = None;
        }
        if input.pointer_clicked() {
            let Some(pointer) = input.pointer() else { return };
            if top_bar::SPEED_BUTTON.contains(pointer) {
                self.stats.speed = self.stats.speed.next();
            } else if let Some(tool) = toolbar.tool_at(pointer) {
                self.focus = tool;
                self.toggle_tool(tool);
            }
        }
    }

    fn draw_ui(&self, input: &Input, viewport: Vec2) {
        let toolbar = toolbar::layout(viewport.x, viewport.y);
        let pointer = match input.last_device() {
            Device::KeyboardMouse if !self.focus_visible => input.pointer(),
            _ => None,
        };
        let speed_hovered = pointer.is_some_and(|p| top_bar::SPEED_BUTTON.contains(p));
        let bar = TopBarView {
            font: &self.font,
            stats: &self.stats,
            speed_hovered,
        };
        top_bar::draw(&bar, viewport.x);

        let view = ToolbarView {
            font: &self.font,
            icons: &self.icons,
            selected: self.tool,
            hovered: pointer.and_then(|p| toolbar.tool_at(p)),
            focus: self.focus_visible.then_some(self.focus),
            demand: DEMAND,
        };
        toolbar::draw(&toolbar, &view, viewport.x);
    }
}

// The part of the screen between the top bar and the toolbar, where the map is drawn.
pub fn map_area(viewport: Vec2) -> Rect {
    let bottom = toolbar::layout(viewport.x, viewport.y).panel.y;
    Rect::new(0.0, top_bar::HEIGHT, viewport.x, bottom - top_bar::HEIGHT)
}

impl Scene for GameplayScene {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Scene>> {
        let dt = get_frame_time();
        let viewport = screen::viewport();
        let motion = Motion {
            pan: input.pan(),
            drag: input.drag(),
            zoom_step: input.zoom_step(),
            zoom_delta: input.zoom_delta(),
        };
        self.camera.update(dt, map_area(viewport).size(), motion);
        self.view.update(dt);
        self.handle_ui(input, viewport);
        None
    }

    fn draw(&self, input: &Input) {
        clear_background(BLACK);
        let viewport = screen::viewport();
        self.view.draw(&self.camera, viewport, map_area(viewport), false);

        set_internal_camera();
        self.draw_ui(input, viewport);
    }
}
