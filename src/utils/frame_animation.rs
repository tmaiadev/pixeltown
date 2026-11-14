use super::animation::Animation;

pub struct FrameAnimation {
    frames: usize,
    clock: Animation,
}

impl FrameAnimation {
    pub fn new(frames: usize, fps: f32) -> Self {
        Self {
            frames,
            clock: Animation::new(frames as f32 / fps).looping().autostart(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.clock.update(dt);
    }

    pub fn frame(&self) -> usize {
        ((self.clock.progress() * self.frames as f32) as usize).min(self.frames - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_on_the_first_frame() {
        assert_eq!(FrameAnimation::new(4, 4.0).frame(), 0);
    }

    #[test]
    fn advances_one_frame_per_interval() {
        let mut a = FrameAnimation::new(4, 4.0);
        a.update(0.26);
        assert_eq!(a.frame(), 1);
        a.update(0.25);
        assert_eq!(a.frame(), 2);
    }

    #[test]
    fn wraps_back_to_the_first_frame() {
        let mut a = FrameAnimation::new(4, 4.0);
        a.update(1.1);
        assert_eq!(a.frame(), 0);
    }

    #[test]
    fn never_exceeds_the_last_frame() {
        let mut a = FrameAnimation::new(3, 3.0);
        a.update(0.999);
        assert_eq!(a.frame(), 2);
    }
}
