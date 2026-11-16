mod coal_plant;
mod effects;
mod gas_plant;
mod oil_plant;

use macroquad::prelude::*;

use super::camera::{TileRect, TILE_SIZE};
use crate::map::buildings::{Building, BuildingKind};
use crate::utils::perspective::{draw_quad, draw_strip, Perspective};
use crate::utils::spritesheet::Spritesheet;

// Chimney smoke climbs high and the lean stretches it, so buildings off screen can still show up.
const MARGIN: i32 = 10;
// Shadow cast per pixel of height, toward the bottom right.
const SHADOW_PER_HEIGHT: Vec2 = Vec2::new(0.33, 0.22);
const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.27);
// Light comes from the top left: walls facing north and west are lit, the others are in shade.
const NORTH: f32 = 0.9;
const WEST: f32 = 0.95;
const EAST: f32 = 0.62;
const SOUTH: f32 = 0.72;
const BASE: Color = Color::new(206.0 / 255.0, 208.0 / 255.0, 210.0 / 255.0, 1.0);
const GLOW: Color = Color::new(1.0, 0.55, 0.15, 0.2);

// A box rising from the ground. Its sprites are all pieces of the buildings sheet.
pub struct Block {
    // Footprint in pixels, relative to the top-left corner of the building.
    pub base: Rect,
    pub height: f32,
    // Seen from above; stretched over the raised top.
    pub roof: Rect,
    // Seen from the front; one for the north and south sides, one for the east and west.
    pub wall_ns: Rect,
    pub wall_ew: Rect,
}

// A round chimney. Its body runs from the ground (first row of the sprite) to the top (last row).
pub struct Stack {
    // Center of the base, relative to the top-left corner of the building.
    pub base: Vec2,
    pub height: f32,
    pub body: Rect,
    pub cap: Rect,
    // Whether it gives off fire and smoke.
    pub plume: bool,
}

// How a kind of building looks: a pad on the ground, boxes and chimneys rising from it.
pub struct Model {
    // Seen from above; covers the whole footprint and is baked into the map's ground.
    pub yard: Rect,
    pub blocks: &'static [Block],
    pub stacks: &'static [Stack],
}

fn model(kind: BuildingKind) -> &'static Model {
    match kind {
        BuildingKind::CoalPlant => &coal_plant::MODEL,
        BuildingKind::OilPlant => &oil_plant::MODEL,
        BuildingKind::GasPlant => &gas_plant::MODEL,
    }
}

fn origin(building: Building) -> Vec2 {
    vec2(building.x as f32, building.y as f32) * TILE_SIZE
}

fn area(building: Building) -> Rect {
    let (w, h) = building.kind.footprint();
    let origin = origin(building);
    Rect::new(origin.x, origin.y, w as f32 * TILE_SIZE, h as f32 * TILE_SIZE)
}

fn shown(buildings: &[Building], visible: TileRect) -> impl Iterator<Item = &Building> {
    let TileRect { x0, y0, x1, y1 } = visible;
    buildings.iter().filter(move |b| {
        let (w, h) = b.kind.footprint();
        let (x, y) = (b.x as i32, b.y as i32);
        x + w as i32 > x0 as i32 - MARGIN
            && x < x1 as i32 + MARGIN
            && y + h as i32 > y0 as i32 - MARGIN
            && y < y1 as i32 + MARGIN
    })
}

// Pads are flat, so they are drawn into the ground of the chunk (map area `chunk`) they touch.
pub fn draw_yards(sheet: &Spritesheet, buildings: &[Building], chunk: Rect) {
    for &building in buildings {
        let area = area(building);
        if area.overlaps(&chunk) {
            sheet.draw_sprite(model(building.kind).yard, area, WHITE);
        }
    }
}

pub fn draw_shadows(buildings: &[Building], visible: TileRect) {
    for &building in shown(buildings, visible) {
        let origin = origin(building);
        let model = model(building.kind);
        for block in model.blocks {
            let polygon = shadow(block.base.offset(origin), SHADOW_PER_HEIGHT * block.height);
            for i in 1..polygon.len() - 1 {
                draw_triangle(polygon[0], polygon[i], polygon[i + 1], SHADOW);
            }
        }
        for stack in model.stacks {
            let from = origin + stack.base;
            let to = from + SHADOW_PER_HEIGHT * stack.height;
            draw_line(from.x, from.y, to.x, to.y, stack.body.w, SHADOW);
        }
    }
}

