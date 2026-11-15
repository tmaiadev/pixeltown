use macroquad::prelude::*;

use crate::input::Action;
use crate::map::autotile::SpriteCell;
use crate::ui::panel::{draw_button, draw_panel, rgb, with_alpha, ButtonState, TEXT};
use crate::ui::text::draw_text_centered;
use crate::utils::spritesheet::Spritesheet;

const TOOL_COUNT: usize = Tool::ALL.len();
const SHEET_COLS: usize = 8;

const PAD: f32 = 4.0;
const GAP: f32 = 2.0;
const GROUP_GAP: f32 = 8.0;
const MAX_BUTTON: f32 = 28.0;
const MIN_SINGLE_ROW_BUTTON: f32 = 24.0;
const BAR_HEIGHT: f32 = 2.0;
const BAR_GAP: f32 = 1.0;
const BAR_INSET: f32 = 5.0;
const TOOLTIP_SIZE: u16 = 8;
const TOOLTIP_HEIGHT: f32 = 14.0;
const TOOLTIP_PAD: f32 = 4.0;
const TOOLTIP_GAP: f32 = 4.0;
const TOOLTIP_MARGIN: f32 = 8.0;

const DEMAND_COLORS: [Color; 4] = [
    rgb(216, 72, 64),
    rgb(64, 120, 216),
    rgb(80, 176, 72),
    rgb(232, 200, 56),
];
const BAR_TRACK: Color = rgb(20, 22, 40);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Roads,
    Electricity,
    Water,
    Waste,
    Residence,
    Commerce,
    Farming,
    Industry,
    Health,
    Police,
    Education,
    Transport,
    Entertainment,
    Bulldoze,
    Reports,
}

const GROUPS: [&[Tool]; 4] = [
    &[Tool::Roads, Tool::Electricity, Tool::Water, Tool::Waste],
    &[Tool::Residence, Tool::Commerce, Tool::Farming, Tool::Industry],
    &[Tool::Health, Tool::Police, Tool::Education, Tool::Transport],
    &[Tool::Entertainment, Tool::Bulldoze, Tool::Reports],
];

impl Tool {
    pub const ALL: [Tool; 15] = [
        Tool::Roads,
        Tool::Electricity,
        Tool::Water,
        Tool::Waste,
        Tool::Residence,
        Tool::Commerce,
        Tool::Farming,
        Tool::Industry,
        Tool::Health,
        Tool::Police,
        Tool::Education,
        Tool::Transport,
        Tool::Entertainment,
        Tool::Bulldoze,
        Tool::Reports,
    ];

    fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).expect("every tool is listed in ALL")
    }

    pub fn label(self) -> &'static str {
        match self {
            Tool::Roads => "Roads",
            Tool::Electricity => "Electricity",
            Tool::Water => "Water",
            Tool::Waste => "Waste",
            Tool::Residence => "Residence",
            Tool::Commerce => "Commerce",
            Tool::Farming => "Farming",
            Tool::Industry => "Industry",
            Tool::Health => "Health",
            Tool::Police => "Police",
            Tool::Education => "Education",
            Tool::Transport => "Transport",
            Tool::Entertainment => "Entertainment",
            Tool::Bulldoze => "Bulldoze",
            Tool::Reports => "Reports",
        }
    }

    // Icons follow `ALL` order in icons.png, right after the trash icon.
    pub fn icon(self) -> Option<SpriteCell> {
        let cell = self.index() + 1;
        (self != Tool::Reports).then(|| SpriteCell::new(cell % SHEET_COLS, cell / SHEET_COLS))
    }

    fn units(self) -> f32 {
        if self == Tool::Reports {
            2.0
        } else {
            1.0
        }
    }
}

pub struct Toolbar {
    pub panel: Rect,
    pub buttons: [Rect; TOOL_COUNT],
}

