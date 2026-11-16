mod submenu;
mod toolbar;
mod top_bar;

use std::path::Path;

use chrono::Local;
use macroquad::prelude::*;

use super::Scene;
use crate::input::{Action, Device, Input};
use crate::map::buildings::BuildingKind;
use crate::map::save::{self, SaveError};
use crate::map::{Map, TerrainType, MAP_SIZE};
use crate::map_view::camera::{Camera, Motion, TILE_SIZE};
use crate::map_view::MapView;
use crate::screen::{self, set_internal_camera};
use crate::utils::assets::{load_font, load_icons};
use crate::utils::seed::time_seed;
use crate::utils::spritesheet::Spritesheet;
use submenu::{PowerItem, SubmenuView};
use toolbar::{Tool, ToolbarView};
use top_bar::{Speed, Stats, TopBarView};

const NAVIGATION: [Action; 4] = [Action::Up, Action::Down, Action::Left, Action::Right];
// Placeholder residence, commerce, farming and industry demand until the simulation drives it.
const DEMAND: [f32; 4] = [1.0, 0.5, 0.7, 0.2];

pub struct GameplayScene {
    font: Font,
    icons: Spritesheet,
    map: Map,
    view: MapView,
    camera: Camera,
    stats: Stats,
    tool: Option<Tool>,
    focus: Tool,
    focus_visible: bool,
    power_focus: PowerItem,
    placing: Option<PowerItem>,
    cursor: Option<(i32, i32)>,
    pointer_cursor: bool,
    drag_from: Option<(i32, i32)>,
    built: Vec<(i32, i32)>,
}

impl GameplayScene {
    pub const ZOOM: f32 = 1.0;

