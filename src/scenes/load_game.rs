mod layout;

use chrono::{DateTime, Local};
use macroquad::prelude::*;

use super::{gameplay::GameplayScene, menu::MenuScene, Scene};
use crate::input::{Action, Device, Input};
use crate::map::save::{self, SaveEntry};
use crate::ui::panel::{draw_button, draw_panel, rgb, with_alpha, ButtonState, TEXT};
use crate::ui::text::{draw_text_centered, draw_text_left, draw_text_right, truncate};
use crate::utils::animation::Animation;
use crate::utils::assets::load_font;
use crate::{internal_width, INTERNAL_HEIGHT};
use layout::{Button, Layout, ROW_HEIGHT};

const FADE_IN: f32 = 0.4;
const FADE_OUT: f32 = 0.6;
const TITLE_SIZE: u16 = 16;
const LABEL_SIZE: u16 = 8;
const ROW_PAD: f32 = 8.0;
const COLUMN_GAP: f32 = 16.0;
const DISABLED_ALPHA: f32 = 0.4;

const LOAD_FILL: Color = rgb(62, 148, 76);
const ERROR: Color = rgb(232, 96, 80);
const SCROLL_TRACK: Color = rgb(20, 22, 40);
const SCROLL_THUMB: Color = rgb(78, 86, 138);

enum Next {
    Menu,
    Gameplay(GameplayScene),
}

struct Row {
    name: String,
    modified: String,
}

