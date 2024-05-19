use std::intrinsics::unlikely;

use macroquad::math::{vec2, Vec2, Vec2Swizzles};

pub const SEARCH_SEQUENCE: [(RectangleCorner, RectangleCorner); 4] = [
    (RectangleCorner::BottomRight, RectangleCorner::BottomLeft),
    (RectangleCorner::TopRight, RectangleCorner::TopLeft),
    (RectangleCorner::TopRight, RectangleCorner::BottomRight),
    (RectangleCorner::TopLeft, RectangleCorner::BottomLeft),
];

#[derive(Default)]
pub struct DrawingStrategy {
    pub body: bool,
    pub vision_distance: bool,
    pub target_line: bool,
}

#[derive(PartialEq, Eq, Hash)]
pub enum RectangleCorner {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl DrawingStrategy {
    /// Whether a circle with the center in `center` and the radius of `radius` intersects a
    /// vertical or horizontal line segment with the ends in `p1` and `p2`.
    pub fn circle_segment_intersection(center: Vec2, radius: f32, p1: Vec2, p2: Vec2) -> bool {
        // // Avoid `error: literal out of range for `f64``
        // if unlikely(radius < 1.0) {
        //     return false;
        // }
        //
        let (p, q, z1, z2, z);
        if p1.x == p2.x {
            (p, q) = (center.x, center.y);
            (z1, z2, z) = (p1.y, p2.y, p1.x);
        } else {
            (p, q) = (center.y, center.x);
            (z1, z2, z) = (p1.x, p2.x, p1.y);
        }

        match (z - p).powi(2) / radius.powi(2) {
            case if case == f32::INFINITY => return false,
            case if case > 1.0 => return false,
            _ => (),
        }

        let g = (z - p) / radius;
        let min = z1.min(z2) - q;
        let max = z1.max(z2) - q;
        let s = radius * (1.0 - g.powi(2)).sqrt();

        (min < -s && -s < max) || (min < s && s < max)
    }

    /// Whether the segment line from `p1` to `p2` contains `p` if it's known that it is on a segment line that
    /// includes the one from `p1` to `p2`.
    pub fn segment_contains_point(p1: Vec2, p2: Vec2, p: Vec2) -> bool {
        (p1.x.min(p2.x) - 1.0 < p.x && p.x < p1.x.max(p2.x) + 1.0)
            && (p1.y.min(p2.y) - 1.0 < p.y && p.y < p1.y.max(p2.y) + 1.0)
    }

    pub fn line_coeffs(p1: Vec2, p2: Vec2) -> (f32, f32, f32) {
        ((p1.y - p2.y), (p2.x - p1.x), (p2.x * p1.y - p1.x * p2.y))
    }

    /// Whether `p1`-`p2` and `p3`-`p4` intersect.
    pub fn segments_intersect(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> bool {
        let (a2, b2, c2) = DrawingStrategy::line_coeffs(p1, p2);
        let (a1, b1, c1) = DrawingStrategy::line_coeffs(p3, p4);

        let d = a1 * b2 - b1 * a2;
        let dx = c1 * b2 - b1 * c2;
        let dy = a1 * c2 - c1 * a2;

        if dx.powi(2).max(dy.powi(2)) / dy.powi(2) == f32::INFINITY {
            return false;
        }

        if d == 0.0 {
            return false;
        }
        let p = vec2(dx / d, dy / d);
        DrawingStrategy::segment_contains_point(p1, p2, p)
            && DrawingStrategy::segment_contains_point(p3, p4, p)
    }
}
