pub struct Animation {
    duration: f32,
    elapsed: f32,
    started: bool,
    ended: bool,
    looping: bool,
}

impl Animation {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            started: false,
            ended: false,
            looping: false,
        }
    }

    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    pub fn autostart(mut self) -> Self {
        self.started = true;
        self
    }

    pub fn start(&mut self) {
        self.started = true;
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.ended = false;
    }

    pub fn update(&mut self, dt: f32) {
        if !self.started || self.ended {
            return;
        }

        self.elapsed += dt;

        if self.elapsed >= self.duration {
            if self.looping && self.duration > 0.0 {
                self.elapsed %= self.duration;
            } else {
                self.elapsed = self.duration;
                self.ended = true;
            }
        }
    }

    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    #[allow(dead_code)]
    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn has_started(&self) -> bool {
        self.started
    }

    pub fn has_ended(&self) -> bool {
        self.ended
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_advances_with_updates() {
        let mut a = Animation::new(2.0).autostart();
        assert_eq!(a.progress(), 0.0);
        a.update(0.5);
        assert_eq!(a.progress(), 0.25);
        a.update(1.0);
        assert_eq!(a.progress(), 0.75);
        assert!(!a.has_ended());
    }

    #[test]
    fn does_nothing_until_started() {
        let mut a = Animation::new(1.0);
        a.update(0.5);
        assert!(!a.has_started());
        assert_eq!(a.elapsed(), 0.0);
        a.start();
        a.update(0.5);
        assert_eq!(a.progress(), 0.5);
    }

    #[test]
    fn clamps_and_stops_at_the_end() {
        let mut a = Animation::new(1.0).autostart();
        a.update(5.0);
        assert!(a.has_ended());
        assert_eq!(a.elapsed(), 1.0);
        assert_eq!(a.progress(), 1.0);
        a.update(1.0);
        assert_eq!(a.elapsed(), 1.0);
    }

    #[test]
    fn loop_wraps_elapsed() {
        let mut a = Animation::new(1.0).looping().autostart();
        a.update(1.25);
        assert!(!a.has_ended());
        assert_eq!(a.elapsed(), 0.25);
        a.update(2.5);
        assert_eq!(a.elapsed(), 0.75);
    }

    #[test]
    fn zero_duration_is_complete() {
        let mut a = Animation::new(0.0).autostart();
        assert_eq!(a.progress(), 1.0);
        a.update(0.1);
        assert!(a.has_ended());
    }

    #[test]
    fn reset_restarts_a_finished_animation() {
        let mut a = Animation::new(1.0).autostart();
        a.update(2.0);
        a.reset();
        assert!(!a.has_ended());
        assert_eq!(a.progress(), 0.0);
        a.update(0.5);
        assert_eq!(a.progress(), 0.5);
    }
}
