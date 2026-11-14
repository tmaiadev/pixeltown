use macroquad::prelude::*;

use crate::screen::to_internal;

pub fn tap_position() -> Option<Vec2> {
    touches()
        .into_iter()
        .find(|t| t.phase == TouchPhase::Started)
        .map(|t| to_internal(t.position))
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gesture {
    pub drag: Vec2,
    pub zoom: f32,
}

#[derive(Default)]
pub struct Gestures {
    previous: Option<(Vec2, f32)>,
}

impl Gestures {
    pub fn poll(&mut self) -> Option<Gesture> {
        let points: Vec<Vec2> = touches().into_iter().map(|t| t.position).collect();
        let [a, b] = points[..] else {
            self.previous = None;
            return None;
        };

        let current = ((a + b) * 0.5, a.distance(b));
        let gesture = self.previous.map(|previous| {
            let gesture = two_finger_gesture(previous, current);
            Gesture {
                drag: to_internal(gesture.drag),
                ..gesture
            }
        });
        self.previous = Some(current);
        gesture
    }
}

fn two_finger_gesture(previous: (Vec2, f32), current: (Vec2, f32)) -> Gesture {
    let zoom = if previous.1 > 0.0 && current.1 > 0.0 {
        (current.1 / previous.1).ln()
    } else {
        0.0
    };
    Gesture {
        drag: current.0 - previous.0,
        zoom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_both_fingers_together_drags_without_zooming() {
        let gesture = two_finger_gesture((vec2(100.0, 100.0), 50.0), (vec2(110.0, 90.0), 50.0));
        assert_eq!(gesture.drag, vec2(10.0, -10.0));
        assert_eq!(gesture.zoom, 0.0);
    }

    #[test]
    fn spreading_fingers_zooms_in_and_pinching_zooms_out() {
        let spread = two_finger_gesture((vec2(0.0, 0.0), 50.0), (vec2(0.0, 0.0), 100.0));
        let pinch = two_finger_gesture((vec2(0.0, 0.0), 100.0), (vec2(0.0, 0.0), 50.0));
        assert!(spread.zoom > 0.0);
        assert!((spread.zoom + pinch.zoom).abs() < 1e-6);
    }

    #[test]
    fn degenerate_distance_does_not_zoom() {
        let gesture = two_finger_gesture((vec2(0.0, 0.0), 0.0), (vec2(0.0, 0.0), 40.0));
        assert_eq!(gesture.zoom, 0.0);
    }
}
