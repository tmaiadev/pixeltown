use macroquad::prelude::*;

use super::camera::TILE_SIZE;
use crate::map::{autotile::{self, Tile}, MAP_SIZE};
use crate::set_internal_camera;
use crate::utils::spritesheet::Spritesheet;

const CHUNK_TILES: usize = 32;
const CHUNKS: usize = MAP_SIZE / CHUNK_TILES;
const CHUNK_PIXELS: f32 = CHUNK_TILES as f32 * TILE_SIZE;

pub type Features = [Option<(u8, u8)>];

// Static tiles are baked into one texture per chunk; animated tiles are left transparent.
pub struct ChunkCache {
    targets: Vec<RenderTarget>,
}

impl ChunkCache {
    pub fn new() -> Self {
        let targets = (0..CHUNKS * CHUNKS)
            .map(|_| {
                let target = render_target(CHUNK_PIXELS as u32, CHUNK_PIXELS as u32);
                target.texture.set_filter(FilterMode::Nearest);
                target
            })
            .collect();
        Self { targets }
    }

    pub fn render(&self, sheet: &Spritesheet, nature: &Spritesheet, cells: &[Tile], features: &Features) {
        for (chunk, target) in self.targets.iter().enumerate() {
            let origin = chunk_origin(chunk);
            set_camera(&Camera2D {
                zoom: Vec2::splat(2.0 / CHUNK_PIXELS),
                target: origin + CHUNK_PIXELS / 2.0,
                render_target: Some(target.clone()),
                ..Default::default()
            });
            clear_background(BLANK);

            let tiles = chunk_tiles(chunk).filter(|&(_, _, i)| !cells[i].is_animated());
            for (x, y, i) in tiles.clone() {
                let (col, row) = autotile::animated_cell(cells[i], 0, 0);
                sheet.draw_tile(col, row, x, y);
            }
            for (x, y, i) in tiles {
                if let Some((col, row)) = features[i] {
                    nature.draw_tile(col as usize, row as usize, x, y);
                }
            }
        }
        // clear_background is immediate, so a later clear must not land on the last chunk.
        set_internal_camera();
    }

    pub fn draw(&self, (x0, y0, x1, y1): (usize, usize, usize, usize)) {
        for cy in y0 / CHUNK_TILES..y1.div_ceil(CHUNK_TILES) {
            for cx in x0 / CHUNK_TILES..x1.div_ceil(CHUNK_TILES) {
                let chunk = cy * CHUNKS + cx;
                let origin = chunk_origin(chunk);
                draw_texture(&self.targets[chunk].texture, origin.x, origin.y, WHITE);
            }
        }
    }
}

fn chunk_origin(chunk: usize) -> Vec2 {
    vec2((chunk % CHUNKS) as f32, (chunk / CHUNKS) as f32) * CHUNK_PIXELS
}

fn chunk_tiles(chunk: usize) -> impl Iterator<Item = (f32, f32, usize)> + Clone {
    let (cx, cy) = (chunk % CHUNKS * CHUNK_TILES, chunk / CHUNKS * CHUNK_TILES);
    (cy..cy + CHUNK_TILES).flat_map(move |y| {
        (cx..cx + CHUNK_TILES).map(move |x| (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, y * MAP_SIZE + x))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_tiles_cover_the_map_exactly_once() {
        let mut seen = vec![0; MAP_SIZE * MAP_SIZE];
        for chunk in 0..CHUNKS * CHUNKS {
            for (_, _, i) in chunk_tiles(chunk) {
                seen[i] += 1;
            }
        }
        assert!(seen.iter().all(|&n| n == 1));
    }

    #[test]
    fn chunk_tiles_lie_inside_their_chunk() {
        let chunk = CHUNKS + 2;
        let origin = chunk_origin(chunk);
        for (x, y, _) in chunk_tiles(chunk) {
            assert!(x >= origin.x && x < origin.x + CHUNK_PIXELS);
            assert!(y >= origin.y && y < origin.y + CHUNK_PIXELS);
        }
    }
}
