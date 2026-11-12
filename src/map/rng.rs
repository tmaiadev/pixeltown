pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn range(&mut self, min: u32, max: u32) -> u32 {
        min + (self.next_u64() % (max - min + 1) as u64) as u32
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }
}

fn lattice(x: i32, y: i32, seed: u64) -> f32 {
    let mut h = seed
        ^ (x as i64 as u64).wrapping_mul(0x9E37_79B1_85EB_CA87)
        ^ (y as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h = (h ^ (h >> 33)).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h = (h ^ (h >> 33)).wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^= h >> 33;
    (h >> 40) as f32 / (1u64 << 24) as f32
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (smooth(x - x0), smooth(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let top = lattice(xi, yi, seed) * (1.0 - fx) + lattice(xi + 1, yi, seed) * fx;
    let bottom = lattice(xi, yi + 1, seed) * (1.0 - fx) + lattice(xi + 1, yi + 1, seed) * fx;
    top * (1.0 - fy) + bottom * fy
}

pub fn fbm(x: f32, y: f32, seed: u64, octaves: u32) -> f32 {
    let (mut sum, mut amplitude, mut frequency, mut total) = (0.0, 1.0, 1.0, 0.0);
    for octave in 0..octaves {
        sum += value_noise(x * frequency, y * frequency, seed.wrapping_add(octave as u64)) * amplitude;
        total += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    sum / total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_deterministic_per_seed() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        let mut c = Rng::new(43);
        assert_eq!(a.next_u64(), b.next_u64());
        assert_ne!(a.next_u64(), c.next_u64());
    }

    #[test]
    fn rng_range_is_inclusive_and_bounded() {
        let mut rng = Rng::new(1);
        for _ in 0..1000 {
            let n = rng.range(1, 3);
            assert!((1..=3).contains(&n));
        }
    }

    #[test]
    fn next_f32_stays_in_unit_interval() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            let f = rng.next_f32();
            assert!((0.0..1.0).contains(&f));
        }
    }

    #[test]
    fn fbm_is_deterministic_and_normalized() {
        for i in 0..200 {
            let (x, y) = (i as f32 * 0.37, i as f32 * 0.91);
            let v = fbm(x, y, 5, 4);
            assert_eq!(v, fbm(x, y, 5, 4));
            assert!((0.0..=1.0).contains(&v));
        }
    }
}
