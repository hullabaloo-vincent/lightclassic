//! The left column of the Classic Develop module: Navigator, Presets, Snapshots and History, each under a header that
//! folds it.
//!
//! The Navigator is as tall as it needs; the three lists share what is left of the column, each scrolling by itself.
//! Their contents are the Modern style's own panels: the Presets column, and the bodies of Versions and History.

use egui::{Align2, Rect, Sense, Stroke, pos2, vec2};
use lightcraft_catalog::PhotoId;
use serde_json::json;

use crate::LightcraftApp;
use crate::classic::LeftPanel;
use crate::icons::{Icon, paint};
use crate::theme::Tokens;
use crate::widgets::{icon_button, register};

/// Height of a panel's header.
const HEADER_H: f32 = 32.0;
/// Height of the Presets panel's own title row (`presets::show`), which this column hides under its header: the
/// row's two buttons are in the header instead.
const PRESETS_TITLE_H: f32 = 46.0;

pub(super) fn show(app: &mut LightcraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Frame::NONE.fill(t.chrome).stroke(Stroke::new(1.0, t.divider)).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let Some(id) = app.session.active() else {
            let r = ui.max_rect();
            super::super::empty_message(ui, r, "No photo selected", "Choose a photo in the filmstrip");
            return;
        };
        let (header, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEADER_H), Sense::hover());
        let open = LeftPanel::Navigator.open(&app.ui);
        if open {
            super::navigator::show(app, ui, id);
        }
        if header_at(ui, header, LeftPanel::Navigator, open).clicked() {
            fold(app, LeftPanel::Navigator);
        }
        // the lists share the rest of the column by weight
        let lists = [(LeftPanel::Presets, 2.0), (LeftPanel::Snapshots, 1.0), (LeftPanel::History, 2.0)];
        let weights: f32 = lists.iter().filter(|(panel, _)| panel.open(&app.ui)).map(|(_, weight)| weight).sum();
        let rest = (ui.available_height() - HEADER_H * lists.len() as f32).max(0.0);
        for (panel, weight) in lists {
            let (header, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEADER_H), Sense::hover());
            let open = panel.open(&app.ui);
            if open {
                let height = (rest * weight / weights.max(1.0)).floor();
                let (body, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
                list(app, ui, panel, body, id);
            }
            // after its body: the header's buttons are then the last registered under their ids (Presets draws its
            // own, hidden, pair)
            let resp = header_at(ui, header, panel, open);
            if panel == LeftPanel::Presets {
                preset_buttons(app, ui, header);
            }
            if resp.clicked() {
                fold(app, panel);
            }
        }
    });
}

fn fold(app: &mut LightcraftApp, panel: LeftPanel) {
    let _ = app.run("view.classicPanel", json!({"panel": panel.id()}));
}

/// A list panel's contents in `body`, clipped to it.
fn list(app: &mut LightcraftApp, ui: &mut egui::Ui, panel: LeftPanel, body: Rect, id: PhotoId) {
    let clip = body.intersect(ui.clip_rect());
    match panel {
        LeftPanel::Navigator => {}
        LeftPanel::Presets => {
            // the panel as the Modern style shows it, moved up by its title row so that the row is clipped away
            let moved = Rect::from_min_max(pos2(body.left(), body.top() - PRESETS_TITLE_H), body.max);
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(moved));
            child.set_clip_rect(clip);
            super::super::presets::show(app, &mut child);
        }
        LeftPanel::Snapshots | LeftPanel::History => {
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));
            child.set_clip_rect(clip);
            egui::ScrollArea::vertical().id_salt(("classic-left", panel.id())).auto_shrink([false, false]).show(&mut child, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                if panel == LeftPanel::Snapshots {
                    super::super::right::versions_body(app, ui, id);
                } else {
                    super::super::right::activity_body(app, ui, id);
                }
            });
        }
    }
}

/// A left panel's header in `r` (`classicPanel:{id}`): the triangle that folds it and its title, at the left as
/// Classic's left panels have them.
fn header_at(ui: &mut egui::Ui, r: Rect, panel: LeftPanel, open: bool) -> egui::Response {
    let title = crate::i18n::tr(panel.title());
    let t = Tokens::get(ui.ctx());
    let resp = ui.interact(r, ui.id().with(("classicPanel", panel.id())), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::CollapsingHeader, true, open, title));
    register(ui.ctx(), format!("classicPanel:{}", panel.id()), r);
    let p = ui.painter();
    p.rect_filled(r, 0.0, if ui.rect_contains_pointer(r) { t.pressed } else { t.hover });
    p.hline(r.x_range(), r.bottom() - 0.5, Stroke::new(1.0, t.divider));
    let triangle = Rect::from_center_size(pos2(r.left() + 16.0, r.center().y), vec2(12.0, 12.0));
    paint(p, triangle, if open { Icon::ChevronDown } else { Icon::ChevronRight }, t.text_label);
    p.text(pos2(r.left() + 30.0, r.center().y), Align2::LEFT_CENTER, title, t.semibold(13.0), t.text);
    resp
}

/// Create Preset and the options menu, at the right of the Presets header: the two buttons of the Presets panel's own
/// title row (`icon:presetCreate`, `icon:presetMore`).
fn preset_buttons(app: &mut LightcraftApp, ui: &mut egui::Ui, header: Rect) {
    let area = Rect::from_min_max(pos2(header.right() - 70.0, header.top()), pos2(header.right() - 6.0, header.bottom()));
    let mut row = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(egui::Layout::right_to_left(egui::Align::Center)));
    row.spacing_mut().item_spacing.x = 2.0;
    let more = icon_button(&mut row, "presetMore", Icon::More, vec2(26.0, 26.0), false, true, crate::i18n::tr("More preset options"));
    egui::Popup::menu(&more).show(|ui| {
        let r = ui.checkbox(&mut app.ui.preset_thumbs, crate::i18n::tr("Show Thumbnails"));
        register(ui.ctx(), "presetMenu:thumbnails", r.rect);
        ui.separator();
        if ui.button(crate::i18n::tr("Import Presets…")).clicked() {
            let _ = app.run("file.importPresets", json!({}));
        }
        let any_user = app.session.presets.iter().any(|p| !p.builtin);
        if ui.add_enabled(any_user, egui::Button::new(crate::i18n::tr("Export User Presets…"))).clicked() {
            let _ = app.run("file.exportPresets", json!({}));
        }
    });
    if icon_button(&mut row, "presetCreate", Icon::Plus, vec2(26.0, 26.0), false, app.session.active().is_some(), crate::i18n::tr("Create Preset…"))
        .clicked()
    {
        let _ = app.run("dialog.createPreset", json!({}));
    }
}