impl Toolbar {
    pub fn tool_at(&self, point: Vec2) -> Option<Tool> {
        self.buttons.iter().position(|b| b.contains(point)).map(|i| Tool::ALL[i])
    }

    pub fn button(&self, tool: Tool) -> Rect {
        self.buttons[tool.index()]
    }
}

fn button_width(tool: Tool, size: f32) -> f32 {
    tool.units() * size + (tool.units() - 1.0) * GAP
}

fn row_width(row: &[&[Tool]], size: f32) -> f32 {
    let groups = row.iter().map(|group| {
        group.iter().map(|&t| button_width(t, size)).sum::<f32>() + (group.len() - 1) as f32 * GAP
    });
    groups.sum::<f32>() + (row.len() - 1) as f32 * GROUP_GAP
}

// Largest button size that lets every row fit across the screen.
fn fit(rows: &[&[&[Tool]]], width: f32) -> f32 {
    let available = width - 2.0 * PAD;
    let size = rows
        .iter()
        .map(|row| {
            let units: f32 = row.iter().flat_map(|g| g.iter()).map(|t| t.units()).sum();
            let fixed = row_width(row, 0.0);
            (available - fixed) / units
        })
        .fold(f32::INFINITY, f32::min);
    size.floor().min(MAX_BUTTON)
}

pub fn layout(width: f32, height: f32) -> Toolbar {
    let single: [&[&[Tool]]; 1] = [&GROUPS];
    let split: [&[&[Tool]]; 2] = [&GROUPS[..2], &GROUPS[2..]];
    let rows: &[&[&[Tool]]] = if fit(&single, width) >= MIN_SINGLE_ROW_BUTTON {
        &single
    } else {
        &split
    };
    let size = fit(rows, width);

    let panel_h = rows.len() as f32 * (size + GAP) - GAP + 2.0 * PAD;
    let panel = Rect::new(0.0, height - panel_h, width, panel_h);

    let mut buttons = [Rect::default(); TOOL_COUNT];
    for (r, row) in rows.iter().enumerate() {
        let y = panel.y + PAD + r as f32 * (size + GAP);
        let mut x = (panel.x + (panel.w - row_width(row, size)) / 2.0).round();
        for group in row.iter() {
            for &tool in group.iter() {
                let w = button_width(tool, size);
                buttons[tool.index()] = Rect::new(x, y, w, size);
                x += w + GAP;
            }
            x += GROUP_GAP - GAP;
        }
    }
    Toolbar { panel, buttons }
}

// Left/Right walk the tools in order; Up/Down jump to the closest button on the next row.
pub fn move_focus(toolbar: &Toolbar, focus: Tool, action: Action) -> Tool {
    let i = focus.index();
    match action {
        Action::Left => Tool::ALL[(i + TOOL_COUNT - 1) % TOOL_COUNT],
        Action::Right => Tool::ALL[(i + 1) % TOOL_COUNT],
        Action::Up | Action::Down => {
            let from = toolbar.button(focus);
            let direction = if action == Action::Up { -1.0 } else { 1.0 };
            let rows_away = |t: Tool| (toolbar.button(t).y - from.y) * direction;
            let next_row = Tool::ALL
                .into_iter()
                .map(rows_away)
                .filter(|&d| d > 0.0)
                .fold(f32::INFINITY, f32::min);
            Tool::ALL
                .into_iter()
                .filter(|&t| rows_away(t) == next_row)
                .min_by(|&a, &b| {
                    let dist = |t: Tool| (toolbar.button(t).center().x - from.center().x).abs();
                    dist(a).total_cmp(&dist(b))
                })
                .unwrap_or(focus)
        }
        _ => focus,
    }
}

pub struct ToolbarView<'a> {
    pub font: &'a Font,
    pub icons: &'a Spritesheet,
    pub selected: Option<Tool>,
    pub hovered: Option<Tool>,
    pub focus: Option<Tool>,
    pub demand: [f32; 4],
    pub tooltip: bool,
}

