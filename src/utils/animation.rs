#[derive(Clone, Copy)]
pub enum Step {
    FadeIn(f32),
    Hold(f32),
    FadeOut(f32),
    Wait(f32),
}

impl Step {
    fn duration(self) -> f32 {
        match self {
            Step::FadeIn(d) | Step::Hold(d) | Step::FadeOut(d) | Step::Wait(d) => d,
        }
    }

    fn alpha(self, progress: f32) -> f32 {
        match self {
            Step::FadeIn(_) => progress,
            Step::Hold(_) => 1.0,
            Step::FadeOut(_) => 1.0 - progress,
            Step::Wait(_) => 0.0,
        }
    }
}

pub struct Timeline {
    steps: Vec<Step>,
    index: usize,
    elapsed: f32,
}

impl Timeline {
    pub fn new(steps: Vec<Step>) -> Self {
        Self {
            steps,
            index: 0,
            elapsed: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
        while let Some(step) = self.steps.get(self.index) {
            let duration = step.duration();
            if self.elapsed < duration {
                break;
            }
            self.elapsed -= duration;
            self.index += 1;
        }
    }

    pub fn alpha(&self) -> f32 {
        match self.steps.get(self.index) {
            Some(step) => {
                let duration = step.duration();
                let progress = if duration > 0.0 {
                    self.elapsed / duration
                } else {
                    1.0
                };
                step.alpha(progress.clamp(0.0, 1.0))
            }
            None => 0.0,
        }
    }

    pub fn is_done(&self) -> bool {
        self.index >= self.steps.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fades_in_holds_and_fades_out() {
        let mut t = Timeline::new(vec![
            Step::FadeIn(1.0),
            Step::Hold(1.0),
            Step::FadeOut(1.0),
        ]);
        assert_eq!(t.alpha(), 0.0);
        t.update(0.5);
        assert_eq!(t.alpha(), 0.5);
        t.update(1.0);
        assert_eq!(t.alpha(), 1.0);
        t.update(1.0);
        assert_eq!(t.alpha(), 0.5);
        assert!(!t.is_done());
        t.update(0.5);
        assert!(t.is_done());
        assert_eq!(t.alpha(), 0.0);
    }

    #[test]
    fn large_delta_skips_steps() {
        let mut t = Timeline::new(vec![Step::Wait(0.5), Step::FadeIn(1.0)]);
        t.update(1.0);
        assert_eq!(t.alpha(), 0.5);
        t.update(10.0);
        assert!(t.is_done());
    }
}
