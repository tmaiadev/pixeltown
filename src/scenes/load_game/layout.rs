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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    Cancel,
    Load,
}

impl Button {
    pub fn label(self) -> &'static str {
        match self {
            Button::Cancel => "Cancel",
            Button::Load => "Load",
        }
    }
}

pub fn move_focus(focus: Button, action: Action) -> Button {
    match (focus, action) {
        (Button::Cancel, Action::Left | Action::Right) => Button::Load,
        (Button::Load, Action::Left | Action::Right) => Button::Cancel,
        _ => focus,
    }
}

pub struct Layout {
    pub panel: Rect,
    pub title: Vec2,
    pub list: Rect,
    pub scrollbar: Rect,
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
            Button::Cancel => self.cancel,
            Button::Load => self.load,
        }
    }

    pub fn button_at(&self, point: Vec2) -> Option<Button> {
        [Button::Cancel, Button::Load]
            .into_iter()
            .find(|&b| self.button(b).contains(point))
    }
}

pub fn layout(width: f32, height: f32) -> Layout {
    let panel_w = (width - 2.0 * MARGIN).min(MAX_WIDTH).round();
    let panel = Rect::new(((width - panel_w) / 2.0).round(), TOP, panel_w, height - 2.0 * TOP);
    let (left, inner_w) = (panel.x + PAD, panel.w - 2.0 * PAD);

    let buttons_y = panel.y + panel.h - PAD - BUTTON_HEIGHT;
    let button_w = BUTTON_WIDTH.min((inner_w - GAP) / 2.0).round();
    let right = panel.x + panel.w - PAD;
    let load = Rect::new(right - button_w, buttons_y, button_w, BUTTON_HEIGHT);
    let cancel = Rect::new(load.x - GAP - button_w, buttons_y, button_w, BUTTON_HEIGHT);

    let list_y = panel.y + PAD + TITLE_HEIGHT;
    let list_h = buttons_y - GAP - list_y;
    let list = Rect::new(left, list_y, inner_w - SCROLLBAR_WIDTH - GAP, list_h);
    let scrollbar = Rect::new(left + inner_w - SCROLLBAR_WIDTH, list_y, SCROLLBAR_WIDTH, list_h);

    Layout {
        panel,
        title: vec2(panel.x + panel.w / 2.0, panel.y + PAD + TITLE_HEIGHT / 2.0),
        list,
        scrollbar,
        cancel,
        load,
    }
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
            for r in [l.list, l.scrollbar, l.cancel, l.load] {
                assert!(r.x >= l.panel.x && r.x + r.w <= l.panel.x + l.panel.w, "{w}x{h}");
                assert!(r.y >= l.panel.y && r.y + r.h <= l.panel.y + l.panel.h, "{w}x{h}");
            }
            assert!(!l.cancel.overlaps(&l.load), "{w}x{h}");
            assert!(l.row(l.visible_rows() - 1).bottom() <= l.list.bottom(), "{w}x{h}");
        }
    }

    #[test]
    fn slot_at_finds_rows_and_misses_outside() {
        let l = layout(853.0, 480.0);
        assert_eq!(l.slot_at(l.row(2).center()), Some(2));
        assert_eq!(l.slot_at(vec2(0.0, 0.0)), None);
        assert_eq!(l.button_at(l.load.center()), Some(Button::Load));
        assert_eq!(l.button_at(l.cancel.center()), Some(Button::Cancel));
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
    fn focus_toggles_between_buttons() {
        assert_eq!(move_focus(Button::Load, Action::Left), Button::Cancel);
        assert_eq!(move_focus(Button::Cancel, Action::Right), Button::Load);
        assert_eq!(move_focus(Button::Load, Action::Up), Button::Load);
    }
}
