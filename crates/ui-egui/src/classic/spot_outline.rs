//! A brushed spot as Lightroom Classic shows it: the outline of the whole stroke, not a ring for every dab.
//!
//! A brushed spot is its dabs joined into one stroke of the brush's radius: the union of the capsules between
//! consecutive dabs. Its outline is drawn from the parts of each capsule's two sides and each dab's circle that lie
//! outside every other capsule. `panels/detail/retouch.rs` draws spots this way in the Classic style (a spot of one
//! dab is a circle either way).

use egui::{Color32, Painter, Pos2, Shape, Stroke, Vec2, pos2};

/// Pixels between the outline's samples.
const STEP: f32 = 2.5;

/// Most dabs an outline is drawn from: a longer stroke is thinned evenly first (its outline barely changes).
const MAX_DABS: usize = 160;

/// The outline of the stroke through `dabs` (screen points) with radius `r`.
pub(crate) fn draw(p: &Painter, dabs: &[Pos2], r: f32, stroke: Stroke) {
    for line in outline(dabs, r) {
        if line.len() >= 2 {
            p.add(Shape::line(line, stroke));
        }
    }
}

/// The stroke being painted: a light fill along it, and its outline.
pub(crate) fn draw_painting(p: &Painter, dabs: &[Pos2], r: f32) {
    let dabs = thin(dabs, r);
    if dabs.len() >= 2 {
        p.add(Shape::line(dabs.clone(), Stroke::new(2.0 * r, Color32::from_white_alpha(40))));
    }
    for d in [dabs.first(), dabs.last()].into_iter().flatten() {
        p.circle_filled(*d, r, Color32::from_white_alpha(40));
    }
    draw(p, &dabs, r, Stroke::new(1.0, Color32::WHITE));
}

/// Dabs closer than a quarter radius to the last one kept are dropped, and at most [`MAX_DABS`] are kept.
fn thin(dabs: &[Pos2], r: f32) -> Vec<Pos2> {
    let mut kept: Vec<Pos2> = Vec::with_capacity(dabs.len().min(MAX_DABS));
    for d in dabs {
        if d.x.is_finite() && d.y.is_finite() && kept.last().is_none_or(|k| k.distance(*d) >= r * 0.25) {
            kept.push(*d);
        }
    }
    if kept.len() > MAX_DABS {
        let n = kept.len();
        kept = (0..MAX_DABS).filter_map(|i| kept.get(i * (n - 1) / (MAX_DABS - 1)).copied()).collect();
    }
    kept
}

/// Distance from `q` to the segment `a`–`b`.
fn to_segment(q: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_sq();
    let t = if len2 > 0.0 { ((q - a).dot(ab) / len2).clamp(0.0, 1.0) } else { 0.0 };
    q.distance(a + ab * t)
}

/// The visible pieces of the outline, each a polyline.
fn outline(dabs: &[Pos2], r: f32) -> Vec<Vec<Pos2>> {
    let dabs = thin(dabs, r);
    if !(r.is_finite() && r > 0.5) || dabs.is_empty() {
        return Vec::new();
    }
    let segments: Vec<(Pos2, Pos2)> = if dabs.len() == 1 { vec![(dabs[0], dabs[0])] } else { dabs.windows(2).map(|w| (w[0], w[1])).collect() };
    // inside the stroke, clear of its edge
    let inside = |q: Pos2| {
        segments.iter().any(|&(a, b)| {
            let (lo, hi) = (a.min(b), a.max(b));
            q.x > lo.x - r && q.x < hi.x + r && q.y > lo.y - r && q.y < hi.y + r && to_segment(q, a, b) < r - 0.75
        })
    };
    let mut lines = Vec::new();
    let mut run: Vec<Pos2> = Vec::new();
    let emit = |q: Option<Pos2>, run: &mut Vec<Pos2>, lines: &mut Vec<Vec<Pos2>>| match q {
        Some(q) if !inside(q) => run.push(q),
        _ => {
            if run.len() >= 2 {
                lines.push(std::mem::take(run));
            } else {
                run.clear();
            }
        }
    };
    // each capsule's two sides
    for &(a, b) in &segments {
        let ab = b - a;
        let len = ab.length();
        if len < 1e-3 {
            continue;
        }
        let n = Vec2::new(-ab.y, ab.x) / len * r;
        let steps = (len / STEP).ceil().max(1.0) as usize;
        for side in [n, -n] {
            for i in 0..=steps {
                emit(Some(a + side + ab * (i as f32 / steps as f32)), &mut run, &mut lines);
            }
            emit(None, &mut run, &mut lines);
        }
    }
    // each dab's circle (the round ends and the outside of bends)
    let steps = ((std::f32::consts::TAU * r / STEP).ceil() as usize).clamp(16, 256);
    for d in &dabs {
        for i in 0..=steps {
            let a = std::f32::consts::TAU * i as f32 / steps as f32;
            emit(Some(pos2(d.x + r * a.cos(), d.y + r * a.sin())), &mut run, &mut lines);
        }
        emit(None, &mut run, &mut lines);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A straight stroke's outline is a stadium: every point of it lies on the stroke's edge, none inside, and it
    /// reaches both round ends and both sides.
    #[test]
    fn a_straight_stroke_is_outlined_as_one_shape() {
        let dabs: Vec<Pos2> = (0..20).map(|i| pos2(100.0 + 6.0 * i as f32, 50.0)).collect();
        let r = 10.0;
        let lines = outline(&dabs, r);
        let pts: Vec<Pos2> = lines.iter().flatten().copied().collect();
        assert!(!pts.is_empty());
        let (a, b) = (dabs[0], dabs[19]);
        for q in &pts {
            let d = to_segment(*q, a, b);
            assert!((d - r).abs() < 0.8, "{q:?} is {d} from the stroke's spine");
        }
        let xs = pts.iter().map(|q| q.x).fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), x| (lo.min(x), hi.max(x)));
        assert!(xs.0 < a.x - r + 1.0 && xs.1 > b.x + r - 1.0, "both ends: {xs:?}");
        assert!(pts.iter().any(|q| q.y < 41.0) && pts.iter().any(|q| q.y > 59.0), "both sides");
        // no ring per dab: a dab's ring would cross the stroke, nearer its spine than the radius (checked above), and
        // the outline is closed, with no gap longer than a couple of samples
        let length = 2.0 * a.distance(b) + std::f32::consts::TAU * r;
        assert!(pts.len() as f32 >= length / (2.0 * STEP), "{} samples for {length} px of outline", pts.len());
    }

    /// One dab is its circle; nothing, a bad radius or non-finite points draw nothing and never panic.
    #[test]
    fn a_single_dab_is_a_circle_and_bad_input_draws_nothing() {
        let lines = outline(&[pos2(0.0, 0.0)], 8.0);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].iter().all(|q| (q.distance(pos2(0.0, 0.0)) - 8.0).abs() < 1e-3));
        assert!(outline(&[], 8.0).is_empty());
        assert!(outline(&[pos2(0.0, 0.0)], f32::NAN).is_empty());
        assert!(outline(&[pos2(f32::NAN, 0.0)], 8.0).is_empty());
        let many: Vec<Pos2> = (0..5000).map(|i| pos2(i as f32, (i as f32 * 0.01).sin() * 30.0)).collect();
        assert!(thin(&many, 4.0).len() <= MAX_DABS);
        assert!(!outline(&many, 4.0).is_empty());
    }
}