    pub fn new(map: &Map) -> Self {
        let area = map_area(screen::viewport());
        Self {
            font: load_font(),
            icons: load_icons(),
            map: map.clone(),
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
            power_focus: PowerItem::PowerLine,
            placing: None,
            cursor: None,
            pointer_cursor: false,
            drag_from: None,
            built: Vec::new(),
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
        self.cursor = None;
        self.power_focus = PowerItem::PowerLine;
        self.placing = None;
    }

    fn submenu_open(&self) -> bool {
        self.tool == Some(Tool::Electricity) && self.placing.is_none()
    }

    fn building(&self) -> bool {
        match self.tool {
            Some(Tool::Roads | Tool::Bulldoze) => true,
            Some(Tool::Electricity) => self.placing.is_some(),
            _ => false,
        }
    }

    // Roads, power lines and the bulldozer paint along a drag; a building is placed once per click.
    fn paints(&self) -> bool {
        self.building() && self.placing.and_then(PowerItem::building).is_none()
    }

    // Only power lines and the coal, oil and gas plants can be built so far; the other plants have no effect yet.
    fn pick_power_item(&mut self, item: PowerItem) {
        self.power_focus = item;
        if item == PowerItem::PowerLine || item.building().is_some() {
            self.placing = Some(item);
            self.cursor = None;
        }
    }

    fn placing_building(&self) -> Option<BuildingKind> {
        match self.tool {
            Some(Tool::Electricity) => self.placing.and_then(PowerItem::building),
            _ => None,
        }
    }

    fn apply_tool(&mut self) {
        let Some((x, y)) = self.cursor else { return };
        self.apply_at(x, y);
    }

    fn apply_at(&mut self, x: i32, y: i32) {
        // A building changes every tile it covers, not just the one aimed at.
        let mut touched = vec![(x, y)];
        let changed = match (self.tool, self.placing_building()) {
            (Some(Tool::Roads), _) => self.map.build_road(x, y),
            (Some(Tool::Bulldoze), _) => {
                if let Some(building) = Map::in_bounds(x, y).then(|| self.map.building_at(x as usize, y as usize)).flatten() {
                    touched = building.tiles().collect();
                }
                self.map.bulldoze(x, y)
            }
            (Some(Tool::Electricity), Some(kind)) => {
                touched = kind.tiles(kind.origin((x, y))).collect();
                self.map.build_building(kind, (x, y))
            }
            (Some(Tool::Electricity), None) => self.map.build_power_line(x, y),
            _ => false,
        };
        if changed {
            self.built.extend(touched);
        }
    }

    // The tiles the tool would act on at the cursor, each with whether it can be built on.
    fn preview(&self, x: i32, y: i32) -> Vec<((i32, i32), bool)> {
        if let Some(kind) = self.placing_building() {
            // Red marks the tiles in the way; a footprint sticking out of the map is red all over.
            let tiles: Vec<_> = kind.tiles(kind.origin((x, y))).collect();
            let inside = tiles.iter().all(|&(tx, ty)| Map::in_bounds(tx, ty));
            return tiles
                .into_iter()
                .filter(|&(tx, ty)| Map::in_bounds(tx, ty))
                .map(|tile| (tile, inside && self.map.can_build_building_on(tile.0, tile.1)))
                .collect();
        }
        let buildable = match self.tool {
            Some(Tool::Roads) => self.map.can_build_road(x, y),
            Some(Tool::Bulldoze) => self.map.can_bulldoze(x, y),
            Some(Tool::Electricity) => self.map.can_build_power_line(x, y),
            _ => false,
        };
        vec![((x, y), buildable)]
    }

    // Applies the tool between the last painted tile and the cursor while the button is held.
    fn drag_tool(&mut self, input: &Input) {
        if !input.pointer_held() || !self.paints() {
            self.drag_from = None;
            return;
        }
        let (Some(from), Some(to)) = (self.drag_from, self.cursor) else { return };
        if from != to {
            for (x, y) in line(from, to) {
                self.apply_at(x, y);
            }
            self.drag_from = Some(to);
        }
    }

    fn move_cursor(&mut self, action: Action, viewport: Vec2) {
        let area = map_area(viewport).size();
        let (dx, dy) = match action {
            Action::Up => (0, -1),
            Action::Down => (0, 1),
            Action::Left => (-1, 0),
            _ => (1, 0),
        };
        let center = self.camera.view_center() / TILE_SIZE;
        let start = (center.x as i32, center.y as i32);
        let (x, y) = self.cursor.map_or(start, |(x, y)| (x + dx, y + dy));
        let max = MAP_SIZE as i32 - 1;
        let tile = (x.clamp(0, max), y.clamp(0, max));
        self.cursor = Some(tile);
        self.pointer_cursor = false;
        self.camera.reveal(tile, area);
    }

    fn handle_ui(&mut self, input: &Input, viewport: Vec2) {
        let toolbar = toolbar::layout(viewport.x, viewport.y);
        for action in NAVIGATION {
            if input.pressed_nav(action) {
                if self.submenu_open() {
                    self.power_focus = submenu::move_focus(self.power_focus, action);
                    self.focus_visible = true;
                    continue;
                }
                if self.building() {
                    self.move_cursor(action, viewport);
                    continue;
                }
                self.focus = toolbar::move_focus(&toolbar, self.focus, action);
                self.focus_visible = true;
            }
        }
        if input.pointer_moved() || input.last_device() == Device::Touch {
            self.focus_visible = false;
            self.pointer_cursor = true;
        }

        if input.pressed(Action::Confirm) {
            if self.submenu_open() {
                self.pick_power_item(self.power_focus);
            } else if self.building() {
                self.apply_tool();
            } else {
                self.toggle_tool(self.focus);
            }
        }
        if input.pressed(Action::Cancel) {
            self.tool = None;
            self.cursor = None;
        }
        if input.pointer_clicked() {
            let Some(pointer) = input.pointer() else { return };
            let submenu = self.submenu_open().then(|| self.submenu(&toolbar));
            if let Some(item) = submenu.as_ref().and_then(|m| m.item_at(pointer)) {
                self.pick_power_item(item);
            } else if submenu.is_some_and(|m| m.contains(pointer)) {
            } else if top_bar::SPEED_BUTTON.contains(pointer) {
                self.stats.speed = self.stats.speed.next();
            } else if let Some(tool) = toolbar.tool_at(pointer) {
                self.focus = tool;
                self.toggle_tool(tool);
            } else if self.building() {
                self.cursor = self.camera.tile_at(pointer, map_area(viewport));
                self.apply_tool();
                self.drag_from = self.cursor;
            }
        }
    }

    fn submenu(&self, toolbar: &toolbar::Toolbar) -> submenu::Submenu {
        submenu::layout(toolbar, toolbar.button(Tool::Electricity).h)
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
            tooltip: !self.submenu_open(),
        };
        toolbar::draw(&toolbar, &view, viewport.x);

        if self.submenu_open() {
            let menu = self.submenu(&toolbar);
            let view = SubmenuView {
                font: &self.font,
                icons: &self.icons,
                hovered: pointer.and_then(|p| menu.item_at(p)),
                focus: self.focus_visible.then_some(self.power_focus),
            };
            submenu::draw(&menu, &view, viewport.x);
        }
    }
}

