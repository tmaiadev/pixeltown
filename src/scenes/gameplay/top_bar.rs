use chrono::{Datelike, NaiveDate};
use macroquad::prelude::*;

use crate::ui::panel::{with_alpha, TEXT};
use crate::ui::text::{draw_text_left, draw_text_right};

pub const HEIGHT: f32 = 16.0;
const PAD: f32 = 8.0;
const MIN_GAP: f32 = 16.0;
const TEXT_SIZE: u16 = 8;
const SPEED_WIDTH: f32 = 2.0 * PAD + 16.0;
const PERSON_WIDTH: f32 = 5.0;
const PERSON_GAP: f32 = 4.0;
const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Speed {
    Paused,
    Normal,
    Fast,
    Fastest,
}

impl Speed {
    pub fn next(self) -> Self {
        match self {
            Speed::Paused => Speed::Normal,
            Speed::Normal => Speed::Fast,
            Speed::Fast => Speed::Fastest,
            Speed::Fastest => Speed::Paused,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Speed::Paused => "||",
            Speed::Normal => "1x",
            Speed::Fast => "2x",
            Speed::Fastest => "3x",
        }
    }
}

pub struct Stats {
    pub speed: Speed,
    pub cash: i64,
    pub date: NaiveDate,
    pub population: u64,
}

// Thousands are grouped with dots, e.g. 200.000.
pub fn format_number(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    if value < 0 {
        grouped.insert(0, '-');
    }
    grouped
}

pub fn format_date(date: NaiveDate) -> String {
    format!("{:02}/{}/{}", date.day(), MONTHS[date.month0() as usize], date.year())
}

pub const SPEED_BUTTON: Rect = Rect::new(0.0, 0.0, SPEED_WIDTH, HEIGHT);

pub struct TopBarView<'a> {
    pub font: &'a Font,
    pub stats: &'a Stats,
    pub speed_hovered: bool,
}

pub fn draw(view: &TopBarView, width: f32) {
    draw_rectangle(0.0, 0.0, width, HEIGHT, BLACK);
    let speed = SPEED_BUTTON;
    if view.speed_hovered {
        draw_rectangle(speed.x, speed.y, speed.w, speed.h, with_alpha(TEXT, 0.2));
    }

    let middle = HEIGHT / 2.0;
    let text_width = |text: &str| measure_text(text, Some(view.font), TEXT_SIZE, 1.0).width;
    let cash = format!("$ {}", format_number(view.stats.cash));
    let date = format_date(view.stats.date);
    let population = format_number(view.stats.population as i64);

    draw_text_left(view.font, view.stats.speed.label(), vec2(PAD, middle), TEXT_SIZE, TEXT);
    let population_x = width - PAD - text_width(&population);
    draw_text_right(view.font, &population, vec2(width - PAD, middle), TEXT_SIZE, TEXT);
    draw_person(vec2(population_x - PERSON_GAP - PERSON_WIDTH, middle));

    // Date and cash share the space between speed and population, dropping the date when cramped.
    let (left, right) = (SPEED_WIDTH, population_x - PERSON_GAP - PERSON_WIDTH);
    let middle_items: Vec<&str> = if text_width(&cash) + text_width(&date) + 3.0 * MIN_GAP <= right - left {
        vec![&date, &cash]
    } else {
        vec![&cash]
    };
    let used: f32 = middle_items.iter().map(|t| text_width(t)).sum();
    let gap = (right - left - used) / (middle_items.len() + 1) as f32;
    let mut x = left + gap;
    for item in middle_items {
        draw_text_left(view.font, item, vec2(x, middle), TEXT_SIZE, TEXT);
        x += text_width(item) + gap;
    }
}

// A tiny head-and-shoulders figure, vertically centered on `anchor.y`.
fn draw_person(anchor: Vec2) {
    let (x, y) = (anchor.x.round(), (anchor.y - 4.0).round());
    draw_rectangle(x + 1.0, y, 3.0, 3.0, TEXT);
    draw_rectangle(x, y + 4.0, 5.0, 4.0, TEXT);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_group_thousands_with_dots() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1_432), "1.432");
        assert_eq!(format_number(200_000), "200.000");
        assert_eq!(format_number(1_234_567), "1.234.567");
        assert_eq!(format_number(-1_500), "-1.500");
    }

    #[test]
    fn dates_use_day_short_month_and_year() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
        assert_eq!(format_date(date), "15/nov/2026");
        let date = NaiveDate::from_ymd_opt(1999, 1, 2).unwrap();
        assert_eq!(format_date(date), "02/jan/1999");
    }

    #[test]
    fn speed_cycles_through_every_option() {
        let mut speed = Speed::Normal;
        let mut seen = vec![speed];
        for _ in 0..3 {
            speed = speed.next();
            seen.push(speed);
        }
        assert_eq!(speed.next(), Speed::Normal);
        assert!(seen.contains(&Speed::Paused) && seen.contains(&Speed::Fastest));
    }
}
