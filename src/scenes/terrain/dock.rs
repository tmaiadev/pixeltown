use macroquad::prelude::*;

use crate::input::Action;
use crate::map::TerrainType;
use crate::ui::panel::{draw_button, draw_panel, rgb, with_alpha, ButtonState, TEXT};
use crate::ui::text::draw_text_centered;

const TYPE_COUNT: usize = TerrainType::ALL.len();
pub const GENERATE: usize = TYPE_COUNT;
pub const ACCEPT: usize = TYPE_COUNT + 1;
pub const BUTTON_COUNT: usize = TYPE_COUNT + 2;

const PAD: f32 = 8.0;
const GAP: f32 = 6.0;
const GROUP_GAP: f32 = 16.0;
const MARGIN: f32 = 8.0;
const BUTTON_HEIGHT: f32 = 36.0;
const TYPE_WIDTH: f32 = 96.0;
const ACTION_WIDTH: f32 = 88.0;
const LABEL_SIZE: u16 = 8;
const ICON_SIZE: Vec2 = Vec2::new(24.0, 12.0);

const GRASS: Color = rgb(90, 158, 58);
const SAND: Color = rgb(230, 208, 140);
const WATER: Color = rgb(47, 111, 192);
const GENERATE_FILL: Color = rgb(58, 104, 168);
const ACCEPT_FILL: Color = rgb(62, 148, 76);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DockButton {
    Terrain(TerrainType),
    Generate,
    Accept,
}

impl DockButton {
    pub fn at(index: usize) -> Self {
        match index {
            GENERATE => Self::Generate,
            ACCEPT => Self::Accept,
            i => Self::Terrain(TerrainType::ALL[i]),
        }
    }
}

enum Arrangement {
    Wide,
    Medium,
    Narrow,
}

pub struct Dock {
    pub panel: Rect,
    pub buttons: [Rect; BUTTON_COUNT],
}

impl Dock {
    pub fn button_at(&self, point: Vec2) -> Option<usize> {
        self.buttons.iter().position(|b| b.contains(point))
    }
}

fn button(x: f32, y: f32, w: f32) -> Rect {
    Rect::new(x.round(), y.round(), w.round(), BUTTON_HEIGHT)
}

pub fn layout(width: f32, height: f32) -> Dock {
    let types_w = TYPE_COUNT as f32 * TYPE_WIDTH + (TYPE_COUNT - 1) as f32 * GAP;
    let actions_w = 2.0 * ACTION_WIDTH + GAP;
    let wide_w = types_w + GROUP_GAP + actions_w + 2.0 * PAD;
    let medium_w = types_w + 2.0 * PAD;

    let arrangement = if width >= wide_w + 2.0 * MARGIN {
        Arrangement::Wide
    } else if width >= medium_w + 2.0 * MARGIN {
        Arrangement::Medium
    } else {
        Arrangement::Narrow
    };
    let (panel_w, panel_h) = match arrangement {
        Arrangement::Wide => (wide_w, BUTTON_HEIGHT + 2.0 * PAD),
        Arrangement::Medium => (medium_w, 2.0 * BUTTON_HEIGHT + GAP + 2.0 * PAD),
        Arrangement::Narrow => (
            width - 2.0 * MARGIN,
            3.0 * BUTTON_HEIGHT + 2.0 * GAP + 2.0 * PAD,
        ),
    };
    let panel = Rect::new(
        ((width - panel_w) / 2.0).round(),
        (height - MARGIN - panel_h).round(),
        panel_w.round(),
        panel_h,
    );
    let (left, top, inner_w) = (panel.x + PAD, panel.y + PAD, panel.w - 2.0 * PAD);

    let mut buttons = [Rect::default(); BUTTON_COUNT];
    let (types, actions) = buttons.split_at_mut(GENERATE);
    match arrangement {
        Arrangement::Wide => {
            let mut x = left;
            for slot in types {
                *slot = button(x, top, TYPE_WIDTH);
                x += TYPE_WIDTH + GAP;
            }
            x += GROUP_GAP - GAP;
            for slot in actions {
                *slot = button(x, top, ACTION_WIDTH);
                x += ACTION_WIDTH + GAP;
            }
        }
        Arrangement::Medium => {
            for (i, slot) in types.iter_mut().enumerate() {
                *slot = button(left + i as f32 * (TYPE_WIDTH + GAP), top, TYPE_WIDTH);
            }
            place_action_row(actions, left, top + BUTTON_HEIGHT + GAP, inner_w);
        }
        Arrangement::Narrow => {
            let half = (inner_w - GAP) / 2.0;
            for (i, slot) in types.iter_mut().enumerate() {
                let (col, row) = ((i % 2) as f32, (i / 2) as f32);
                *slot = button(
                    left + col * (half + GAP),
                    top + row * (BUTTON_HEIGHT + GAP),
                    half,
                );
            }
            place_action_row(actions, left, top + 2.0 * (BUTTON_HEIGHT + GAP), inner_w);
        }
    }

    Dock { panel, buttons }
}

