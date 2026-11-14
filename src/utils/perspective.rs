use macroquad::prelude::*;

pub struct Perspective {
    center: Vec2,
    altitude: f32,
}

impl Perspective {
    // At the top and bottom edges of the view, things lean by their full height.
    pub fn new(view_center: Vec2, view_size: Vec2) -> Self {
        Self {
            center: view_center,
            altitude: view_size.y / 2.0,
        }
    }

    // Offset of a point `height` pixels above `ground`, pushed away from the view center.
    pub fn lean(&self, ground: Vec2, height: f32) -> Vec2 {
        ((ground - self.center) * height / self.altitude).round()
    }

    // Lifts a flat area to `height`; areas sharing an edge still share it once raised.
    pub fn raise(&self, area: Rect, height: f32) -> Rect {
        let min = area.point() + self.lean(area.point(), height);
        let corner = area.point() + area.size();
        let max = corner + self.lean(corner, height);
        Rect::new(min.x, min.y, max.x - min.x, max.y - min.y)
    }
}

// Stretches `source` into a band `width` pixels wide running from `from` to `to`.
pub fn draw_strip(texture: &Texture2D, source: Rect, from: Vec2, to: Vec2, width: f32) {
    let span = to - from;
    let length = span.length();
    if length == 0.0 {
        return;
    }
    // Unrotated, the band hangs down from `from`; rotate it so it points at `to`.
    let rotation = span.y.atan2(span.x) - std::f32::consts::FRAC_PI_2;
    draw_texture_ex(
        texture,
        from.x - width / 2.0,
        from.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width, length)),
            source: Some(source),
            rotation,
            pivot: Some(from),
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const CENTER: Vec2 = Vec2::new(500.0, 500.0);
    const VIEW: Vec2 = Vec2::new(800.0, 480.0);

    fn perspective() -> Perspective {
        Perspective::new(CENTER, VIEW)
    }

    #[test]
    fn nothing_leans_at_the_view_center() {
        assert_eq!(perspective().lean(CENTER, 10.0), Vec2::ZERO);
    }

    #[test]
    fn leans_away_from_the_view_center() {
        let p = perspective();
        assert!(p.lean(CENTER + vec2(300.0, 0.0), 10.0).x > 0.0);
        assert!(p.lean(CENTER - vec2(300.0, 0.0), 10.0).x < 0.0);
        assert!(p.lean(CENTER + vec2(0.0, 200.0), 10.0).y > 0.0);
        assert!(p.lean(CENTER - vec2(0.0, 200.0), 10.0).y < 0.0);
    }

    #[test]
    fn lean_is_symmetric() {
        let p = perspective();
        let offset = vec2(240.0, -180.0);
        assert_eq!(p.lean(CENTER + offset, 12.0), -p.lean(CENTER - offset, 12.0));
    }

    #[test]
    fn lean_grows_with_height_and_distance() {
        let p = perspective();
        let near = p.lean(CENTER + vec2(200.0, 0.0), 10.0).x;
        let far = p.lean(CENTER + vec2(400.0, 0.0), 10.0).x;
        let tall = p.lean(CENTER + vec2(200.0, 0.0), 20.0).x;
        assert!(far > near);
        assert!(tall > near);
    }

    #[test]
    fn lean_snaps_to_whole_pixels() {
        let lean = perspective().lean(CENTER + vec2(123.0, 77.0), 7.0);
        assert_eq!(lean, lean.round());
    }

    #[test]
    fn raising_grows_an_area_around_the_view_center() {
        let area = Rect::new(CENTER.x - 100.0, CENTER.y - 50.0, 200.0, 100.0);
        let raised = perspective().raise(area, 12.0);
        assert_eq!(raised.center(), area.center());
        assert!(raised.w > area.w && raised.h > area.h);
    }

    #[test]
    fn raised_neighbors_stay_seamless() {
        let p = perspective();
        let left = p.raise(Rect::new(613.0, 280.0, 512.0, 512.0), 6.0);
        let right = p.raise(Rect::new(1125.0, 280.0, 512.0, 512.0), 6.0);
        assert_eq!(left.right(), right.left());
    }

    #[test]
    fn both_axes_share_the_same_scale() {
        let p = perspective();
        let lean = p.lean(CENTER + vec2(240.0, 240.0), 12.0);
        assert_eq!(lean.x, lean.y);
    }
}
