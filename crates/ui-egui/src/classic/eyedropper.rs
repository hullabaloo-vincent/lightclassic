//! Lightroom Classic's White Balance Selector: an eyedropper on the WB row's button, and the pointer over the photo
//! while it picks, the dropper's tip on the point it will sample.
//!
//! Drawn in code (original work): a rounded bulb at the top right, a collar across, and a glass tube narrowing to the
//! tip at the bottom left.

use egui::{Color32, CursorIcon, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, vec2};

use crate::LightcraftApp;
use crate::theme::Tokens;
use crate::widgets::register;

/// The dropper's parts in a 20 × 20 box, tip at `TIP`: the tube, the collar and the bulb, each a convex outline.
const TIP: (f32, f32) = (2.0, 18.0);

/// Side of the pointer's dropper, in points.
const POINTER: f32 = 22.0;

/// Points along the dropper: `s` from the tip towards the bulb, `w` across it, both in box units.
fn at(s: f32, w: f32) -> (f32, f32) {
    let k = std::f32::consts::FRAC_1_SQRT_2;
    (TIP.0 + (s + w) * k, TIP.1 + (w - s) * k)
}

/// The tube, the collar and the bulb, in box units.
fn parts() -> [Vec<(f32, f32)>; 3] {
    let tube = vec![at(0.0, -0.45), at(12.0, -1.7), at(12.0, 1.7), at(0.0, 0.45)];
    let collar = vec![at(11.6, -3.2), at(14.2, -3.2), at(14.2, 3.2), at(11.6, 3.2)];
    // the bulb: straight sides, then a half circle at its end
    let (r, end) = (2.6, 19.4);
    let mut bulb = vec![at(14.0, -r)];
    let steps = 10;
    for i in 0..=steps {
        let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / steps as f32;
        bulb.push(at(end + r * a.cos(), r * a.sin()));
    }
    bulb.push(at(14.0, r));
    [tube, collar, bulb]
}

/// The parts placed in `rect`.
fn placed(rect: Rect) -> [Vec<Pos2>; 3] {
    let scale = rect.width().min(rect.height()) / 20.0;
    parts().map(|part| part.into_iter().map(|(x, y)| rect.min + vec2(x, y) * scale).collect())
}

/// The eyedropper in `rect`, as a button shows it: an outlined glass tube, a solid collar and bulb.
pub(crate) fn paint(p: &Painter, rect: Rect, color: Color32) {
    let [tube, collar, bulb] = placed(rect);
    p.add(Shape::convex_polygon(tube, color.gamma_multiply(0.25), Stroke::new(1.2, color)));
    p.add(Shape::convex_polygon(collar, color, Stroke::NONE));
    p.add(Shape::convex_polygon(bulb, color, Stroke::NONE));
}

/// The WB row's White Balance Selector button (`icon:wbPicker`, as the Modern style's): the eyedropper, on a lit
/// square while picking.
pub(crate) fn button(ui: &mut Ui, active: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let tooltip = crate::i18n::tr("White Balance Selector (W)");
    let (r, resp) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, active, tooltip));
    register(ui, "icon:wbPicker", r);
    let p = ui.painter();
    if active {
        p.rect_filled(r, 4.0, t.tool_active);
    } else if resp.hovered() {
        p.rect_filled(r, 4.0, t.hover.gamma_multiply(0.7));
    }
    let c = if active {
        t.pick
    } else if resp.hovered() {
        t.text
    } else {
        t.icon
    };
    paint(p, Rect::from_center_size(r.center(), vec2(22.0, 22.0)), c);
    resp.on_hover_text(tooltip)
}

/// Over the photo while the White Balance Selector picks, in the Classic style: the dropper in place of the system
/// pointer, its tip on the pointer. False (nothing drawn) otherwise.
pub(crate) fn pointer(app: &LightcraftApp, ui: &Ui) -> bool {
    if !crate::classic::active(app) || app.ui.tool != "wbPicker" {
        return false;
    }
    let Some(at) = ui.ctx().pointer_hover_pos() else {
        return false;
    };
    ui.ctx().set_cursor_icon(CursorIcon::None);
    let p = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::unique("classicEyedropper")));
    let [tube, collar, bulb] = placed(pointer_rect(at));
    // white on a dark edge, to read over light and dark photos alike
    let edge = Stroke::new(1.0, Color32::from_black_alpha(220));
    for part in [tube, collar, bulb] {
        p.add(Shape::convex_polygon(part, Color32::WHITE, edge));
    }
    true
}

/// Where the pointer's dropper goes so that its tip is on `at`.
fn pointer_rect(at: Pos2) -> Rect {
    let scale = POINTER / 20.0;
    Rect::from_min_size(at - vec2(TIP.0, TIP.1) * scale, vec2(POINTER, POINTER))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    /// The tip is the dropper's lowest, leftmost point, and every part stays inside its box.
    #[test]
    fn the_tip_is_at_the_bottom_left_and_the_parts_fit_their_box() {
        let all: Vec<(f32, f32)> = parts().into_iter().flatten().collect();
        for &(x, y) in &all {
            assert!((0.0..=20.0).contains(&x) && (0.0..=20.0).contains(&y), "({x}, {y}) leaves the box");
        }
        let lowest = all.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
        let leftmost = all.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
        assert!((lowest - TIP.1).abs() < 0.5 && (leftmost - TIP.0).abs() < 0.5, "tip {lowest} {leftmost}");
    }

    /// The pointer's dropper puts its tip exactly on the pointer.
    #[test]
    fn the_pointer_tip_is_on_the_pointer() {
        let at = pos2(140.0, 75.0);
        let r = pointer_rect(at);
        let tip = r.min + vec2(TIP.0, TIP.1) * (r.width() / 20.0);
        assert!(tip.distance(at) < 1e-3, "{tip:?}");
    }
}
