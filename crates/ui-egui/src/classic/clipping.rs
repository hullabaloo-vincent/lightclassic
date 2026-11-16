//! Lightroom Classic's histogram clipping triangles: the left one shows clipped shadows on the photo (blue), the right
//! one clipped highlights (red), each on its own; J (`view.clipping`) shows or hides both, as the Modern style's single
//! switch does.
//!
//! Both sides on is upstream's own state (`UiState::show_clipping`, [`Overlay::ClipWarnings`]); one side alone is kept
//! here ([`ClassicState::clip_shadows`] / [`ClassicState::clip_highlights`]) and shown with the one-sided overlays.
//! [`normalize`] keeps the two from overlapping: both sides on is always `show_clipping`.
//!
//! [`ClassicState::clip_shadows`]: super::ClassicState::clip_shadows
//! [`ClassicState::clip_highlights`]: super::ClassicState::clip_highlights

use egui::{Color32, Painter, Rect, Sense, Shape, Stroke, StrokeKind, pos2, vec2};
use lightcraft_pipeline::Overlay;
use serde_json::{Value, json};

use crate::LightcraftApp;
use crate::state::UiState;
use crate::theme::Tokens;
use crate::widgets::register;

/// A side of the histogram: `shadows` or `highlights`, as the buttons' widget ids name them (`histogram:{side}`).
pub(crate) fn is_side(side: &str) -> bool {
    matches!(side, "shadows" | "highlights")
}

/// Whether the photo shows `side`'s clipping.
pub(crate) fn side_on(ui: &UiState, side: &str) -> bool {
    ui.show_clipping
        || match side {
            "shadows" => ui.classic.clip_shadows,
            _ => ui.classic.clip_highlights,
        }
}

/// Both sides on is `show_clipping` alone; otherwise the one side on is kept here and `show_clipping` is off.
pub(crate) fn normalize(ui: &mut UiState) {
    if ui.show_clipping || (ui.classic.clip_shadows && ui.classic.clip_highlights) {
        ui.show_clipping = true;
        ui.classic.clip_shadows = false;
        ui.classic.clip_highlights = false;
    }
}

/// `view.clippingSide {side, on?}`: show or hide one side's clipping (`on` omitted: toggle it).
pub(crate) fn command(app: &mut LightcraftApp, p: &Value) -> Result<Value, String> {
    let side = p.get("side").and_then(Value::as_str).unwrap_or_default();
    if !is_side(side) {
        return Err("view.clippingSide: `side` is shadows or highlights".to_string());
    }
    let ui = &mut app.ui;
    let on = p.get("on").and_then(Value::as_bool).unwrap_or(!side_on(ui, side));
    let (mut shadows, mut highlights) = (side_on(ui, "shadows"), side_on(ui, "highlights"));
    if side == "shadows" {
        shadows = on;
    } else {
        highlights = on;
    }
    ui.show_clipping = false;
    ui.classic.clip_shadows = shadows;
    ui.classic.clip_highlights = highlights;
    normalize(ui);
    Ok(json!({"shadows": shadows, "highlights": highlights}))
}

/// The overlay for one side shown alone; `None` when both or neither show (upstream decides those).
pub(crate) fn overlay(ui: &UiState) -> Option<Overlay> {
    if ui.show_clipping {
        return None;
    }
    match (ui.classic.clip_shadows, ui.classic.clip_highlights) {
        (true, false) => Some(Overlay::ClipWarningsShadows),
        (false, true) => Some(Overlay::ClipWarningsHighlights),
        _ => None,
    }
}

