mod layout;

use chrono::{DateTime, Local};
use macroquad::prelude::*;

use super::{gameplay::GameplayScene, menu::MenuScene, Scene};
use crate::input::{Action, Device, Input};
use crate::map::save::{self, SaveEntry};
use crate::ui::panel::{draw_button, draw_panel, draw_toggle, rgb, with_alpha, ButtonState, ERROR, TEXT};
use crate::ui::text::{draw_text_centered, draw_text_left, draw_text_right, truncate, wrap};
use crate::utils::animation::Animation;
use crate::utils::assets::{load_font, load_icons};
use crate::utils::spritesheet::Spritesheet;
use crate::{internal_width, INTERNAL_HEIGHT};
use layout::{Button, Dialog, DialogButton, Layout, ROW_HEIGHT};

const FADE_IN: f32 = 0.4;
const FADE_OUT: f32 = 0.6;
const TITLE_SIZE: u16 = 16;
const LABEL_SIZE: u16 = 8;
const ROW_PAD: f32 = 8.0;
const COLUMN_GAP: f32 = 16.0;
const DISABLED_ALPHA: f32 = 0.4;
const LINE_HEIGHT: f32 = 12.0;
const TRASH_ICON: (usize, usize) = (0, 0);
const CONFIRM_MESSAGE: &str = "Are you sure you want to delete this save?";

