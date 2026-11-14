use macroquad::prelude::*;

pub const TEXT: Color = rgb(240, 240, 255);
pub const TEXT_DARK: Color = rgb(30, 24, 10);
pub const ERROR: Color = rgb(232, 96, 80);
const BORDER: Color = rgb(12, 12, 24);
const PANEL_FILL: Color = rgb(34, 38, 66);
const BEVEL_LIGHT: Color = rgb(78, 86, 138);
const BEVEL_SHADOW: Color = rgb(20, 22, 40);
const BUTTON_FILL: Color = rgb(54, 60, 104);
const BUTTON_HOVER: Color = rgb(74, 82, 136);
const SELECTED_FILL: Color = rgb(224, 176, 56);
const SELECTED_HOVER: Color = rgb(244, 200, 88);
const FOCUS_RING: Color = rgb(255, 255, 255);

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

pub fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, color.a * alpha)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonState {
    Normal,
    Hovered,
    Selected,
    SelectedHovered,
}

impl ButtonState {
    pub fn new(selected: bool, hovered: bool) -> Self {
        match (selected, hovered) {
            (false, false) => Self::Normal,
            (false, true) => Self::Hovered,
            (true, false) => Self::Selected,
            (true, true) => Self::SelectedHovered,
        }
    }

    pub fn is_selected(self) -> bool {
        matches!(self, Self::Selected | Self::SelectedHovered)
    }

    pub fn text_color(self) -> Color {
        if self.is_selected() {
            TEXT_DARK
        } else {
            TEXT
        }
    }
}

fn bevel_box(rect: Rect, fill: Color, light: Color, shadow: Color, alpha: f32) {
    let tint = |c| with_alpha(c, alpha);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, tint(BORDER));
    draw_rectangle(rect.x + 1.0, rect.y + 1.0, rect.w - 2.0, rect.h - 2.0, tint(fill));
    draw_rectangle(rect.x + 1.0, rect.y + 1.0, rect.w - 2.0, 1.0, tint(light));
    draw_rectangle(rect.x + 1.0, rect.y + 1.0, 1.0, rect.h - 2.0, tint(light));
    draw_rectangle(rect.x + 1.0, rect.y + rect.h - 2.0, rect.w - 2.0, 1.0, tint(shadow));
    draw_rectangle(rect.x + rect.w - 2.0, rect.y + 1.0, 1.0, rect.h - 2.0, tint(shadow));
}

pub fn draw_panel(rect: Rect, alpha: f32) {
    bevel_box(rect, PANEL_FILL, BEVEL_LIGHT, BEVEL_SHADOW, alpha);
}

pub fn draw_button(rect: Rect, state: ButtonState, fill: Option<Color>, focused: bool, alpha: f32) {
    let (base, hover) = match fill {
        Some(color) => (color, lighten(color)),
        None => (BUTTON_FILL, BUTTON_HOVER),
    };
    let fill = match state {
        ButtonState::Normal => base,
        ButtonState::Hovered => hover,
        ButtonState::Selected => SELECTED_FILL,
        ButtonState::SelectedHovered => SELECTED_HOVER,
    };
    bevel_box(rect, fill, lighten(fill), darken(fill), alpha);
    if focused {
        draw_focus_ring(rect, alpha);
    }
}

// Pressed toggles sink: darker fill and the bevel flips.
pub fn draw_toggle(rect: Rect, pressed: bool, hovered: bool, fill: Color, focused: bool, alpha: f32) {
    if !pressed {
        draw_button(rect, ButtonState::new(false, hovered), Some(fill), focused, alpha);
        return;
    }
    let fill = if hovered { fill } else { darken(fill) };
    bevel_box(rect, fill, darken(fill), lighten(fill), alpha);
    if focused {
        draw_focus_ring(rect, alpha);
    }
}

fn draw_focus_ring(rect: Rect, alpha: f32) {
    let ring = with_alpha(FOCUS_RING, alpha);
    draw_rectangle_lines(rect.x - 2.0, rect.y - 2.0, rect.w + 4.0, rect.h + 4.0, 2.0, ring);
}

fn lighten(color: Color) -> Color {
    Color::new(
        (color.r + 0.18).min(1.0),
        (color.g + 0.18).min(1.0),
        (color.b + 0.18).min(1.0),
        color.a,
    )
}

fn darken(color: Color) -> Color {
    Color::new(color.r * 0.6, color.g * 0.6, color.b * 0.6, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_state_combines_selection_and_hover() {
        assert_eq!(ButtonState::new(false, false), ButtonState::Normal);
        assert_eq!(ButtonState::new(false, true), ButtonState::Hovered);
        assert_eq!(ButtonState::new(true, false), ButtonState::Selected);
        assert_eq!(ButtonState::new(true, true), ButtonState::SelectedHovered);
    }

    #[test]
    fn with_alpha_scales_existing_alpha() {
        let color = with_alpha(Color::new(1.0, 0.5, 0.0, 0.5), 0.5);
        assert_eq!(color.a, 0.25);
        assert_eq!(color.r, 1.0);
    }
}