fn place_action_row(actions: &mut [Rect], left: f32, y: f32, inner_w: f32) {
    let w = (inner_w - GAP) / 2.0;
    actions[0] = button(left, y, w);
    actions[1] = button(left + w + GAP, y, w);
}

pub fn move_focus(focus: usize, action: Action) -> usize {
    match action {
        Action::Left => (focus + BUTTON_COUNT - 1) % BUTTON_COUNT,
        Action::Right => (focus + 1) % BUTTON_COUNT,
        Action::Up | Action::Down if focus >= GENERATE => focus - GENERATE,
        Action::Up | Action::Down => GENERATE + focus / 2,
        _ => focus,
    }
}

fn draw_icon(kind: TerrainType, origin: Vec2, alpha: f32) {
    let tint = |c| with_alpha(c, alpha);
    let rect = |x: f32, y: f32, w: f32, h: f32, c: Color| {
        draw_rectangle(origin.x + x, origin.y + y, w, h, tint(c));
    };
    match kind {
        TerrainType::Inland => {
            rect(0.0, 0.0, 24.0, 12.0, GRASS);
            rect(10.0, 0.0, 4.0, 12.0, SAND);
            rect(11.0, 0.0, 2.0, 12.0, WATER);
        }
        TerrainType::Island => {
            rect(0.0, 0.0, 24.0, 12.0, WATER);
            rect(4.0, 2.0, 16.0, 8.0, SAND);
            rect(5.0, 3.0, 14.0, 6.0, GRASS);
        }
        TerrainType::Archipelago => {
            rect(0.0, 0.0, 24.0, 12.0, WATER);
            rect(7.0, 3.0, 10.0, 6.0, SAND);
            rect(8.0, 4.0, 8.0, 4.0, GRASS);
            rect(1.0, 1.0, 3.0, 3.0, SAND);
            rect(19.0, 8.0, 3.0, 3.0, SAND);
        }
        TerrainType::Seaside => {
            rect(0.0, 0.0, 24.0, 12.0, GRASS);
            rect(0.0, 6.0, 24.0, 2.0, SAND);
            rect(0.0, 8.0, 24.0, 4.0, WATER);
        }
    }
}

pub struct DockView<'a> {
    pub font: &'a Font,
    pub selected: TerrainType,
    pub hovered: Option<usize>,
    pub focus: Option<usize>,
    pub alpha: f32,
}

pub fn draw(dock: &Dock, view: &DockView) {
    draw_panel(dock.panel, view.alpha);

    for (i, &rect) in dock.buttons.iter().enumerate() {
        let hovered = view.hovered == Some(i);
        let focused = view.focus == Some(i);
        match DockButton::at(i) {
            DockButton::Terrain(kind) => draw_terrain_button(rect, kind, view, hovered, focused),
            DockButton::Generate => {
                draw_action_button(rect, "Generate", GENERATE_FILL, view, hovered, focused)
            }
            DockButton::Accept => {
                draw_action_button(rect, "Accept", ACCEPT_FILL, view, hovered, focused)
            }
        }
    }
}

