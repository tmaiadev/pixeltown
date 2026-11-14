use macroquad::prelude::*;

use super::camera::{TileRect, TILE_SIZE};
use crate::map::MAP_SIZE;
use crate::set_internal_camera;
use crate::utils::perspective::Perspective;

const CHUNK_TILES: usize = 32;
const CHUNKS: usize = MAP_SIZE / CHUNK_TILES;
const CHUNK_PIXELS: f32 = CHUNK_TILES as f32 * TILE_SIZE;

// A map-wide layer baked into textures per chunk, with one texture per animation frame.
pub struct ChunkCache {
    targets: Vec<Vec<RenderTarget>>,
}

impl ChunkCache {
    pub fn new() -> Self {
        Self {
            targets: (0..CHUNKS * CHUNKS).map(|_| Vec::new()).collect(),
        }
    }

    // `frames` says how many frames a chunk needs; `bake` draws a chunk's frame.
    pub fn render(&mut self, frames: impl Fn(usize) -> usize, bake: impl Fn(usize, usize)) {
        for (chunk, targets) in self.targets.iter_mut().enumerate() {
            let count = frames(chunk).max(1);
            targets.truncate(count);
            while targets.len() < count {
                let target = render_target(CHUNK_PIXELS as u32, CHUNK_PIXELS as u32);
                target.texture.set_filter(FilterMode::Nearest);
                targets.push(target);
            }
            let origin = chunk_origin(chunk);
            for (frame, target) in targets.iter().enumerate() {
                set_camera(&Camera2D {
                    zoom: Vec2::splat(2.0 / CHUNK_PIXELS),
                    target: origin + CHUNK_PIXELS / 2.0,
                    render_target: Some(target.clone()),
                    ..Default::default()
                });
                clear_background(BLANK);
                bake(chunk, frame);
            }
        }
        // clear_background is immediate, so a later clear must not land on the last chunk.
        set_internal_camera();
    }

    // Chunks baked with a single frame show it at every frame.
    fn texture(&self, chunk: usize, frame: usize) -> &Texture2D {
        let targets = &self.targets[chunk];
        &targets[frame % targets.len()].texture
    }

    pub fn draw(&self, visible: TileRect, frame: usize) {
        for chunk in visible_chunks(visible) {
            let origin = chunk_origin(chunk);
            draw_texture(self.texture(chunk, frame), origin.x, origin.y, WHITE);
        }
    }

    pub fn draw_raised(&self, visible: TileRect, perspective: &Perspective, height: f32) {
        for chunk in visible_chunks(visible) {
            let origin = chunk_origin(chunk);
            let area = perspective.raise(Rect::new(origin.x, origin.y, CHUNK_PIXELS, CHUNK_PIXELS), height);
            draw_texture_ex(
                self.texture(chunk, 0),
                area.x,
                area.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(area.size()),
                    ..Default::default()
                },
            );
        }
    }
}

fn visible_chunks(TileRect { x0, y0, x1, y1 }: TileRect) -> impl Iterator<Item = usize> {
    (y0 / CHUNK_TILES..y1.div_ceil(CHUNK_TILES))
        .flat_map(move |cy| (x0 / CHUNK_TILES..x1.div_ceil(CHUNK_TILES)).map(move |cx| cy * CHUNKS + cx))
}

fn chunk_origin(chunk: usize) -> Vec2 {
    vec2((chunk % CHUNKS) as f32, (chunk / CHUNKS) as f32) * CHUNK_PIXELS
}

pub fn chunk_tiles(chunk: usize) -> impl Iterator<Item = (f32, f32, usize)> + Clone {
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
