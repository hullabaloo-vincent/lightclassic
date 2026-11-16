//! Lightroom Classic's soft proofing. A Soft Proofing checkbox sits at the right end of the toolbar under the photo
//! (S does the same). While it is on, the Histogram panel is titled Proof Histogram. Its top corners hold the two
//! gamut warnings in place of the clipping triangles: the display's at the left (blue on the photo) and the
//! destination's at the right (red). The proof's options sit under the plot: Profile, Intent, Simulate Paper & Ink
//! and Create Proof Copy.
//!
//! Every control runs upstream's `view.softProof` and keeps the widget ids of the Modern style's Soft Proofing strip
//! (`panels/edit/histogram.rs`), which the Classic style does not draw.

use egui::{Color32, Painter, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use lightcraft_catalog::PhotoId;
use serde_json::{Value, json};

use crate::LightcraftApp;
use crate::classic::Module;
use crate::state::ViewMode;
use crate::theme::Tokens;
use crate::widgets::register;

/// The colours the photo shows each gamut warning in: the display's (blue) and the destination's (red).
const DISPLAY_WARNING: Color32 = Color32::from_rgb(70, 120, 255);
const DEST_WARNING: Color32 = Color32::from_rgb(255, 60, 60);

/// Width of the options' names, right-aligned against their values.
const NAME_W: f32 = 64.0;

/// The Soft Proofing checkbox (`check:softProof`) at the right end of the toolbar under the photo, in the Develop
/// module's loupe; the Modern style and the other views draw nothing. `ui` lays out right to left.
pub(crate) fn toolbar_checkbox(app: &mut LightcraftApp, ui: &mut Ui) {
    if !crate::classic::active(app) || app.ui.classic.module != Module::Develop || app.ui.view != ViewMode::Detail {
        return;
    }
    let mut on = app.ui.soft_proof;
    let r = ui.checkbox(&mut on, crate::i18n::tr("Soft Proofing"));
    register(ui, "check:softProof", r.rect);
    if r.on_hover_text(crate::i18n::tr("Preview how the photo will look printed or exported (S)")).changed() {
        let _ = app.run("view.softProof", json!({"on": on}));
    }
}

/// The Histogram panel's title: Proof Histogram while soft proofing.
pub(crate) fn histogram_title(app: &LightcraftApp) -> &'static str {
    if app.ui.soft_proof { "Proof Histogram" } else { "Histogram" }
}

/// What a corner button shows.
#[derive(Clone, Copy)]
enum Glyph {
    /// A display: the gamut warning for the screen.
    Display,
    /// A sheet of paper: the gamut warning for the destination (printer or export space).
    Page,
}

/// The histogram's two top corners while soft proofing: the display gamut warning at the left
/// (`button:proofDisplayWarning`), the destination's at the right (`button:proofDestWarning`).
pub(crate) fn corners(app: &mut LightcraftApp, ui: &Ui, [left, right]: [Rect; 2]) {
    let t = Tokens::get(ui.ctx());
    let pr = app.ui.proof;
    let mut change = None;
    for (rect, id, on, colour, glyph, label) in [
        (left, "proofDisplayWarning", pr.display_warning, DISPLAY_WARNING, Glyph::Display, "Show display gamut warning"),
        (right, "proofDestWarning", pr.dest_warning, DEST_WARNING, Glyph::Page, "Show destination gamut warning"),
    ] {
        let resp = ui.interact(rect, ui.scope_id().with(("proof-warning", id)), Sense::click());
        register(ui, format!("button:{id}"), rect);
        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, crate::i18n::tr(label)));
        paint_corner(ui.painter(), rect, &t, on, resp.hovered(), colour, glyph);
        let how = match (glyph, on) {
            (Glyph::Display, false) => "Show in blue the colours this display cannot show",
            (Glyph::Display, true) => "Hide the display gamut warning",
            (Glyph::Page, false) => "Show in red the colours the proof profile cannot hold",
            (Glyph::Page, true) => "Hide the destination gamut warning",
        };
        if resp.on_hover_text(format!("{}\n{}", crate::i18n::tr(label), crate::i18n::tr(how))).clicked() {
            change = Some(warning_params(glyph, !on));
        }
    }
    if let Some(p) = change {
        let _ = app.run("view.softProof", p);
    }
}