impl Row {
    fn new(entry: &SaveEntry) -> Self {
        let modified: DateTime<Local> = entry.modified.into();
        Self {
            name: entry.name().to_owned(),
            modified: modified.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

pub struct LoadGameScene {
    font: Font,
    saves: Vec<SaveEntry>,
    rows: Vec<Row>,
    selected: usize,
    scroll: f32,
    focus: Button,
    focus_visible: bool,
    device: Device,
    pointer: Option<Vec2>,
    error: Option<&'static str>,
    fade_in: Animation,
    fade_out: Animation,
    next: Option<Next>,
}

impl LoadGameScene {
    pub fn new() -> Self {
        let saves = save::list();
        Self {
            font: load_font(),
            rows: saves.iter().map(Row::new).collect(),
            saves,
            selected: 0,
            scroll: 0.0,
            focus: Button::Load,
            focus_visible: false,
            device: Device::KeyboardMouse,
            pointer: None,
            error: None,
            fade_in: Animation::new(FADE_IN).autostart(),
            fade_out: Animation::new(FADE_OUT),
            next: None,
        }
    }

    fn leave(&mut self, next: Next) {
        self.next = Some(next);
        self.fade_out.start();
    }

    fn activate(&mut self, button: Button) {
        match button {
            Button::Cancel => self.leave(Next::Menu),
            Button::Load => {
                let Some(entry) = self.saves.get(self.selected) else {
                    return;
                };
                match GameplayScene::load(&entry.path) {
                    Ok(scene) => self.leave(Next::Gameplay(scene)),
                    Err(err) => {
                        eprintln!("error: could not load {}: {err}", entry.path.display());
                        self.error = Some("Could not load this save");
                    }
                }
            }
        }
    }

    fn select(&mut self, index: usize, visible: usize) {
        if index != self.selected {
            self.error = None;
        }
        self.selected = index;
        self.scroll = layout::reveal(self.scroll, index, visible);
    }

    fn first_row(&self) -> usize {
        self.scroll.round() as usize
    }

    fn hovered_row(&self, layout: &Layout) -> Option<usize> {
        let slot = self.pointer.and_then(|p| layout.slot_at(p))?;
        Some(self.first_row() + slot).filter(|&i| i < self.rows.len())
    }

    fn handle_input(&mut self, input: &Input, layout: &Layout) {
        let visible = layout.visible_rows();
        let len = self.rows.len();

        if input.pressed(Action::Up) && len > 0 {
            self.select((self.selected + len - 1) % len, visible);
            self.focus_visible = true;
        }
        if input.pressed(Action::Down) && len > 0 {
            self.select((self.selected + 1) % len, visible);
            self.focus_visible = true;
        }
        for action in [Action::Left, Action::Right] {
            if input.pressed_nav(action) {
                self.focus = layout::move_focus(self.focus, action);
                self.focus_visible = true;
            }
        }
        if input.pointer_moved() || input.last_device() == Device::Touch {
            self.focus_visible = false;
        }

        let scrolled = input.scroll() - input.drag().y / ROW_HEIGHT;
        self.scroll = (self.scroll + scrolled).clamp(0.0, layout::max_scroll(len, visible));

        if input.pressed(Action::Cancel) {
            self.activate(Button::Cancel);
        } else if input.pressed(Action::Confirm) {
            self.activate(self.focus);
        } else if input.pointer_clicked() {
            if let Some(i) = self.hovered_row(layout) {
                self.select(i, visible);
            } else if let Some(button) = input.pointer().and_then(|p| layout.button_at(p)) {
                self.focus = button;
                self.activate(button);
            }
        }
    }

    fn draw_rows(&self, layout: &Layout, alpha: f32) {
        if self.rows.is_empty() {
            let text = with_alpha(TEXT, alpha);
            draw_text_centered(&self.font, "No saved games", layout.list.center(), LABEL_SIZE, text);
            return;
        }

        let hovered = match self.device {
            Device::KeyboardMouse if !self.focus_visible => self.hovered_row(layout),
            _ => None,
        };
        let rows = self.rows.iter().enumerate().skip(self.first_row());
        for (slot, (i, row)) in rows.take(layout.visible_rows()).enumerate() {
            let rect = layout.row(slot);
            let state = ButtonState::new(i == self.selected, hovered == Some(i));
            draw_button(rect, state, None, false, alpha);

            let color = with_alpha(state.text_color(), alpha);
            let measure = |t: &str| measure_text(t, Some(&self.font), LABEL_SIZE, 1.0).width;
            let inner_w = rect.w - 2.0 * ROW_PAD;
            let show_date = measure(&row.name) + COLUMN_GAP + measure(&row.modified) <= inner_w;
            let y = rect.y + rect.h / 2.0;
            if show_date {
                let right = vec2(rect.x + rect.w - ROW_PAD, y);
                draw_text_right(&self.font, &row.modified, right, LABEL_SIZE, color);
            }
            let name = truncate(&self.font, &row.name, LABEL_SIZE, inner_w);
            draw_text_left(&self.font, &name, vec2(rect.x + ROW_PAD, y), LABEL_SIZE, color);
        }
    }

    fn draw_scrollbar(&self, layout: &Layout, alpha: f32) {
        let (len, visible) = (self.rows.len(), layout.visible_rows());
        if len <= visible {
            return;
        }
        let track = layout.scrollbar;
        draw_rectangle(track.x, track.y, track.w, track.h, with_alpha(SCROLL_TRACK, alpha));
        let thumb_h = (track.h * visible as f32 / len as f32).round();
        let travel = track.h - thumb_h;
        let thumb_y = track.y + (travel * self.scroll / layout::max_scroll(len, visible)).round();
        draw_rectangle(track.x, thumb_y, track.w, thumb_h, with_alpha(SCROLL_THUMB, alpha));
    }

    fn draw_buttons(&self, layout: &Layout, alpha: f32) {
        let hovered = match self.device {
            Device::KeyboardMouse if !self.focus_visible => {
                self.pointer.and_then(|p| layout.button_at(p))
            }
            _ => None,
        };
        for button in [Button::Cancel, Button::Load] {
            let (fill, alpha) = match button {
                Button::Cancel => (None, alpha),
                Button::Load if self.rows.is_empty() => (Some(LOAD_FILL), alpha * DISABLED_ALPHA),
                Button::Load => (Some(LOAD_FILL), alpha),
            };
            let rect = layout.button(button);
            let state = ButtonState::new(false, hovered == Some(button));
            let focused = self.focus_visible && self.focus == button;
            draw_button(rect, state, fill, focused, alpha);
            let text = with_alpha(TEXT, alpha);
            draw_text_centered(&self.font, button.label(), rect.center(), LABEL_SIZE, text);
        }

        if let Some(error) = self.error {
            let max_w = layout.cancel.x - layout.list.x - COLUMN_GAP;
            let error = truncate(&self.font, error, LABEL_SIZE, max_w);
            let anchor = vec2(layout.list.x, layout.cancel.center().y);
            draw_text_left(&self.font, &error, anchor, LABEL_SIZE, with_alpha(ERROR, alpha));
        }
    }
}

impl Scene for LoadGameScene {
    fn update(&mut self, input: &Input) -> Option<Box<dyn Scene>> {
        let dt = get_frame_time();
        self.device = input.last_device();
        self.pointer = input.pointer();
        self.fade_in.update(dt);
        if self.fade_out.has_started() {
            self.fade_out.update(dt);
            if !self.fade_out.has_ended() {
                return None;
            }
            return match self.next.take()? {
                Next::Menu => Some(Box::new(MenuScene::new())),
                Next::Gameplay(scene) => Some(Box::new(scene)),
            };
        }
        if self.fade_in.has_ended() {
            let layout = layout::layout(internal_width(), INTERNAL_HEIGHT);
            self.handle_input(input, &layout);
        }
        None
    }

    fn draw(&self) {
        clear_background(BLACK);
        let layout = layout::layout(internal_width(), INTERNAL_HEIGHT);
        let alpha = self.fade_in.progress() * (1.0 - self.fade_out.progress());

        draw_panel(layout.panel, alpha);
        let title = with_alpha(TEXT, alpha);
        draw_text_centered(&self.font, "Load Game", layout.title, TITLE_SIZE, title);
        self.draw_rows(&layout, alpha);
        self.draw_scrollbar(&layout, alpha);
        self.draw_buttons(&layout, alpha);
    }
}
