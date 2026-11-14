use macroquad::prelude::*;

use crate::input::Action;

const MAX_WIDTH: f32 = 560.0;
const MARGIN: f32 = 8.0;
const TOP: f32 = 24.0;
const PAD: f32 = 8.0;
const GAP: f32 = 6.0;
const TITLE_HEIGHT: f32 = 32.0;
const BUTTON_HEIGHT: f32 = 36.0;
const BUTTON_WIDTH: f32 = 88.0;
pub const ROW_HEIGHT: f32 = 24.0;
const ROW_GAP: f32 = 2.0;
const SCROLLBAR_WIDTH: f32 = 4.0;
const DIALOG_WIDTH: f32 = 320.0;
const MESSAGE_HEIGHT: f32 = 44.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    Trash,
    Cancel,
    Load,
}

impl Button {
    pub const ALL: [Button; 3] = [Button::Trash, Button::Cancel, Button::Load];
}

pub fn move_focus(focus: Button, action: Action) -> Button {
    let i = Button::ALL.iter().position(|&b| b == focus).unwrap_or(0);
    let len = Button::ALL.len();
    match action {
        Action::Left => Button::ALL[(i + len - 1) % len],
        Action::Right => Button::ALL[(i + 1) % len],
        _ => focus,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DialogButton {
    Cancel,
    Delete,
}

impl DialogButton {
    pub fn toggle(self) -> Self {
        match self {
            DialogButton::Cancel => DialogButton::Delete,
            DialogButton::Delete => DialogButton::Cancel,
        }
    }
}

pub struct Layout {
    pub panel: Rect,
    pub title: Vec2,
    pub list: Rect,
    pub scrollbar: Rect,
    pub trash: Rect,
    pub cancel: Rect,
    pub load: Rect,
}

impl Layout {
    pub fn visible_rows(&self) -> usize {
        ((self.list.h + ROW_GAP) / (ROW_HEIGHT + ROW_GAP)).floor() as usize
    }

    pub fn row(&self, slot: usize) -> Rect {
        let y = self.list.y + slot as f32 * (ROW_HEIGHT + ROW_GAP);
        Rect::new(self.list.x, y, self.list.w, ROW_HEIGHT)
    }

    pub fn slot_at(&self, point: Vec2) -> Option<usize> {
        (0..self.visible_rows()).find(|&slot| self.row(slot).contains(point))
    }

    pub fn button(&self, button: Button) -> Rect {
        match button {
            Button::Trash => self.trash,
            Button::Cancel => self.cancel,
            Button::Load => self.load,
        }
    }

    pub fn button_at(&self, point: Vec2) -> Option<Button> {
        Button::ALL.into_iter().find(|&b| self.button(b).contains(point))
    }
}

pub struct Dialog {
    pub panel: Rect,
    pub message: Rect,
    pub cancel: Rect,
    pub delete: Rect,
}

impl Dialog {
    pub fn button(&self, button: DialogButton) -> Rect {
        match button {
            DialogButton::Cancel => self.cancel,
            DialogButton::Delete => self.delete,
        }
    }

    pub fn button_at(&self, point: Vec2) -> Option<DialogButton> {
        [DialogButton::Cancel, DialogButton::Delete]
            .into_iter()
            .find(|&b| self.button(b).contains(point))
    }
}

// Width of each of two buttons sharing `space`.
fn button_width(space: f32) -> f32 {
    BUTTON_WIDTH.min((space - GAP) / 2.0).round()
}

pub fn layout(width: f32, height: f32) -> Layout {
    let panel_w = (width - 2.0 * MARGIN).min(MAX_WIDTH).round();
    let panel = Rect::new(((width - panel_w) / 2.0).round(), TOP, panel_w, height - 2.0 * TOP);
    let (left, inner_w) = (panel.x + PAD, panel.w - 2.0 * PAD);

    let buttons_y = panel.y + panel.h - PAD - BUTTON_HEIGHT;
    let button_w = button_width(inner_w - BUTTON_HEIGHT - GAP);
    let right = panel.x + panel.w - PAD;
    let load = Rect::new(right - button_w, buttons_y, button_w, BUTTON_HEIGHT);
    let cancel = Rect::new(load.x - GAP - button_w, buttons_y, button_w, BUTTON_HEIGHT);
    let trash = Rect::new(left, buttons_y, BUTTON_HEIGHT, BUTTON_HEIGHT);

    let list_y = panel.y + PAD + TITLE_HEIGHT;
    let list_h = buttons_y - GAP - list_y;
    let list = Rect::new(left, list_y, inner_w - SCROLLBAR_WIDTH - GAP, list_h);
    let scrollbar = Rect::new(left + inner_w - SCROLLBAR_WIDTH, list_y, SCROLLBAR_WIDTH, list_h);

    Layout {
        panel,
        title: vec2(panel.x + panel.w / 2.0, panel.y + PAD + TITLE_HEIGHT / 2.0),
        list,
        scrollbar,
        trash,
        cancel,
        load,
    }
}

pub fn dialog(width: f32, height: f32) -> Dialog {
    let panel_w = (width - 2.0 * MARGIN).min(DIALOG_WIDTH).round();
    let panel_h = 2.0 * PAD + MESSAGE_HEIGHT + GAP + BUTTON_HEIGHT;
    let (x, y) = (((width - panel_w) / 2.0).round(), ((height - panel_h) / 2.0).round());
    let panel = Rect::new(x, y, panel_w, panel_h);
    let inner_w = panel.w - 2.0 * PAD;
    let message = Rect::new(panel.x + PAD, panel.y + PAD, inner_w, MESSAGE_HEIGHT);

    let buttons_y = message.y + MESSAGE_HEIGHT + GAP;
    let button_w = button_width(inner_w);
    let right = panel.x + panel.w - PAD;
    let delete = Rect::new(right - button_w, buttons_y, button_w, BUTTON_HEIGHT);
    let cancel = Rect::new(delete.x - GAP - button_w, buttons_y, button_w, BUTTON_HEIGHT);

    Dialog { panel, message, cancel, delete }
}

pub fn max_scroll(len: usize, visible: usize) -> f32 {
    len.saturating_sub(visible) as f32
}

// Smallest scroll change that brings `selected` into view.
pub fn reveal(scroll: f32, selected: usize, visible: usize) -> f32 {
    let selected = selected as f32;
    if selected < scroll {
        selected
    } else if selected >= scroll + visible as f32 {
        selected + 1.0 - visible as f32
    } else {
        scroll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZES: [(f32, f32); 4] = [(853.0, 480.0), (1120.0, 480.0), (360.0, 480.0), (220.0, 480.0)];

    #[test]
    fn everything_fits_inside_the_panel_at_every_aspect_ratio() {
        for (w, h) in SIZES {
            let l = layout(w, h);
            assert!(l.panel.x >= 0.0 && l.panel.x + l.panel.w <= w, "{w}x{h}");
            assert!(l.panel.y + l.panel.h <= h, "{w}x{h}");
            for r in [l.list, l.scrollbar, l.trash, l.cancel, l.load] {
                assert!(r.x >= l.panel.x && r.x + r.w <= l.panel.x + l.panel.w, "{w}x{h}");
                assert!(r.y >= l.panel.y && r.y + r.h <= l.panel.y + l.panel.h, "{w}x{h}");
            }
            assert!(!l.cancel.overlaps(&l.load), "{w}x{h}");
            assert!(!l.trash.overlaps(&l.cancel), "{w}x{h}");
            assert!(l.row(l.visible_rows() - 1).bottom() <= l.list.bottom(), "{w}x{h}");
        }
    }

    #[test]
    fn dialog_fits_on_screen_at_every_aspect_ratio() {
        for (w, h) in SIZES {
            let d = dialog(w, h);
            assert!(d.panel.x >= 0.0 && d.panel.x + d.panel.w <= w, "{w}x{h}");
            for r in [d.message, d.cancel, d.delete] {
                assert!(r.x >= d.panel.x && r.x + r.w <= d.panel.x + d.panel.w, "{w}x{h}");
                assert!(r.y >= d.panel.y && r.y + r.h <= d.panel.y + d.panel.h, "{w}x{h}");
            }
            assert!(!d.cancel.overlaps(&d.delete), "{w}x{h}");
            assert_eq!(d.button_at(d.delete.center()), Some(DialogButton::Delete));
        }
    }

    #[test]
    fn slot_at_finds_rows_and_misses_outside() {
        let l = layout(853.0, 480.0);
        assert_eq!(l.slot_at(l.row(2).center()), Some(2));
        assert_eq!(l.slot_at(vec2(0.0, 0.0)), None);
        assert_eq!(l.button_at(l.load.center()), Some(Button::Load));
        assert_eq!(l.button_at(l.cancel.center()), Some(Button::Cancel));
        assert_eq!(l.button_at(l.trash.center()), Some(Button::Trash));
    }

    #[test]
    fn reveal_scrolls_only_when_selection_leaves_the_view() {
        assert_eq!(reveal(0.0, 3, 10), 0.0);
        assert_eq!(reveal(0.0, 10, 10), 1.0);
        assert_eq!(reveal(5.0, 2, 10), 2.0);
        assert_eq!(max_scroll(3, 10), 0.0);
        assert_eq!(max_scroll(15, 10), 5.0);
    }

    #[test]
    fn focus_cycles_through_buttons() {
        assert_eq!(move_focus(Button::Load, Action::Left), Button::Cancel);
        assert_eq!(move_focus(Button::Cancel, Action::Left), Button::Trash);
        assert_eq!(move_focus(Button::Trash, Action::Left), Button::Load);
        assert_eq!(move_focus(Button::Load, Action::Right), Button::Trash);
        assert_eq!(move_focus(Button::Load, Action::Up), Button::Load);
        assert_eq!(DialogButton::Cancel.toggle(), DialogButton::Delete);
    }
}