/// `view.softProof`'s parameters to turn one gamut warning on or off.
fn warning_params(glyph: Glyph, on: bool) -> Value {
    match glyph {
        Glyph::Display => json!({"displayWarning": on}),
        Glyph::Page => json!({"destWarning": on}),
    }
}

/// A gamut warning's corner button: boxed while its warning shows, the glyph in the warning's colour then.
fn paint_corner(p: &Painter, rect: Rect, t: &Tokens, on: bool, hovered: bool, colour: Color32, glyph: Glyph) {
    let r = Rect::from_center_size(rect.center(), vec2(16.0, 16.0));
    if on || hovered {
        p.rect_filled(r, 2.0, if on { t.pressed } else { t.hover });
    }
    if on {
        p.rect_stroke(r, 2.0, Stroke::new(1.0, t.text), StrokeKind::Inside);
    }
    let ink = if on { colour } else { Color32::from_gray(150) };
    let c = r.center();
    match glyph {
        Glyph::Display => {
            // the screen, and its stand
            let screen = Rect::from_center_size(pos2(c.x, c.y - 1.0), vec2(11.0, 7.5));
            p.rect_stroke(screen, 1.0, Stroke::new(1.2, ink), StrokeKind::Inside);
            p.line_segment([pos2(c.x, screen.bottom()), pos2(c.x, c.y + 4.5)], Stroke::new(1.2, ink));
            p.line_segment([pos2(c.x - 3.0, c.y + 4.5), pos2(c.x + 3.0, c.y + 4.5)], Stroke::new(1.2, ink));
        }
        Glyph::Page => {
            // a sheet with its top right corner folded
            let (l, r, top, b) = (c.x - 4.0, c.x + 4.0, c.y - 5.5, c.y + 5.5);
            let outline = vec![pos2(l, top), pos2(r - 3.0, top), pos2(r, top + 3.0), pos2(r, b), pos2(l, b), pos2(l, top)];
            p.add(egui::Shape::line(outline, Stroke::new(1.2, ink)));
            p.add(egui::Shape::line(vec![pos2(r - 3.0, top), pos2(r - 3.0, top + 3.0), pos2(r, top + 3.0)], Stroke::new(1.0, ink)));
        }
    }
}