// Outline of the ground a box shades when the shadow falls toward the bottom right by `skew`.
fn shadow(base: Rect, skew: Vec2) -> [Vec2; 6] {
    let (min, max) = (base.point(), base.point() + base.size());
    [
        min,
        vec2(max.x, min.y),
        vec2(max.x, min.y) + skew,
        max + skew,
        vec2(min.x, max.y) + skew,
        vec2(min.x, max.y),
    ]
}

// Things of a building that can hide one another, drawn farthest from the viewer first.
enum Part<'a> {
    Block(&'a Block),
    Stack(&'a Stack),
}

impl Part<'_> {
    fn height(&self) -> f32 {
        match self {
            Part::Block(block) => block.height,
            Part::Stack(stack) => stack.height,
        }
    }

    fn distance(&self, origin: Vec2, view: Vec2) -> f32 {
        match self {
            Part::Block(block) => {
                let base = block.base.offset(origin);
                let nearest = view.clamp(base.point(), base.point() + base.size());
                nearest.distance(view)
            }
            Part::Stack(stack) => (origin + stack.base).distance(view),
        }
    }
}

pub fn draw(sheet: &Spritesheet, buildings: &[Building], visible: TileRect, perspective: &Perspective, time: f32) {
    for &building in shown(buildings, visible) {
        let origin = origin(building);
        let model = model(building.kind);
        let view = perspective.center();

        let mut parts: Vec<Part> = model.blocks.iter().map(Part::Block).chain(model.stacks.iter().map(Part::Stack)).collect();
        parts.sort_by(|a, b| {
            let (da, db) = (a.distance(origin, view), b.distance(origin, view));
            db.total_cmp(&da).then(a.height().total_cmp(&b.height()))
        });
        for part in parts {
            match part {
                Part::Block(block) => draw_block(sheet, block, origin, perspective),
                Part::Stack(stack) => draw_stack(sheet, stack, origin, perspective),
            }
        }

        for (i, stack) in model.stacks.iter().enumerate().filter(|(_, s)| s.plume) {
            let phase = phase(building, i);
            draw_plume(sheet, stack, origin, perspective, time, phase);
        }
    }
}

// Keeps chimneys, and plants, from puffing in step.
fn phase(building: Building, stack: usize) -> f32 {
    (stack as f32 * 0.37 + building.x as f32 * 0.13 + building.y as f32 * 0.29).fract()
}

fn lifted(ground: Vec2, height: f32, perspective: &Perspective) -> Vec2 {
    ground + perspective.lean(ground, height)
}

fn draw_block(sheet: &Spritesheet, block: &Block, origin: Vec2, perspective: &Perspective) {
    let base = block.base.offset(origin);
    let (nw, se) = (base.point(), base.point() + base.size());
    let (ne, sw) = (vec2(se.x, nw.y), vec2(nw.x, se.y));
    let view = perspective.center();

    // Sides turned away from the viewer are hidden behind the roof.
    let sides = [
        (nw, ne, vec2(0.0, -1.0), block.wall_ns, NORTH),
        (ne, se, vec2(1.0, 0.0), block.wall_ew, EAST),
        (se, sw, vec2(0.0, 1.0), block.wall_ns, SOUTH),
        (sw, nw, vec2(-1.0, 0.0), block.wall_ew, WEST),
    ];
    for (from, to, outward, wall, light) in sides {
        if (view - (from + to) / 2.0).dot(outward) <= 0.0 {
            continue;
        }
        let corners = [
            lifted(from, block.height, perspective),
            lifted(to, block.height, perspective),
            to,
            from,
        ];
        draw_quad(sheet.texture(), Spritesheet::inset(wall), corners, Color::new(light, light, light, 1.0));
    }
    sheet.draw_sprite(block.roof, perspective.raise(base, block.height), WHITE);
}

// The body is stretched between the base and the top like a trunk, the cap sits on top.
fn draw_stack(sheet: &Spritesheet, stack: &Stack, origin: Vec2, perspective: &Perspective) {
    let ground = origin + stack.base;
    let top = lifted(ground, stack.height, perspective);
    // Round footprint, so a wide body reads as a cylinder rather than a flat ribbon.
    draw_circle(ground.x, ground.y, stack.body.w / 2.0, BASE);
    draw_strip(sheet.texture(), Spritesheet::inset(stack.body), ground, top, stack.body.w);
    let half = stack.cap.size() / 2.0;
    sheet.draw_sprite(stack.cap, Rect::new(top.x - half.x, top.y - half.y, stack.cap.w, stack.cap.h), WHITE);
}