pub fn draw(toolbar: &Toolbar, view: &ToolbarView, screen_width: f32) {
    draw_panel(toolbar.panel, 1.0);
    for (&rect, tool) in toolbar.buttons.iter().zip(Tool::ALL) {
        let state = ButtonState::new(view.selected == Some(tool), view.hovered == Some(tool));
        draw_button(rect, state, None, view.focus == Some(tool), 1.0);
        match tool.icon() {
            Some(cell) => {
                let size = Vec2::splat(view.icons.tile_size());
                let at = (rect.center() - size / 2.0).round();
                view.icons.draw_tile(cell, at.x, at.y);
            }
            None => draw_demand(rect, view.demand),
        }
    }
    if let Some(tool) = view.hovered.or(view.focus).filter(|_| view.tooltip) {
        draw_tooltip(tool.label(), toolbar.button(tool).center().x, toolbar.panel.y, view.font, screen_width);
    }
}

// Residence, commerce, farming and industry demand as stacked bars, each from 0 to 1.
fn draw_demand(rect: Rect, demand: [f32; 4]) {
    let max_w = rect.w - 2.0 * BAR_INSET;
    let total_h = 4.0 * BAR_HEIGHT + 3.0 * BAR_GAP;
    let top = (rect.center().y - total_h / 2.0).round();
    for (i, (value, color)) in demand.into_iter().zip(DEMAND_COLORS).enumerate() {
        let y = top + i as f32 * (BAR_HEIGHT + BAR_GAP);
        let x = rect.x + BAR_INSET;
        draw_rectangle(x, y, max_w, BAR_HEIGHT, BAR_TRACK);
        draw_rectangle(x, y, (max_w * value.clamp(0.0, 1.0)).round(), BAR_HEIGHT, color);
    }
}