/// The proof's options under the histogram while soft proofing: Profile, Intent and Simulate Paper & Ink (both for a
/// printer profile, dimmed otherwise), and Create Proof Copy.
pub(crate) fn options(app: &mut LightcraftApp, ui: &mut Ui, id: PhotoId) {
    use lightcraft_engine::pipeline::OutputSpace;
    use lightcraft_engine::proof::ProofIntent;
    let t = Tokens::get(ui.ctx());
    let device = app.ui.proof_profile.as_ref().map(|(_, n)| n.clone());
    egui::Frame::NONE.inner_margin(egui::Margin { left: 10, right: 22, top: 8, bottom: 8 }).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        // Profile : <space or printer profile>
        ui.horizontal(|ui| {
            name(ui, &t, "Profile");
            let cur = app.ui.proof.space;
            let shown = device.clone().unwrap_or_else(|| cur.label().to_string());
            let mut pick = false;
            let width = (ui.available_width() - 4.0).clamp(80.0, 190.0);
            // as the Modern strip's (`panels/edit/histogram.rs`)
            let combo = crate::widgets::combo_box("proof-space").width(width).selected_text(shown).show_ui(ui, |ui| {
                for s in OutputSpace::ALL {
                    if ui.selectable_label(device.is_none() && cur == s, s.label()).clicked() {
                        let _ = app.run("view.softProof", json!({"space": s}));
                    }
                }
                if let Some(n) = &device {
                    let _ = ui.selectable_label(true, n);
                }
                if app.services.pick_display_profile.is_some() && ui.selectable_label(false, crate::i18n::tr("Other…")).clicked() {
                    pick = true;
                }
            });
            crate::widgets::name_combo(&combo.response, crate::i18n::tr("Profile"));
            if pick
                && let Some(path) = app.services.pick_display_profile.as_mut().and_then(|pick| pick().into_iter().next())
                && let Err(e) = app.run("view.softProof", json!({"profile": path}))
            {
                app.toast(ui.ctx(), e);
            }
        });
        let printer = device.is_some();
        let only_printer = crate::i18n::tr("For a printer profile: choose one with Profile ▸ Other…");
        // Intent : Perceptual  Relative
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            name(ui, &t, "Intent");
            for (i, label) in [(ProofIntent::Perceptual, "Perceptual"), (ProofIntent::Relative, "Relative")] {
                let chosen = printer && app.ui.proof_intent == i;
                let colour = if !printer {
                    t.text_disabled
                } else if chosen {
                    t.text
                } else {
                    t.text_dim
                };
                let font = if chosen { t.semibold(13.0) } else { t.font(13.0) };
                let label = crate::i18n::tr(label);
                let sense = if printer { Sense::click() } else { Sense::hover() };
                let r = ui.add(egui::Label::new(RichText::new(label).font(font).color(colour)).sense(sense));
                r.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, printer, chosen, label));
                register(ui, format!("button:proofIntent-{}", i.name()), r.rect);
                let r = if printer { r } else { r.on_hover_text(only_printer) };
                if r.clicked() && app.ui.proof_intent != i {
                    let _ = app.run("view.softProof", json!({"intent": i.name()}));
                }
            }
        });
        // Simulate Paper & Ink
        ui.horizontal(|ui| {
            ui.add_space(NAME_W + ui.spacing().item_spacing.x);
            let mut paper = printer && app.ui.proof_paper_ink;
            let r = ui.add_enabled(printer, egui::Checkbox::new(&mut paper, crate::i18n::tr("Simulate Paper & Ink")));
            register(ui, "button:proofPaperInk", r.rect);
            let r = if printer { r } else { r.on_disabled_hover_text(only_printer) };
            if r.changed() {
                let _ = app.run("view.softProof", json!({"paperInk": paper}));
            }
        });
        // Create Proof Copy, at the right
        let row = vec2(ui.available_width(), 26.0);
        ui.allocate_ui_with_layout(row, egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let r = ui.button(crate::i18n::tr("Create Proof Copy"));
            register(ui, "button:createProofCopy", r.rect);
            if r.on_hover_text(crate::i18n::tr("A virtual copy of this photo to edit for this proof")).clicked() {
                let target = app.ui.proof_profile.as_ref().map_or_else(|| app.ui.proof.space.label().to_string(), |(_, n)| n.clone());
                let name = crate::i18n::tr_format!("Proof Copy ({})", target);
                if app.run("photo.virtualCopy", json!({"ids": [id.0], "name": name})).is_ok() {
                    app.toast(ui.ctx(), crate::i18n::tr("Proof copy created"));
                }
            }
        });
    });
}

/// An option's name, right-aligned in its column ("Profile :").
fn name(ui: &mut Ui, t: &Tokens, text: &str) {
    let label = format!("{} :", crate::i18n::tr(text));
    let (r, _) = ui.allocate_exact_size(vec2(NAME_W, 20.0), Sense::hover());
    ui.painter().text(pos2(r.right(), r.center().y), egui::Align2::RIGHT_CENTER, label, t.font(13.0), t.text_label);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each corner turns its own warning on or off, by the parameter `view.softProof` reads.
    #[test]
    fn each_corner_switches_its_own_warning() {
        assert_eq!(warning_params(Glyph::Display, true), json!({"displayWarning": true}));
        assert_eq!(warning_params(Glyph::Page, false), json!({"destWarning": false}));
    }
}
