//! Human-looking cursor paths: a gently curved line with ease-in / ease-out,
//! instead of a teleport or a ruler-straight slide.

use crate::platform::Point;
use crate::rng::Rng;

/// Milliseconds between two points of a path.
pub const STEP_MS: u64 = 12;

fn ease(t: f64) -> f64 {
    // smoothstep: slow start, slow end
    t * t * (3.0 - 2.0 * t)
}

/// Points to visit when gliding from `from` to `to`. Always ends exactly on `to`.
pub fn glide(from: Point, to: Point, rng: &mut Rng) -> Vec<Point> {
    let dist = from.dist(to);
    if dist < 1.0 {
        return vec![to];
    }
    let steps = ((dist / 9.0).round() as usize).clamp(8, 45);
    let (fx, fy, tx, ty) = (from.x as f64, from.y as f64, to.x as f64, to.y as f64);
    // perpendicular unit vector, used to bow the path sideways a little
    let (nx, ny) = (-(ty - fy) / dist, (tx - fx) / dist);
    let bow = dist * rng.range_f64(-0.18, 0.18);
    let bow2 = dist * rng.range_f64(-0.10, 0.10);
    let c1 = (
        fx + (tx - fx) * 0.3 + nx * bow,
        fy + (ty - fy) * 0.3 + ny * bow,
    );
    let c2 = (
        fx + (tx - fx) * 0.7 + nx * bow2,
        fy + (ty - fy) * 0.7 + ny * bow2,
    );
    let mut out: Vec<Point> = Vec::with_capacity(steps);
    for i in 1..=steps {
        let t = ease(i as f64 / steps as f64);
        let u = 1.0 - t;
        let x = u * u * u * fx + 3.0 * u * u * t * c1.0 + 3.0 * u * t * t * c2.0 + t * t * t * tx;
        let y = u * u * u * fy + 3.0 * u * u * t * c1.1 + 3.0 * u * t * t * c2.1 + t * t * t * ty;
        let p = Point::new(x.round() as i32, y.round() as i32);
        if out.last() != Some(&p) {
            out.push(p);
        }
    }
    if out.last() != Some(&to) {
        out.push(to);
    }
    out
}