impl GameplayScene {
    // Small tool icon next to the pointer, or next to the cursor tile when driven by keys or a gamepad.
    fn draw_cursor_icon(&self, input: &Input, area: Rect) {
        let icon = match self.tool {
            _ if !self.building() => return,
            Some(Tool::Electricity) => match self.placing {
                Some(item) => item.icon(),
                None => return,
            },
            tool => match tool.and_then(Tool::icon) {
                Some(icon) => icon,
                None => return,
            },
        };
        let at = match (self.pointer_cursor, input.pointer(), self.cursor) {
            (true, Some(pointer), _) if area.contains(pointer) => pointer + vec2(6.0, 6.0),
            (false, _, Some(tile)) => self.camera.tile_origin(tile, area) + Vec2::splat(TILE_SIZE * self.camera.zoom() * 0.75),
            _ => return,
        };
        self.icons.draw_tile(icon, at.x.round(), at.y.round());
    }
}

// Tiles along a straight line, both ends included.
fn line(from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
    let (dx, dy) = ((to.0 - from.0).abs(), -(to.1 - from.1).abs());
    let (sx, sy) = ((to.0 - from.0).signum(), (to.1 - from.1).signum());
    let (mut x, mut y, mut err) = (from.0, from.1, dx + dy);
    let mut tiles = vec![(x, y)];
    while (x, y) != to {
        let doubled = 2 * err;
        if doubled >= dy {
            err += dy;
            x += sx;
        }
        if doubled <= dx {
            err += dx;
            y += sy;
        }
        tiles.push((x, y));
    }
    tiles
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
        if self.building() && self.pointer_cursor {
            let area = map_area(viewport);
            self.cursor = input.pointer().and_then(|p| self.camera.tile_at(p, area));
        }
        self.handle_ui(input, viewport);
        self.drag_tool(input);
        if !self.built.is_empty() {
            self.view.update_tiles(&self.map, &self.built);
            self.built.clear();
        }
        None
    }

    fn draw(&self, input: &Input) {
        clear_background(BLACK);
        let viewport = screen::viewport();
        let area = map_area(viewport);
        self.view.draw(&self.camera, viewport, area, self.building());
        if let (true, Some((x, y))) = (self.building(), self.cursor) {
            self.view.draw_highlight(&self.camera, viewport, area, self.preview(x, y));
        }

        set_internal_camera();
        self.draw_ui(input, viewport);
        self.draw_cursor_icon(input, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_includes_both_ends_and_has_no_gaps() {
        let tiles = line((2, 3), (7, 5));
        assert_eq!((tiles[0], *tiles.last().unwrap()), ((2, 3), (7, 5)));
        assert!(tiles.windows(2).all(|w| (w[1].0 - w[0].0).abs() <= 1 && (w[1].1 - w[0].1).abs() <= 1));
        assert_eq!(line((4, 4), (4, 4)), vec![(4, 4)]);
    }
}