fn draw_terrain_button(rect: Rect, kind: TerrainType, view: &DockView, hovered: bool, focused: bool) {
    let state = ButtonState::new(kind == view.selected, hovered);
    draw_button(rect, state, None, focused, view.alpha);
    let center_x = rect.x + rect.w / 2.0;
    let icon_at = vec2((center_x - ICON_SIZE.x / 2.0).round(), rect.y + 6.0);
    draw_icon(kind, icon_at, view.alpha);
    draw_text_centered(
        view.font,
        kind.label(),
        vec2(center_x, rect.y + rect.h - 10.0),
        LABEL_SIZE,
        with_alpha(state.text_color(), view.alpha),
    );
}

fn draw_action_button(
    rect: Rect,
    label: &str,
    fill: Color,
    view: &DockView,
    hovered: bool,
    focused: bool,
) {
    let state = ButtonState::new(false, hovered);
    draw_button(rect, state, Some(fill), focused, view.alpha);
    draw_text_centered(view.font, label, rect.center(), LABEL_SIZE, with_alpha(TEXT, view.alpha));
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZES: [(f32, f32); 6] = [
        (853.0, 480.0),
        (1120.0, 480.0),
        (640.0, 480.0),
        (500.0, 480.0),
        (360.0, 480.0),
        (220.0, 480.0),
    ];

    #[test]
    fn dock_fits_inside_the_screen_at_every_aspect_ratio() {
        for (w, h) in SIZES {
            let dock = layout(w, h);
            let screen = Rect::new(0.0, 0.0, w, h);
            assert!(screen.contains(vec2(dock.panel.x, dock.panel.y)), "{w}x{h}");
            assert!(dock.panel.x + dock.panel.w <= w, "{w}x{h}");
            assert!(dock.panel.y + dock.panel.h <= h, "{w}x{h}");
            for b in dock.buttons {
                assert!(b.x >= dock.panel.x && b.x + b.w <= dock.panel.x + dock.panel.w);
                assert!(b.y >= dock.panel.y && b.y + b.h <= dock.panel.y + dock.panel.h);
            }
        }
    }

    #[test]
    fn buttons_never_overlap_each_other() {
        for (w, h) in SIZES {
            let dock = layout(w, h);
            for i in 0..BUTTON_COUNT {
                for j in i + 1..BUTTON_COUNT {
                    assert!(!dock.buttons[i].overlaps(&dock.buttons[j]), "{w}x{h} {i} {j}");
                }
            }
        }
    }

    #[test]
    fn layout_uses_fewer_rows_on_wider_screens() {
        let rows = |w: f32| layout(w, 480.0).panel.h;
        assert!(rows(1120.0) < rows(500.0));
        assert!(rows(500.0) < rows(220.0));
    }

    #[test]
    fn buttons_are_big_enough_to_tap() {
        for (w, h) in SIZES {
            for b in layout(w, h).buttons {
                assert!(b.h >= 32.0 && b.w >= 64.0, "{w}x{h}");
            }
        }
    }

    #[test]
    fn button_at_finds_the_button_under_the_pointer() {
        let dock = layout(853.0, 480.0);
        let target = dock.buttons[GENERATE];
        let inside = vec2(target.x + target.w / 2.0, target.y + target.h / 2.0);
        assert_eq!(dock.button_at(inside), Some(GENERATE));
        assert_eq!(dock.button_at(vec2(0.0, 0.0)), None);
    }

    #[test]
    fn focus_wraps_horizontally_and_jumps_between_groups() {
        assert_eq!(move_focus(0, Action::Left), ACCEPT);
        assert_eq!(move_focus(ACCEPT, Action::Right), 0);
        assert_eq!(move_focus(2, Action::Right), 3);
        assert_eq!(move_focus(GENERATE, Action::Up), 0);
        assert_eq!(move_focus(3, Action::Down), ACCEPT);
        assert_eq!(move_focus(1, Action::Up), GENERATE);
        assert_eq!(move_focus(2, Action::Up), ACCEPT);
        assert_eq!(move_focus(ACCEPT, Action::Down), 1);
        assert_eq!(move_focus(2, Action::Confirm), 2);
    }
}
