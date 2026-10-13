//! The Navigator panel of the Classic Develop module: the whole photo, with the part the loupe shows outlined while
//! zoomed in (click or drag there to move it), and the zoom steps under it.

use egui::{Color32, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use lightcraft_catalog::PhotoId;
use serde_json::json;

use crate::LightcraftApp;
use crate::render::Slot;
use crate::state::{ViewMode, Zoom};
use crate::theme::Tokens;
use crate::widgets::{register, text_button};

/// Height of the photo's area (points).
const PHOTO_H: f32 = 168.0;

pub(super) fn show(app: &mut LightcraftApp, ui: &mut egui::Ui, id: PhotoId) {
    let t = Tokens::get(ui.ctx());
    let (area, _) = ui.allocate_exact_size(vec2(ui.available_width(), PHOTO_H), Sense::hover());
    ui.painter().rect_filled(area, 0.0, t.canvas);
    // The loupe's own render when it is of this photo (it follows a slider as it moves), else the thumbnail, which
    // the filmstrip keeps too.
    super::super::grid::request_thumb(app, id, 256, 8);
    let tex = app.renderer.textures.get(&Slot::Main).filter(|tex| tex.photo == id).or_else(|| app.renderer.thumb(id));
    if let Some(tex) = tex {
        let [w, h] = tex.size.map(|edge| edge.max(1) as f32);
        let fit = area.shrink(8.0);
        let scale = (fit.width() / w).min(fit.height() / h);
        let frame = Rect::from_center_size(fit.center(), vec2(w * scale, h * scale));
        ui.painter().image(tex.tex.id(), frame, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        register(ui.ctx(), "classicNavigator", frame);
        // the part of the photo in the loupe: its canvas and image rectangles are last frame's, which is this
        // frame's too unless the view is changing
        let loupe = match (app.ui.view, app.canvas_rect, app.image_rect) {
            (ViewMode::Detail, Some(canvas), Some(img)) if img.width() > canvas.width() + 1.0 || img.height() > canvas.height() + 1.0 => {
                Some((canvas, img))
            }
            _ => None,
        };
        if let Some((canvas, img)) = loupe {
            let seen = canvas.intersect(img);
            let place = |q: egui::Pos2| {
                let (u, v) = ((q.x - img.left()) / img.width().max(1.0), (q.y - img.top()) / img.height().max(1.0));
                pos2(frame.left() + u.clamp(0.0, 1.0) * frame.width(), frame.top() + v.clamp(0.0, 1.0) * frame.height())
            };
            let view = Rect::from_min_max(place(seen.min), place(seen.max));
            let p = ui.painter_at(area);
            p.rect_stroke(view, 0.0, Stroke::new(1.5, Color32::WHITE), StrokeKind::Inside);
            p.rect_stroke(view.expand(1.5), 0.0, Stroke::new(1.0, Color32::from_black_alpha(160)), StrokeKind::Outside);
            let resp = ui.interact(frame, egui::Id::new("classic-navigator"), Sense::click_and_drag());
            if (resp.clicked() || resp.dragged())
                && let Some(q) = resp.interact_pointer_pos()
            {
                // centre the loupe on the point under the pointer
                app.ui.pan = (
                    ((q.x - frame.left()) / frame.width().max(1.0)).clamp(0.0, 1.0),
                    ((q.y - frame.top()) / frame.height().max(1.0)).clamp(0.0, 1.0),
                );
            }
        }
    }
    egui::Frame::NONE.inner_margin(egui::Margin { left: 12, right: 12, top: 6, bottom: 8 }).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if text_button(ui, "navigatorFit", crate::i18n::tr("Fit"), app.ui.zoom == Zoom::Fit).clicked() {
                let _ = app.run("view.zoomFit", json!({}));
            }
            if text_button(ui, "navigator100", "100%", app.ui.zoom == Zoom::Percent(100.0)).clicked() {
                let _ = app.run("view.zoom100", json!({}));
            }
        });
    });
}