fn draw_plume(sheet: &Spritesheet, stack: &Stack, origin: Vec2, perspective: &Perspective, time: f32, phase: f32) {
    let ground = origin + stack.base;
    let top = lifted(ground, stack.height, perspective);

    let flame = effects::flame(time, phase);
    let flicker = 0.75 + 0.25 * ((time * 23.0 + phase * 40.0).sin() * 0.5 + 0.5);
    draw_circle(top.x, top.y, 7.0, Color { a: GLOW.a * flicker, ..GLOW });
    let flame_at = (top - vec2(flame.w / 2.0, flame.h - 3.0)).round();
    sheet.draw_sprite(flame, Rect::new(flame_at.x, flame_at.y, flame.w, flame.h), WHITE);

    for puff in effects::smoke(time, phase) {
        let at = lifted(ground, stack.height + puff.height, perspective) + puff.drift;
        let half = puff.sprite.size() / 2.0;
        let corner = (at - half).round();
        sheet.draw_sprite(puff.sprite, Rect::new(corner.x, corner.y, puff.sprite.w, puff.sprite.h), puff.tint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KINDS: [BuildingKind; 3] = [BuildingKind::CoalPlant, BuildingKind::OilPlant, BuildingKind::GasPlant];

    fn building() -> Building {
        Building { kind: BuildingKind::CoalPlant, x: 100, y: 100 }
    }

    fn view(x0: usize, y0: usize, x1: usize, y1: usize) -> TileRect {
        TileRect { x0, y0, x1, y1 }
    }

    #[test]
    fn model_parts_stay_inside_the_footprint() {
      for kind in KINDS {
        let (w, h) = kind.footprint();
        let footprint = Rect::new(0.0, 0.0, w as f32 * TILE_SIZE, h as f32 * TILE_SIZE);
        let model = model(kind);
        assert_eq!(model.yard.size(), footprint.size());
        for block in model.blocks {
            assert!(footprint.contains(block.base.point()) && footprint.contains(block.base.point() + block.base.size()));
            assert_eq!(block.roof.size(), block.base.size());
            assert_eq!(block.wall_ns.size(), vec2(block.base.w, block.height));
            assert_eq!(block.wall_ew.size(), vec2(block.base.h, block.height));
        }
        for stack in model.stacks {
            let radius = stack.body.w / 2.0;
            assert!(footprint.contains(stack.base - radius) && footprint.contains(stack.base + radius));
        }
      }
    }

    #[test]
    fn blocks_do_not_overlap() {
        for kind in KINDS {
            let blocks = model(kind).blocks;
            for (i, a) in blocks.iter().enumerate() {
                assert!(blocks[i + 1..].iter().all(|b| !a.base.overlaps(&b.base)));
            }
        }
    }

    #[test]
    fn buildings_far_off_screen_are_skipped() {
        let all = [building()];
        assert_eq!(shown(&all, view(95, 95, 110, 110)).count(), 1);
        assert_eq!(shown(&all, view(90, 90, 98, 98)).count(), 1);
        assert_eq!(shown(&all, view(0, 0, 50, 50)).count(), 0);
        assert_eq!(shown(&all, view(160, 100, 200, 120)).count(), 0);
    }

    #[test]
    fn shadow_outline_is_the_box_swept_along_the_skew() {
        let outline = shadow(Rect::new(10.0, 20.0, 40.0, 30.0), vec2(6.0, 4.0));
        assert_eq!(outline[0], vec2(10.0, 20.0));
        assert_eq!(outline[3], vec2(56.0, 54.0));
    }

    #[test]
    fn farther_parts_are_drawn_first() {
        let near = Part::Stack(&Stack { base: vec2(10.0, 10.0), height: 30.0, body: Rect::default(), cap: Rect::default(), plume: false });
        let far = Part::Stack(&Stack { base: vec2(70.0, 70.0), height: 30.0, body: Rect::default(), cap: Rect::default(), plume: false });
        let view = vec2(0.0, 0.0);
        assert!(far.distance(Vec2::ZERO, view) > near.distance(Vec2::ZERO, view));
    }

    #[test]
    fn a_block_under_the_view_center_is_at_distance_zero() {
        let block = &model(BuildingKind::CoalPlant).blocks[0];
        let part = Part::Block(block);
        assert_eq!(part.distance(Vec2::ZERO, block.base.center()), 0.0);
    }

    #[test]
    fn plants_do_not_puff_in_step() {
        let a = phase(building(), 0);
        let b = phase(Building { x: 101, ..building() }, 0);
        assert_ne!(a, b);
        assert_ne!(phase(building(), 0), phase(building(), 1));
    }
}
