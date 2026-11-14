use macroquad::prelude::*;

use super::camera::TILE_SIZE;
use crate::map::{autotile::{self, TreeCell}, MAP_SIZE};
use crate::utils::perspective::{draw_strip, Perspective};
use crate::utils::spritesheet::Spritesheet;

const TREE_HEIGHT: f32 = 6.0;
const TRUNK_WIDTH: f32 = 2.0;
// Crowns lean out of their own tile, so trees just off screen can still show up.
const MARGIN: usize = 1;

pub fn draw(nature: &Spritesheet, trees: &[TreeCell], visible: (usize, usize, usize, usize), perspective: &Perspective) {
    let (x0, y0) = (visible.0.saturating_sub(MARGIN), visible.1.saturating_sub(MARGIN));
    let (x1, y1) = ((visible.2 + MARGIN).min(MAP_SIZE), (visible.3 + MARGIN).min(MAP_SIZE));
    let (trunk_col, trunk_row) = autotile::tree_trunk();
    let trunk = Rect::new(trunk_col as f32 * TILE_SIZE, trunk_row as f32 * TILE_SIZE, TRUNK_WIDTH, TILE_SIZE);
    let (shadow_col, shadow_row) = autotile::tree_shadow();
    let half = nature.tile_size() / 2.0;

    let start = trees.partition_point(|t| (t.y as usize) < y0);
    let shown = trees[start..]
        .iter()
        .take_while(|t| (t.y as usize) < y1)
        .filter(|t| (x0..x1).contains(&(t.x as usize)));
    let ground = |tree: &TreeCell| (vec2(tree.x as f32, tree.y as f32) + 0.5) * TILE_SIZE;

    // Shadows go down first so no shadow lands on a neighbor's leaning crown.
    for tree in shown.clone() {
        let ground = ground(tree);
        nature.draw_tile(shadow_col, shadow_row, ground.x - half, ground.y - half);
    }
    for tree in shown {
        let ground = ground(tree);
        let top = ground + perspective.lean(ground, TREE_HEIGHT);
        draw_strip(nature.texture(), trunk, ground, top, TRUNK_WIDTH);
        let (col, row) = tree.crown;
        nature.draw_tile(col as usize, row as usize, top.x - half, top.y - half);
    }
}