const LOAD_FILL: Color = rgb(62, 148, 76);
const DELETE_FILL: Color = rgb(186, 52, 44);
const OVERLAY: Color = Color::new(0.0, 0.0, 0.0, 0.6);
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
    icons: Spritesheet,
    saves: Vec<SaveEntry>,
    rows: Vec<Row>,
    selected: usize,
    scroll: f32,
    focus: Button,
    focus_visible: bool,
    deleting: bool,
    confirm: Option<DialogButton>,
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
            icons: load_icons(),
            rows: saves.iter().map(Row::new).collect(),
            saves,
            selected: 0,
            scroll: 0.0,
            focus: Button::Load,
            focus_visible: false,
            deleting: false,
            confirm: None,
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
            Button::Trash => {
                self.deleting = !self.deleting && !self.rows.is_empty();
                self.error = None;
            }
            Button::Cancel => self.leave(Next::Menu),
            Button::Load if self.deleting => {
                if self.selected < self.saves.len() {
                    self.confirm = Some(DialogButton::Cancel);
                }
            }
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

    fn activate_dialog(&mut self, button: DialogButton) {
        self.confirm = None;
        if button == DialogButton::Delete {
            self.delete_selected();
        }
    }

    fn delete_selected(&mut self) {
        let Some(entry) = self.saves.get(self.selected) else {
            return;
        };
        if let Err(err) = save::delete(&entry.path) {
            eprintln!("error: could not delete {}: {err}", entry.path.display());
            self.error = Some("Could not delete this save");
            return;
        }
        self.saves.remove(self.selected);
        self.rows.remove(self.selected);
        self.selected = self.selected.min(self.rows.len().saturating_sub(1));
        if self.rows.is_empty() {
            self.deleting = false;
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

    fn handle_dialog_input(&mut self, input: &Input, dialog: &Dialog, mut focus: DialogButton) {
        if input.pressed_nav(Action::Left) || input.pressed_nav(Action::Right) {
            focus = focus.toggle();
            self.confirm = Some(focus);
            self.focus_visible = true;
        }
        if input.pointer_moved() || input.last_device() == Device::Touch {
            self.focus_visible = false;
        }

        if input.pressed(Action::Cancel) {
            self.activate_dialog(DialogButton::Cancel);
        } else if input.pressed(Action::Confirm) {
            self.activate_dialog(focus);
        } else if input.pointer_clicked() {
            if let Some(button) = input.pointer().and_then(|p| dialog.button_at(p)) {
                self.activate_dialog(button);
            }
        }
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

        if input.pressed(Action::Cancel) && self.deleting {
            self.activate(Button::Trash);
        } else if input.pressed(Action::Cancel) {
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

        let hovered = match self.confirm {
            None if self.shows_hover() => self.hovered_row(layout),
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

    fn shows_hover(&self) -> bool {
        self.device == Device::KeyboardMouse && !self.focus_visible
    }

    fn draw_buttons(&self, layout: &Layout, alpha: f32) {
        let hovered = match self.confirm {
            None if self.shows_hover() => self.pointer.and_then(|p| layout.button_at(p)),
            _ => None,
        };
        let focused = |button| self.confirm.is_none() && self.focus_visible && self.focus == button;
        let enabled_alpha = if self.rows.is_empty() { alpha * DISABLED_ALPHA } else { alpha };

        let trash = layout.trash;
        let (pressed, focus) = (self.deleting, focused(Button::Trash));
        let trash_hovered = hovered == Some(Button::Trash);
        draw_toggle(trash, pressed, trash_hovered, DELETE_FILL, focus, enabled_alpha);
        let sink = if pressed { 1.0 } else { 0.0 };
        let icon_size = Vec2::splat(self.icons.tile_size());
        let icon = (trash.center() - icon_size / 2.0 + sink).round();
        let tint = with_alpha(WHITE, enabled_alpha);
        self.icons.draw_tile_tinted(TRASH_ICON.0, TRASH_ICON.1, icon.x, icon.y, tint);

        for button in [Button::Cancel, Button::Load] {
            let (label, fill, alpha) = match button {
                Button::Cancel => ("Cancel", None, alpha),
                Button::Load if self.deleting => ("Delete", Some(DELETE_FILL), enabled_alpha),
                _ => ("Load", Some(LOAD_FILL), enabled_alpha),
            };
            let rect = layout.button(button);
            let state = ButtonState::new(false, hovered == Some(button));
            draw_button(rect, state, fill, focused(button), alpha);
            let text = with_alpha(TEXT, alpha);
            draw_text_centered(&self.font, label, rect.center(), LABEL_SIZE, text);
        }

        if let Some(error) = self.error {
            let left = trash.x + trash.w + COLUMN_GAP;
            let max_w = layout.cancel.x - left - COLUMN_GAP;
            let error = truncate(&self.font, error, LABEL_SIZE, max_w);
            let anchor = vec2(left, layout.cancel.center().y);
            draw_text_left(&self.font, &error, anchor, LABEL_SIZE, with_alpha(ERROR, alpha));
        }
    }

    fn draw_dialog(&self, dialog: &Dialog, focus: DialogButton, alpha: f32) {
        draw_rectangle(0.0, 0.0, internal_width(), INTERNAL_HEIGHT, with_alpha(OVERLAY, alpha));
        draw_panel(dialog.panel, alpha);

        let measure = |t: &str| measure_text(t, Some(&self.font), LABEL_SIZE, 1.0).width;
        let lines = wrap(CONFIRM_MESSAGE, dialog.message.w, measure);
        let top = dialog.message.center().y - (lines.len() as f32 - 1.0) * LINE_HEIGHT / 2.0;
        let text = with_alpha(TEXT, alpha);
        for (i, line) in lines.iter().enumerate() {
            let center = vec2(dialog.message.center().x, top + i as f32 * LINE_HEIGHT);
            draw_text_centered(&self.font, line, center, LABEL_SIZE, text);
        }

        let hovered = self.pointer.filter(|_| self.shows_hover()).and_then(|p| dialog.button_at(p));
        for button in [DialogButton::Cancel, DialogButton::Delete] {
            let (label, fill) = match button {
                DialogButton::Cancel => ("Cancel", None),
                DialogButton::Delete => ("Delete", Some(DELETE_FILL)),
            };
            let rect = dialog.button(button);
            let state = ButtonState::new(false, hovered == Some(button));
            draw_button(rect, state, fill, self.focus_visible && focus == button, alpha);
            draw_text_centered(&self.font, label, rect.center(), LABEL_SIZE, text);
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
            let (w, h) = (internal_width(), INTERNAL_HEIGHT);
            match self.confirm {
                Some(focus) => self.handle_dialog_input(input, &layout::dialog(w, h), focus),
                None => self.handle_input(input, &layout::layout(w, h)),
            }
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
        if let Some(focus) = self.confirm {
            self.draw_dialog(&layout::dialog(internal_width(), INTERNAL_HEIGHT), focus, alpha);
        }
    }
}
