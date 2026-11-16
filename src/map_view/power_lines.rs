use macroquad::prelude::*;

use super::camera::{TileRect, TILE_SIZE};
use super::trees::shown;
use crate::map::autotile::{WireCell, Wiring};
use crate::utils::perspective::Perspective;

const POLE_HEIGHT: f32 = 9.0;
// Wires with no pole under them hang lower than the ones between poles.
const SPAN_HEIGHT: f32 = 6.0;
const POLE_WIDTH: f32 = 2.0;
const ARM_WIDTH: f32 = 6.0;
const WIRE_SAG: f32 = 1.5;
const SHADOW_SKEW: Vec2 = Vec2::new(3.0, 2.0);
const POLE: Color = Color::new(0.43, 0.30, 0.19, 1.0);
const ARM: Color = Color::new(0.30, 0.20, 0.12, 1.0);
const WIRE: Color = Color::new(0.10, 0.10, 0.13, 1.0);
const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.27);

// A wire leaving a tile center: where it ends on the ground and how high.
struct Wire {
    to: Vec2,
    height: f32,
    sags: bool,
}

fn center(cell: &WireCell) -> Vec2 {
    (vec2(cell.x as f32, cell.y as f32) + 0.5) * TILE_SIZE
}

fn height(wiring: Wiring) -> f32 {
    match wiring {
        Wiring::Pole => POLE_HEIGHT,
        Wiring::Span(_) => SPAN_HEIGHT,
    }
}

// Wires to the linked neighbors this tile owns, plus the ones running off the tile's open sides.
fn wires(cell: &WireCell) -> impl Iterator<Item = Wire> + '_ {
    let from = center(cell);
    let neighbor = |wiring: Option<Wiring>, step: Vec2| {
        wiring.map(|other| Wire {
            to: from + step * TILE_SIZE,
            height: height(other),
            sags: cell.wiring == Wiring::Pole && other == Wiring::Pole,
        })
    };
    let open = |bit: u8, step: Vec2| {
        (cell.open & bit != 0).then(|| Wire { to: from + step * TILE_SIZE / 2.0, height: height(cell.wiring), sags: false })
    };
    [
        neighbor(cell.east, vec2(1.0, 0.0)),
        neighbor(cell.south, vec2(0.0, 1.0)),
        open(1, vec2(0.0, -1.0)),
        open(2, vec2(1.0, 0.0)),
        open(4, vec2(0.0, 1.0)),
        open(8, vec2(-1.0, 0.0)),
    ]
    .into_iter()
    .flatten()
}

fn top(ground: Vec2, height: f32, perspective: &Perspective) -> Vec2 {
    ground + perspective.lean(ground, height)
}

pub fn draw_shadows(cells: &[WireCell], visible: TileRect) {
    for cell in shown(cells, |c| (c.x, c.y), visible) {
        let ground = center(cell) + SHADOW_SKEW;
        if cell.wiring == Wiring::Pole {
            let foot = center(cell);
            draw_line(foot.x, foot.y, ground.x, ground.y, 1.0, SHADOW);
        }
        for wire in wires(cell) {
            let to = wire.to + SHADOW_SKEW;
            draw_line(ground.x, ground.y, to.x, to.y, 1.0, SHADOW);
        }
    }
}

pub fn draw_poles(cells: &[WireCell], visible: TileRect, perspective: &Perspective) {
    for cell in shown(cells, |c| (c.x, c.y), visible) {
        let ground = center(cell);
        let from = top(ground, height(cell.wiring), perspective);
        for wire in wires(cell) {
            let to = top(wire.to, wire.height, perspective);
            let mid = wire.sags.then(|| {
                let halfway = (ground + wire.to) / 2.0;
                top(halfway, POLE_HEIGHT - WIRE_SAG, perspective)
            });
            draw_wire(from, to, mid);
        }
        if cell.wiring == Wiring::Pole {
            draw_pole(ground, from);
        }
    }
}

// Seen from straight above, a pole is a dot.
fn draw_pole(ground: Vec2, top: Vec2) {
    if top == ground {
        draw_rectangle(top.x - POLE_WIDTH / 2.0, top.y - POLE_WIDTH / 2.0, POLE_WIDTH, POLE_WIDTH, POLE);
        return;
    }
    draw_line(ground.x, ground.y, top.x, top.y, POLE_WIDTH, POLE);
    draw_line(top.x - ARM_WIDTH / 2.0, top.y, top.x + ARM_WIDTH / 2.0, top.y, 1.0, ARM);
}

// A wire between two poles hangs from a midpoint that sits a little lower.
fn draw_wire(from: Vec2, to: Vec2, mid: Option<Vec2>) {
    let mid = mid.unwrap_or(to);
    draw_line(from.x, from.y, mid.x, mid.y, 1.0, WIRE);
    if mid != to {
        draw_line(mid.x, mid.y, to.x, to.y, 1.0, WIRE);
    }
}
