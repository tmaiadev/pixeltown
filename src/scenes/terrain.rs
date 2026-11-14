mod dock;

use std::path::PathBuf;

use macroquad::prelude::*;

use super::{gameplay::GameplayScene, Scene};
use crate::input::{Action, Device, Input};
use crate::map::{rng::Rng, save, Map, TerrainType};
use crate::map_view::camera::{self, Camera, Motion, MAP_CENTER};
use crate::map_view::MapView;
use crate::ui::panel::{with_alpha, ERROR};
use crate::ui::text::draw_text_centered;
use crate::utils::animation::Animation;
use crate::utils::assets::load_font;
use crate::screen::{self, set_internal_camera};
use crate::utils::seed::time_seed;
use dock::{DockButton, DockView};

const FADE_IN: f32 = 1.0;
const FADE_OUT: f32 = 1.2;
const ERROR_SIZE: u16 = 8;
const ERROR_GAP: f32 = 12.0;
const NAVIGATION: [Action; 4] = [Action::Up, Action::Down, Action::Left, Action::Right];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    FadingIn,
    Editing,
    FadingOut,
}

pub struct TerrainScene {
    font: Font,
    map: Map,
    view: MapView,
    selected: TerrainType,
    camera: Camera,
    departure: (Vec2, f32),
    save: Option<PathBuf>,
    error: Option<&'static str>,
    rng: Rng,
    phase: Phase,
    fade_in: Animation,
    fade_out: Animation,
    focus: DockButton,
    focus_visible: bool,
    grid_visible: bool,
}

impl TerrainScene {
    pub fn new() -> Self {
        let mut rng = Rng::new(time_seed());
        let selected = TerrainType::Inland;
        let map = Map::generate(selected, rng.next_u64());
        let viewport = screen::viewport();
        let camera = Camera::new(viewport);

        Self {
            font: load_font(),
            view: MapView::new(&map),
            map,
            selected,
            departure: camera.view(),
            camera,
            save: None,
            error: None,
            rng,
            phase: Phase::FadingIn,
            fade_in: Animation::new(FADE_IN).autostart(),
            fade_out: Animation::new(FADE_OUT),
            focus: DockButton::Generate,
            focus_visible: false,
            grid_visible: false,
        }
    }

    fn generate(&mut self) {
        self.map = Map::generate(self.selected, self.rng.next_u64());
        self.view.set_map(&self.map);
    }

    fn accept(&mut self) {
        match save::write(&self.map) {
            Ok(path) => self.save = Some(path),
            Err(err) => {
                eprintln!("error: could not save map: {err}");
                self.error = Some("Could not save this map");
                return;
            }
        }
        self.departure = self.camera.view();
        self.phase = Phase::FadingOut;
        self.fade_out.start();
    }

    fn gameplay(&self) -> GameplayScene {
        let loaded = self.save.as_deref().map(GameplayScene::load);
        match loaded {
            Some(Ok(scene)) => scene,
            Some(Err(err)) => {
                eprintln!("error: could not load saved map: {err}");
                GameplayScene::new(&self.map)
            }
            None => GameplayScene::new(&self.map),
        }
    }

    fn activate(&mut self, button: DockButton) {
        self.error = None;
        match button {
            DockButton::Terrain(kind) => {
                self.selected = kind;
                self.generate();
            }
            DockButton::Generate => self.generate(),
            DockButton::Accept => self.accept(),
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
            if let Some(button) = input.pointer().and_then(|p| dock.button_at(p)) {
                self.focus = button;
                self.activate(button);
            }
        }
    }

    fn draw_ui(&self, input: &Input, viewport: Vec2, alpha: f32) {
        let dock = dock::layout(viewport.x, viewport.y);
        let hovered = match input.last_device() {
            Device::KeyboardMouse if !self.focus_visible => {
                input.pointer().and_then(|p| dock.button_at(p))
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
        if let Some(error) = self.error {
            let anchor = vec2(dock.panel.center().x, dock.panel.y - ERROR_GAP);
            draw_text_centered(&self.font, error, anchor, ERROR_SIZE, with_alpha(ERROR, alpha));
        }
    }
}

impl Scene for TerrainScene {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Scene>> {
        let dt = get_frame_time();
        let viewport = screen::viewport();

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
                    return Some(Box::new(self.gameplay()));
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
        if self.phase == Phase::FadingOut {
            let home = (MAP_CENTER, GameplayScene::ZOOM);
            let (center, zoom) = camera::glide(self.departure, home, self.fade_out.progress());
            self.camera.set_view(center, zoom, viewport);
        }
        self.view.update(dt);

        if self.phase == Phase::Editing {
            if input.grid_toggled() {
                self.grid_visible = !self.grid_visible;
            }
            self.handle_dock(input, viewport);
        }
        None
    }

    fn draw(&self, input: &Input) {
        clear_background(BLACK);
        let viewport = screen::viewport();
        self.view.draw(&self.camera, viewport, self.grid_visible);

        set_internal_camera();
        let ui_alpha = match self.phase {
            Phase::FadingOut => 1.0 - self.fade_out.progress(),
            _ => 1.0,
        };
        if ui_alpha > 0.0 {
            self.draw_ui(input, viewport, ui_alpha);
        }

        if self.phase == Phase::FadingIn {
            let alpha = 1.0 - self.fade_in.progress();
            draw_rectangle(0.0, 0.0, viewport.x, viewport.y, Color::new(0.0, 0.0, 0.0, alpha));
        }
    }
}
