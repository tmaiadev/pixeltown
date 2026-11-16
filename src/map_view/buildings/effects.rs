use macroquad::prelude::*;

// Grey discs that grow with age, in the buildings sheet.
pub const SMOKE: [Rect; 4] = [
    Rect { x: 220.0, y: 12.0, w: 4.0, h: 4.0 },
    Rect { x: 226.0, y: 12.0, w: 6.0, h: 6.0 },
    Rect { x: 234.0, y: 12.0, w: 8.0, h: 8.0 },
    Rect { x: 244.0, y: 12.0, w: 10.0, h: 10.0 },
];
pub const FLAME_SIZE: Vec2 = Vec2::new(8.0, 12.0);
const FLAME_FRAMES: usize = 4;
const FLAME_FPS: f32 = 9.0;

const PUFFS: usize = 7;
const LIFETIME: f32 = 3.5;
// Pixels per second a puff climbs, and how far the wind pushes it sideways.
const RISE: f32 = 9.0;
const WIND: Vec2 = Vec2::new(6.0, -2.0);
const JITTER: f32 = 1.5;
const DARKEST: f32 = 0.25;
const LIGHTEST: f32 = 0.85;
const OPACITY: f32 = 1.0;

pub fn flame(time: f32, phase: f32) -> Rect {
    let frame = ((time * FLAME_FPS + phase * FLAME_FRAMES as f32) as usize) % FLAME_FRAMES;
    Rect::new(208.0 + frame as f32 * 10.0, 40.0, FLAME_SIZE.x, FLAME_SIZE.y)
}

pub struct Puff {
    pub sprite: Rect,
    // Height above the top of the chimney.
    pub height: f32,
    // Sideways drift on the ground plane, away from the chimney.
    pub drift: Vec2,
    pub tint: Color,
}

// Puffs leave the chimney evenly spaced; each is a pure function of the time, so nothing is stored.
// Returned oldest first, so drawing in order puts the young ones on top.
pub fn smoke(time: f32, phase: f32) -> Vec<Puff> {
    let mut puffs: Vec<(f32, Puff)> = (0..PUFFS)
        .map(|n| {
            let cycles = time / LIFETIME + phase + n as f32 / PUFFS as f32;
            let age = cycles.fract();
            let seconds = age * LIFETIME;
            let sway = vec2(noise(cycles.floor() as i64, n, 0), noise(cycles.floor() as i64, n, 1)) * JITTER;
            let stage = ((age * SMOKE.len() as f32) as usize).min(SMOKE.len() - 1);
            let shade = DARKEST + (LIGHTEST - DARKEST) * age;
            let fade_in = (age * 10.0).min(1.0);
            let alpha = OPACITY * fade_in * (1.0 - age).powf(0.8);
            let puff = Puff {
                sprite: SMOKE[stage],
                height: seconds * RISE,
                drift: WIND * seconds + sway,
                tint: Color::new(shade, shade, shade, alpha),
            };
            (age, puff)
        })
        .collect();
    puffs.sort_by(|a, b| b.0.total_cmp(&a.0));
    puffs.into_iter().map(|(_, puff)| puff).collect()
}

// Deterministic value in -1..1 for a puff's n-th cycle.
fn noise(cycle: i64, n: usize, axis: u64) -> f32 {
    let mut h = (cycle as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (n as u64 + 1).wrapping_mul(0xBF58_476D_1CE4_E5B9) ^ axis.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 32;
    (h % 2001) as f32 / 1000.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_puff_stays_within_its_lifetime_height() {
        for step in 0..200 {
            for puff in smoke(step as f32 * 0.07, 0.3) {
                assert!(puff.height >= 0.0 && puff.height <= LIFETIME * RISE);
                assert!(puff.tint.a >= 0.0 && puff.tint.a <= OPACITY);
            }
        }
    }

    #[test]
    fn smoke_grows_as_it_ages() {
        let puffs = smoke(1.0, 0.0);
        assert!(puffs.windows(2).all(|w| w[0].sprite.w >= w[1].sprite.w));
        assert!(puffs.first().unwrap().sprite.w > puffs.last().unwrap().sprite.w);
    }

    #[test]
    fn smoke_is_darker_when_young() {
        let puffs = smoke(1.0, 0.0);
        assert!(puffs.first().unwrap().tint.r > puffs.last().unwrap().tint.r);
    }

    #[test]
    fn smoke_repeats_every_lifetime_up_to_the_jitter() {
        let a = smoke(0.4, 0.0);
        let b = smoke(0.4 + LIFETIME / PUFFS as f32, 0.0);
        assert_eq!(a.len(), b.len());
        for (p, q) in a.iter().zip(&b) {
            assert!((p.height - q.height).abs() < 1e-3);
        }
    }

    #[test]
    fn phase_offsets_the_plume() {
        let a = smoke(1.0, 0.0);
        let b = smoke(1.0, 0.5);
        assert!(a.iter().zip(&b).any(|(p, q)| (p.height - q.height).abs() > 0.5));
    }

    #[test]
    fn flame_cycles_through_its_frames() {
        let frames: Vec<f32> = (0..8).map(|i| flame(i as f32 / FLAME_FPS, 0.0).x).collect();
        assert_eq!(frames[0], frames[FLAME_FRAMES]);
        assert_ne!(frames[0], frames[1]);
    }

    #[test]
    fn noise_stays_in_range_and_varies() {
        let values: Vec<f32> = (0..50).map(|c| noise(c, 2, 0)).collect();
        assert!(values.iter().all(|v| (-1.0..=1.0).contains(v)));
        assert!(values.windows(2).any(|w| w[0] != w[1]));
    }
}
