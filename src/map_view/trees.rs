use macroquad::prelude::*;

use super::camera::TILE_SIZE;
use crate::map::{autotile::{self, TreeCell}, MAP_SIZE};
use crate::utils::perspective::{draw_strip, Perspective};
use crate::utils::spritesheet::Spritesheet;

pub const TREE_HEIGHT: f32 = 6.0;
const TRUNK_WIDTH: f32 = 2.0;
// Crowns lean out of their own tile, so trees just off screen can still show up.
const MARGIN: usize = 1;

pub type Visible = (usize, usize, usize, usize);

pub fn draw_shadows(nature: &Spritesheet, trees: &[TreeCell], visible: Visible) {
    let (col, row) = autotile::tree_shadow();
    let half = nature.tile_size() / 2.0;
    for tree in shown(trees, |t| (t.x, t.y), visible) {
        let ground = ground(tree.x, tree.y);
        nature.draw_tile(col, row, ground.x - half, ground.y - half);
    }
}

pub fn draw_trunks(nature: &Spritesheet, trunks: &[(u16, u16)], visible: Visible, perspective: &Perspective) {
    for &(x, y) in shown(trunks, |&t| t, visible) {
        draw_trunk(nature, ground(x, y), perspective);
    }
}

pub fn draw_trees(nature: &Spritesheet, trees: &[TreeCell], visible: Visible, perspective: &Perspective) {
    let half = nature.tile_size() / 2.0;
    for tree in shown(trees, |t| (t.x, t.y), visible) {
        let top = draw_trunk(nature, ground(tree.x, tree.y), perspective);
        let (col, row) = tree.crown;
        nature.draw_tile(col as usize, row as usize, top.x - half, top.y - half);
    }
}

fn draw_trunk(nature: &Spritesheet, ground: Vec2, perspective: &Perspective) -> Vec2 {
    let (col, row) = autotile::tree_trunk();
    let source = Rect::new(col as f32 * TILE_SIZE, row as f32 * TILE_SIZE, TRUNK_WIDTH, TILE_SIZE);
    let top = ground + perspective.lean(ground, TREE_HEIGHT);
    draw_strip(nature.texture(), source, ground, top, TRUNK_WIDTH);
    top
}

fn ground(x: u16, y: u16) -> Vec2 {
    (vec2(x as f32, y as f32) + 0.5) * TILE_SIZE
}

// `items` must be in row-major order.
fn shown<T>(items: &[T], position: impl Fn(&T) -> (u16, u16) + Copy, visible: Visible) -> impl Iterator<Item = &T> {
    let (x0, y0) = (visible.0.saturating_sub(MARGIN), visible.1.saturating_sub(MARGIN));
    let (x1, y1) = ((visible.2 + MARGIN).min(MAP_SIZE), (visible.3 + MARGIN).min(MAP_SIZE));
    let start = items.partition_point(|t| (position(t).1 as usize) < y0);
    items[start..]
        .iter()
        .take_while(move |t| (position(t).1 as usize) < y1)
        .filter(move |t| (x0..x1).contains(&(position(t).0 as usize)))
}