// Drawn above `top`, centered on `anchor` and kept inside the screen.
pub fn draw_tooltip(label: &str, anchor: f32, top: f32, font: &Font, screen_width: f32) {
    let text_w = measure_text(label, Some(font), TOOLTIP_SIZE, 1.0).width;
    let w = (text_w + 2.0 * TOOLTIP_PAD).round();
    let max_x = (screen_width - TOOLTIP_MARGIN - w).max(TOOLTIP_MARGIN);
    let x = (anchor - w / 2.0).clamp(TOOLTIP_MARGIN, max_x).round();
    let y = top - TOOLTIP_GAP - TOOLTIP_HEIGHT;
    draw_rectangle(x, y, w, TOOLTIP_HEIGHT, with_alpha(BLACK, 0.85));
    let center = vec2(x + w / 2.0, y + TOOLTIP_HEIGHT / 2.0);
    draw_text_centered(font, label, center, TOOLTIP_SIZE, TEXT);
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

    fn rows(toolbar: &Toolbar) -> usize {
        let mut ys: Vec<f32> = toolbar.buttons.iter().map(|b| b.y).collect();
        ys.dedup();
        ys.len()
    }

    #[test]
    fn toolbar_fits_inside_the_screen_at_every_aspect_ratio() {
        for (w, h) in SIZES {
            let toolbar = layout(w, h);
            let p = toolbar.panel;
            assert!(p.x >= 0.0 && p.y >= 0.0 && p.right() <= w && p.bottom() <= h, "{w}x{h}");
            for b in toolbar.buttons {
                assert!(b.x >= p.x && b.right() <= p.right(), "{w}x{h}");
                assert!(b.y >= p.y && b.bottom() <= p.bottom(), "{w}x{h}");
            }
        }
    }

    #[test]
    fn buttons_never_overlap_each_other() {
        for (w, h) in SIZES {
            let toolbar = layout(w, h);
            for i in 0..TOOL_COUNT {
                for j in i + 1..TOOL_COUNT {
                    assert!(!toolbar.buttons[i].overlaps(&toolbar.buttons[j]), "{w}x{h} {i} {j}");
                }
            }
        }
    }

    #[test]
    fn buttons_are_big_enough_for_an_icon() {
        for (w, h) in SIZES {
            for b in layout(w, h).buttons {
                assert!(b.h >= 20.0 && b.w >= 20.0, "{w}x{h}");
            }
        }
    }

    #[test]
    fn toolbar_spans_the_full_width_flush_with_the_bottom() {
        for (w, h) in SIZES {
            let p = layout(w, h).panel;
            assert_eq!((p.x, p.w, p.bottom()), (0.0, w, h), "{w}x{h}");
        }
    }

    #[test]
    fn wide_screens_use_a_single_row() {
        assert_eq!(rows(&layout(853.0, 480.0)), 1);
        assert_eq!(rows(&layout(360.0, 480.0)), 2);
    }

    #[test]
    fn reports_spans_two_buttons() {
        let toolbar = layout(853.0, 480.0);
        let reports = toolbar.button(Tool::Reports);
        let bulldoze = toolbar.button(Tool::Bulldoze);
        assert_eq!(reports.w, 2.0 * bulldoze.w + GAP);
        assert_eq!(reports.h, bulldoze.h);
    }

    #[test]
    fn groups_are_spaced_wider_than_buttons() {
        let toolbar = layout(853.0, 480.0);
        let gap = |a: Tool, b: Tool| toolbar.button(b).x - toolbar.button(a).right();
        assert_eq!(gap(Tool::Roads, Tool::Electricity), GAP);
        assert_eq!(gap(Tool::Waste, Tool::Residence), GROUP_GAP);
    }

    #[test]
    fn tool_at_finds_the_tool_under_the_pointer() {
        let toolbar = layout(853.0, 480.0);
        let target = toolbar.button(Tool::Police);
        assert_eq!(toolbar.tool_at(target.center()), Some(Tool::Police));
        assert_eq!(toolbar.tool_at(vec2(0.0, 0.0)), None);
    }

    #[test]
    fn focus_wraps_horizontally() {
        let toolbar = layout(853.0, 480.0);
        assert_eq!(move_focus(&toolbar, Tool::Roads, Action::Left), Tool::Reports);
        assert_eq!(move_focus(&toolbar, Tool::Reports, Action::Right), Tool::Roads);
        assert_eq!(move_focus(&toolbar, Tool::Waste, Action::Right), Tool::Residence);
    }

    #[test]
    fn focus_moves_between_rows_when_split() {
        let single = layout(853.0, 480.0);
        assert_eq!(move_focus(&single, Tool::Police, Action::Up), Tool::Police);
        assert_eq!(move_focus(&single, Tool::Police, Action::Down), Tool::Police);

        let split = layout(360.0, 480.0);
        assert_eq!(move_focus(&split, Tool::Roads, Action::Down), Tool::Health);
        assert_eq!(move_focus(&split, Tool::Health, Action::Up), Tool::Roads);
        assert_eq!(move_focus(&split, Tool::Health, Action::Down), Tool::Health);
        assert_eq!(move_focus(&split, Tool::Industry, Action::Down), Tool::Reports);
    }

    #[test]
    fn every_tool_but_reports_has_its_own_icon() {
        let cells: Vec<_> = Tool::ALL.into_iter().filter_map(Tool::icon).collect();
        assert_eq!(cells.len(), TOOL_COUNT - 1);
        assert!(!cells.contains(&SpriteCell::new(0, 0)));
        for (i, a) in cells.iter().enumerate() {
            assert!(cells[i + 1..].iter().all(|b| b != a));
        }
    }

    #[test]
    fn groups_list_every_tool_in_order() {
        let flat: Vec<Tool> = GROUPS.iter().flat_map(|g| g.iter().copied()).collect();
        assert_eq!(flat, Tool::ALL);
    }
}
