//! Lightroom Classic's slider rows: the name at the left, set against the track, the track, and the value at the
//! right, all on one line, with a pointed thumb and faint tick marks over the track.
//!
//! Every slider drawn inside [`rows`] takes this shape; `widgets::slider` asks [`on`] and keeps its behaviour (drag,
//! double-click to reset, typing or scrubbing the value, the wheel and arrow keys) and its widget ids as they are. The
//! Classic Develop column draws its panels inside [`rows`]; the Modern style never does, so its sliders are unchanged.

use egui::{Color32, Id, Painter, Pos2, Rect, Shape, Stroke, Ui, pos2, vec2};

/// Height of a one-line slider row.
pub(crate) const ROW_H: f32 = 25.0;

/// Where [`rows`] keeps whether it is drawing.
fn key() -> Id {
    Id::unique("classicSliderRows")
}

/// Draw `f` with Classic's one-line slider rows.
pub(crate) fn rows<R>(ui: &mut Ui, f: impl FnOnce(&mut Ui) -> R) -> R {
    let was = on(ui);
    ui.ctx().data_mut(|d| d.insert_temp(key(), true));
    let r = f(ui);
    ui.ctx().data_mut(|d| d.insert_temp(key(), was));
    r
}

/// Whether sliders are drawn as Classic's one-line rows here.
pub(crate) fn on(ui: &Ui) -> bool {
    ui.ctx().data(|d| d.get_temp::<bool>(key())).unwrap_or(false)
}

/// The name's, the track's and the value's rectangles in a row: the name right-aligned in the left third, the value
/// right-aligned at the right end.
pub(crate) fn layout(row: Rect) -> (Rect, Rect, Rect) {
    let (pad_l, pad_r) = (8.0, 14.0);
    let name_w = (row.width() * 0.3).clamp(60.0, 104.0);
    let value_w = 46.0;
    let cy = row.center().y;
    let name = Rect::from_min_max(pos2(row.left() + pad_l, cy - 9.0), pos2(row.left() + name_w, cy + 9.0));
    let value = Rect::from_min_max(pos2(row.right() - pad_r - value_w, cy - 9.0), pos2(row.right() - pad_r, cy + 9.0));
    let track_right = (value.left() - 10.0).max(name.right() + 20.0);
    let track = Rect::from_min_max(pos2(name.right() + 10.0, cy - 7.0), pos2(track_right, cy + 7.0));
    (name, track, value)
}

/// Faint marks over the track at every eighth, as Classic draws them.
pub(crate) fn ticks(p: &Painter, track: Rect, color: Color32) {
    let y = track.center().y - 5.0;
    for i in 0..=8 {
        let x = track.left() + track.width() * i as f32 / 8.0;
        p.line_segment([pos2(x, y - 2.5), pos2(x, y)], Stroke::new(1.0, color));
    }
}

/// The thumb: a drop pointing up at the value, centred on the track at `(x, y)`.
pub(crate) fn thumb(p: &Painter, x: f32, y: f32, fill: Color32, outline: Color32) {
    p.add(Shape::convex_polygon(outline_points(x, y), fill, Stroke::new(1.0, outline)));
}

/// The thumb's outline, clockwise from its point.
fn outline_points(x: f32, y: f32) -> Vec<Pos2> {
    let (r, c) = (5.0, pos2(x, y + 1.5));
    let mut pts = vec![pos2(x, y - 7.5)];
    // the round bottom, from the right shoulder (-25°) down and round to the left one (205°)
    let steps = 10;
    for i in 0..=steps {
        let a = (-25.0 + 230.0 * i as f32 / steps as f32).to_radians();
        pts.push(c + vec2(a.cos(), a.sin()) * r);
    }
    pts
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three parts of a row don't overlap and stay inside it, however narrow the column.
    #[test]
    fn a_row_holds_name_track_and_value_side_by_side() {
        for w in [180.0, 266.0, 420.0] {
            let row = Rect::from_min_size(pos2(0.0, 0.0), vec2(w, ROW_H));
            let (name, track, value) = layout(row);
            assert!(name.right() < track.left() && track.right() < value.left(), "{w}: {name:?} {track:?} {value:?}");
            assert!(row.contains_rect(name) && row.contains_rect(value) && track.width() >= 20.0, "{w}");
        }
    }

    /// The thumb is a convex drop: it points up, and its round part sits on the track.
    #[test]
    fn the_thumb_points_up() {
        let pts = outline_points(10.0, 20.0);
        let top = pts.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let bottom = pts.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(top, 12.5);
        assert!(bottom > 20.0 && pts.iter().all(|p| (p.x - 10.0).abs() <= 5.0 + 1e-4));
    }

    /// [`rows`] turns the style on for what it draws only, and restores what was there.
    #[test]
    fn rows_is_scoped() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            assert!(!on(ui));
            rows(ui, |ui| {
                assert!(on(ui));
                rows(ui, |ui| assert!(on(ui)));
                assert!(on(ui), "a nested scope keeps it on");
            });
            assert!(!on(ui));
        });
        out.textures_delta.clear();
    }
}