/// The histogram's two top corners in the Classic style (`[shadows, highlights]`): a triangle for each side's
/// clipping (`histogram:shadows`, `histogram:highlights`), or the gamut warnings while soft proofing
/// ([`super::proof::corners`]). False, drawing nothing, in the Modern style, whose own buttons go there instead.
pub(crate) fn corners(app: &mut LightcraftApp, ui: &egui::Ui, rects: [Rect; 2], clipping: Option<(f32, f32)>) -> bool {
    if !super::active(app) {
        return false;
    }
    if app.ui.soft_proof {
        super::proof::corners(app, ui, rects);
        return true;
    }
    let t = Tokens::get(ui.ctx());
    let mut clicked = None;
    for (name, rect, label, clipped) in [
        ("shadows", rects[0], "Shadow clipping", clipping.map(|(lo, _)| lo > 0.003)),
        ("highlights", rects[1], "Highlight clipping", clipping.map(|(_, hi)| hi > 0.003)),
    ] {
        let response = ui.interact(rect, ui.scope_id().with(("histogram-clipping", name)), Sense::click());
        register(ui, format!("histogram:{name}"), rect);
        let active = side_on(&app.ui, name);
        response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), active, crate::i18n::tr(label)));
        paint(ui.painter(), rect, &t, active, response.hovered(), clipped == Some(true));
        let how = match (name == "shadows", active) {
            (true, true) => "Click to hide clipped shadows (blue). J shows or hides both sides.",
            (true, false) => "Click to show clipped shadows in blue. J shows or hides both sides.",
            (false, true) => "Click to hide clipped highlights (red). J shows or hides both sides.",
            (false, false) => "Click to show clipped highlights in red. J shows or hides both sides.",
        };
        let status = match clipped {
            Some(true) => "Clipping detected in this preview.",
            Some(false) => "No clipping detected in this preview.",
            None => "Histogram not yet available.",
        };
        let tip = format!("{}\n{}\n{}", crate::i18n::tr(label), crate::i18n::tr(how), crate::i18n::tr(status));
        if response.on_hover_text(tip).clicked() {
            clicked = Some(name);
        }
    }
    if let Some(side) = clicked {
        let _ = app.run("view.clippingSide", json!({"side": side}));
    }
    true
}

/// A clipping button as Classic draws it: a triangle that is bright when this side of the photo clips, and a box
/// round it while its warnings show.
pub(crate) fn paint(p: &Painter, rect: Rect, t: &Tokens, active: bool, hovered: bool, clipped: bool) {
    let r = Rect::from_center_size(rect.center(), vec2(16.0, 16.0));
    if active || hovered {
        p.rect_filled(r, 2.0, if active { t.pressed } else { t.hover });
    }
    if active {
        p.rect_stroke(r, 2.0, Stroke::new(1.0, t.text), StrokeKind::Inside);
    }
    let c = r.center();
    let fill = if clipped { t.text } else { Color32::from_gray(120) };
    let tri = vec![pos2(c.x, c.y - 4.5), pos2(c.x + 5.0, c.y + 4.0), pos2(c.x - 5.0, c.y + 4.0)];
    p.add(Shape::convex_polygon(tri, fill, Stroke::NONE));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ui() -> UiState {
        UiState::default()
    }

    /// Each triangle turns its own side on and off; both on is the J state, and J then turns both off.
    #[test]
    fn each_side_toggles_on_its_own() {
        let mut u = ui();
        assert!(!side_on(&u, "shadows") && !side_on(&u, "highlights"));
        let toggle = |u: &mut UiState, side: &str| {
            let on = !side_on(u, side);
            let (mut s, mut h) = (side_on(u, "shadows"), side_on(u, "highlights"));
            if side == "shadows" {
                s = on
            } else {
                h = on
            }
            u.show_clipping = false;
            u.classic.clip_shadows = s;
            u.classic.clip_highlights = h;
            normalize(u);
        };
        toggle(&mut u, "shadows");
        assert!(side_on(&u, "shadows") && !side_on(&u, "highlights"));
        assert_eq!(overlay(&u), Some(Overlay::ClipWarningsShadows));
        toggle(&mut u, "highlights");
        assert!(u.show_clipping && !u.classic.clip_shadows && !u.classic.clip_highlights, "both: the J state");
        assert_eq!(overlay(&u), None, "upstream draws both");
        toggle(&mut u, "shadows");
        assert!(!u.show_clipping && side_on(&u, "highlights") && !side_on(&u, "shadows"));
        assert_eq!(overlay(&u), Some(Overlay::ClipWarningsHighlights));
        // J from one side shows both (normalize runs after every command); J again hides both
        u.show_clipping = true;
        normalize(&mut u);
        assert!(side_on(&u, "shadows") && side_on(&u, "highlights"));
        u.show_clipping = false;
        assert!(!side_on(&u, "shadows") && !side_on(&u, "highlights"));
    }
}
