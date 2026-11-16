use macroquad::prelude::*;

use super::toolbar::{draw_tooltip, Toolbar};
use crate::input::Action;
use crate::map::autotile::SpriteCell;
use crate::map::buildings::BuildingKind;
use crate::ui::panel::{draw_button, draw_panel, ButtonState};
use crate::utils::spritesheet::Spritesheet;

const SHEET_COLS: usize = 8;
const FIRST_ICON_ROW: usize = 2;
const PAD: f32 = 4.0;
const GAP: f32 = 2.0;
const GROUP_GAP: f32 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerItem {
    PowerLine,
    Coal,
    Oil,
    Gas,
    Hydroelectric,
    Solar,
    Wind,
    Nuclear,
}

const GROUPS: [&[PowerItem]; 3] = [
    &[PowerItem::PowerLine],
    &[PowerItem::Coal, PowerItem::Oil, PowerItem::Gas],
    &[PowerItem::Hydroelectric, PowerItem::Solar, PowerItem::Wind, PowerItem::Nuclear],
];

impl PowerItem {
    pub const ALL: [PowerItem; 8] = [
        PowerItem::PowerLine,
        PowerItem::Coal,
        PowerItem::Oil,
        PowerItem::Gas,
        PowerItem::Hydroelectric,
        PowerItem::Solar,
        PowerItem::Wind,
        PowerItem::Nuclear,
    ];

    fn index(self) -> usize {
        Self::ALL.iter().position(|&i| i == self).expect("every item is listed in ALL")
    }

    pub fn label(self) -> &'static str {
        match self {
            PowerItem::PowerLine => "Power Line",
            PowerItem::Coal => "Coal Power Plant",
            PowerItem::Oil => "Oil Power Plant",
            PowerItem::Gas => "Gas Power Plant",
            PowerItem::Hydroelectric => "Hydroelectric Plant",
            PowerItem::Solar => "Solar Power Plant",
            PowerItem::Wind => "Wind Power Plant",
            PowerItem::Nuclear => "Nuclear Power Plant",
        }
    }

    // What this item builds, when it is a building rather than a power line.
    pub fn building(self) -> Option<BuildingKind> {
        match self {
            PowerItem::Coal => Some(BuildingKind::CoalPlant),
            PowerItem::Oil => Some(BuildingKind::OilPlant),
            _ => None,
        }
    }

    pub fn icon(self) -> SpriteCell {
        let cell = FIRST_ICON_ROW * SHEET_COLS + self.index();
        SpriteCell::new(cell % SHEET_COLS, cell / SHEET_COLS)
    }
}

pub struct Submenu {
    pub panel: Rect,
    pub buttons: [Rect; PowerItem::ALL.len()],
}

impl Submenu {
    pub fn item_at(&self, point: Vec2) -> Option<PowerItem> {
        self.buttons.iter().position(|b| b.contains(point)).map(|i| PowerItem::ALL[i])
    }

    pub fn contains(&self, point: Vec2) -> bool {
        self.panel.contains(point)
    }
}

// Sits flush on top of the toolbar, centered on the screen.
pub fn layout(toolbar: &Toolbar, button_size: f32) -> Submenu {
    let count = PowerItem::ALL.len() as f32;
    let fixed = (count - GROUPS.len() as f32) * GAP + (GROUPS.len() - 1) as f32 * GROUP_GAP;
    let fitting = ((toolbar.panel.w - 2.0 * PAD - fixed) / count).floor();
    let size = button_size.min(fitting);
    let width = count * size + fixed;
    let panel_w = width + 2.0 * PAD;
    let x = ((toolbar.panel.w - panel_w) / 2.0).round().max(0.0);
    let panel_h = size + 2.0 * PAD;
    let panel = Rect::new(x, toolbar.panel.y - panel_h, panel_w, panel_h);

    let mut buttons = [Rect::default(); PowerItem::ALL.len()];
    let mut bx = panel.x + PAD;
    for group in GROUPS {
        for &item in group {
            buttons[item.index()] = Rect::new(bx, panel.y + PAD, size, size);
            bx += size + GAP;
        }
        bx += GROUP_GAP - GAP;
    }
    Submenu { panel, buttons }
}

// Focus is trapped: Left/Right wrap around, Up/Down stay put.
pub fn move_focus(focus: PowerItem, action: Action) -> PowerItem {
    let count = PowerItem::ALL.len();
    let i = focus.index();
    match action {
        Action::Left => PowerItem::ALL[(i + count - 1) % count],
        Action::Right => PowerItem::ALL[(i + 1) % count],
        _ => focus,
    }
}

pub struct SubmenuView<'a> {
    pub font: &'a Font,
    pub icons: &'a Spritesheet,
    pub hovered: Option<PowerItem>,
    pub focus: Option<PowerItem>,
}

pub fn draw(submenu: &Submenu, view: &SubmenuView, screen_width: f32) {
    draw_panel(submenu.panel, 1.0);
    for (&rect, item) in submenu.buttons.iter().zip(PowerItem::ALL) {
        let state = ButtonState::new(false, view.hovered == Some(item));
        draw_button(rect, state, None, view.focus == Some(item), 1.0);
        let size = Vec2::splat(view.icons.tile_size());
        let at = (rect.center() - size / 2.0).round();
        view.icons.draw_tile(item.icon(), at.x, at.y);
    }
    if let Some(item) = view.hovered.or(view.focus) {
        let anchor = submenu.buttons[item.index()].center().x;
        draw_tooltip(item.label(), anchor, submenu.panel.y, view.font, screen_width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenes::gameplay::toolbar::{self, Tool};

    #[test]
    fn submenu_sits_above_the_toolbar_inside_the_screen() {
        for (w, h) in [(853.0, 480.0), (360.0, 480.0), (220.0, 480.0)] {
            let bar = toolbar::layout(w, h);
            let menu = layout(&bar, bar.button(Tool::Electricity).h);
            assert!(menu.panel.x >= 0.0 && menu.panel.right() <= w, "{w}x{h}");
            assert_eq!(menu.panel.bottom(), bar.panel.y, "{w}x{h}");
            assert!((menu.panel.x - (w - menu.panel.right())).abs() <= 1.0, "{w}x{h}");
            for b in menu.buttons {
                assert!(b.x >= menu.panel.x && b.right() <= menu.panel.right(), "{w}x{h}");
            }
        }
    }

    #[test]
    fn focus_wraps_and_ignores_vertical_moves() {
        assert_eq!(move_focus(PowerItem::PowerLine, Action::Left), PowerItem::Nuclear);
        assert_eq!(move_focus(PowerItem::Nuclear, Action::Right), PowerItem::PowerLine);
        assert_eq!(move_focus(PowerItem::Coal, Action::Up), PowerItem::Coal);
    }

    #[test]
    fn only_the_coal_and_oil_plants_build_a_building() {
        let builders: Vec<_> = PowerItem::ALL.into_iter().filter(|i| i.building().is_some()).collect();
        assert_eq!(builders, vec![PowerItem::Coal, PowerItem::Oil]);
    }

    #[test]
    fn items_have_distinct_icons() {
        for (i, a) in PowerItem::ALL.iter().enumerate() {
            assert!(PowerItem::ALL[i + 1..].iter().all(|b| b.icon() != a.icon()));
        }
    }
}
