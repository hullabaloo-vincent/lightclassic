//! The Classic interface style: its drawing (state and commands are in [`crate::classic`]).
//!
//! The window, top to bottom: the top bar (shared with the Modern style), the module picker, then the left column,
//! the photo area with its toolbar and the right column side by side, and the filmstrip across the whole width.
//! The panels are fixed in place, as in Lightroom Classic: the docking of the Modern style is not used here (its
//! saved arrangement is kept for when the style is switched back).

mod develop_left;
mod navigator;
mod print;

use egui::{Align2, Sense, vec2};
use serde_json::json;

use crate::LightcraftApp;
use crate::classic::{InterfaceStyle, Module};
use crate::icons::Icon;
use crate::state::{LEFT_WIDTH, RIGHT_WIDTH, RightPanel, ViewMode};
use crate::theme::Tokens;
use crate::widgets::{icon_button, register};

/// Height of the module picker's row.
const MODULE_BAR_H: f32 = 32.0;
/// Height of the Develop tool strip (Crop, Remove, Red Eye, Masking).
const TOOL_STRIP_H: f32 = 44.0;

/// Settings ▸ Interface: the style, as `button:settingsInterface-{0,1}` (Classic, Modern).
pub(super) fn settings_row(app: &mut LightcraftApp, ui: &mut egui::Ui, t: &Tokens) {
    super::settings::heading(ui, t, crate::i18n::tr("Interface Style"));
    let options = InterfaceStyle::ALL.map(|s| (s, s.label()));
    let mut style = crate::classic::style(&app.ui);
    ui.horizontal(|ui| {
        if super::settings::choices(ui, "settingsInterface", &options, &mut style) {
            let _ = app.run("view.interface", json!({"style": style}));
        }
    });
    super::settings::hint(
        ui,
        t,
        crate::i18n::tr("Classic: Library and Develop modules, with panels stacked on both sides. Modern: one window with a tool strip."),
    );
}

/// Everything below the top bar, in the Classic style.
pub fn show(app: &mut LightcraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    module_bar(app, ui, &t);
    // Order matters: the filmstrip takes the whole bottom edge, then the columns take the sides, and the toolbar
    // sits under the photo area between them.
    if app.ui.filmstrip {
        let r =
            egui::Panel::bottom("classic_filmstrip").exact_size(t.film_h).resizable(false).frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |ui| {
                let strip = ui.max_rect();
                super::detail::filmstrip(app, ui, strip);
            });
        register(ui.ctx(), "panel:filmstrip", r.response.rect);
    }
    let frame = egui::Frame::NONE.fill(t.chrome);
    // Print always has its panels; the other modules show the panel that is open
    let printing = app.ui.classic.module == Module::Print;
    let right = printing || app.ui.right != RightPanel::None;
    if app.ui.left_panel {
        let reserve = if right { RIGHT_WIDTH.min } else { 0.0 };
        let dragged = super::resizable_side(ui, true, "left_panel", frame, app.ui.left_width, LEFT_WIDTH, reserve, |ui| {
            if app.ui.classic.module == Module::Develop { develop_left::show(app, ui) } else { super::left::show(app, ui) }
        });
        if let Some(width) = dragged {
            app.ui.left_width = width;
        }
    }
    if right {
        let dragged = super::resizable_side(ui, false, "right_panel", frame, app.ui.right_width, RIGHT_WIDTH, 0.0, |ui| {
            // Develop's own column, except while the profile browser has the whole of it
            if printing {
                print::panels(app, ui);
            } else if app.ui.classic.module == Module::Develop && app.ui.right != RightPanel::Profiles {
                super::edit::classic::show(app, ui);
            } else {
                super::right::show(app, ui);
            }
        });
        if let Some(width) = dragged {
            app.ui.right_width = width;
        }
    }
    if printing {
        // the page, in place of a view of the photo and its toolbar
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |ui| print::page(app, ui));
        return;
    }
    super::bottombar::show(app, ui);
    let bg = match app.ui.view {
        ViewMode::Detail | ViewMode::Compare | ViewMode::Survey | ViewMode::Reference => t.canvas,
        ViewMode::PhotoGrid | ViewMode::SquareGrid | ViewMode::People => t.grid_bg,
    };
    super::compare::prepare(app);
    // The loupe, Compare, Survey and Reference draw a filmstrip of their own inside the photo area while the flag
    // is set; here the one across the window stands in for it.
    let filmstrip = std::mem::take(&mut app.ui.filmstrip);
    // Likewise the loupe's mini map while zoomed in: Develop's left column has the Navigator panel (folded, it is
    // the user's choice not to see one).
    let panel = app.ui.classic.module == Module::Develop && app.ui.left_panel;
    let navigator = if panel { std::mem::take(&mut app.ui.navigator) } else { false };
    // No wildcard: a view added to `ViewMode` must be placed in this frame too.
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(bg)).show(ui, |ui| match app.ui.view {
        ViewMode::PhotoGrid | ViewMode::SquareGrid => super::grid::show(app, ui),
        ViewMode::Detail => super::detail::show(app, ui),
        ViewMode::Compare => super::compare::show_compare(app, ui),
        ViewMode::Survey => super::compare::show_survey(app, ui),
        ViewMode::Reference => super::compare::show_reference(app, ui),
        ViewMode::People => super::people::show(app, ui),
    });
    // a toggle made while the view was drawn (it saw the strip as off) is kept
    app.ui.filmstrip = filmstrip != app.ui.filmstrip;
    if panel {
        app.ui.navigator = navigator != app.ui.navigator;
    }
}

/// The photos in the filmstrip across the window (the cells it drew last frame, with those just out of view), for
/// the grid above it: the grid drops the thumbnails of photos it is not showing, and these are still on screen.
/// Empty in the Modern style, whose grid has no filmstrip beside it. (Not by `ui.filmstrip`: [`show`] clears that
/// flag while the view is drawn.)
pub(super) fn filmstrip_photos(app: &LightcraftApp) -> impl Iterator<Item = lightcraft_catalog::PhotoId> + '_ {
    let classic = crate::classic::active(app);
    app.widgets.iter().filter(move |_| classic).filter_map(|(id, _)| id.strip_prefix("film:")?.parse().ok()).map(lightcraft_catalog::PhotoId)
}

/// The module picker: the modules' names at the right of their own row, the one showing in the brighter text
/// (`module:library`, `module:develop`).
fn module_bar(app: &mut LightcraftApp, ui: &mut egui::Ui, t: &Tokens) {
    let r = egui::Panel::top("classic_module_bar")
        .exact_size(MODULE_BAR_H)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin { left: 18, right: 10, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let full = ui.max_rect();
            ui.painter().text(full.left_center(), Align2::LEFT_CENTER, "LightCraft", t.semibold(13.0), t.text_dim);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for (i, module) in Module::ALL.into_iter().rev().enumerate() {
                    if i > 0 {
                        let (sep, _) = ui.allocate_exact_size(vec2(1.0, 14.0), Sense::hover());
                        ui.painter().rect_filled(sep, 0.0, t.button_border);
                    }
                    if module_button(ui, t, module, app.ui.classic.module == module).clicked() {
                        let _ = app.run(module.command(), json!({}));
                    }
                }
            });
        });
    let bar = r.response.rect;
    ui.painter().hline(bar.x_range(), bar.bottom() - 0.5, egui::Stroke::new(1.0, t.divider));
}

fn module_button(ui: &mut egui::Ui, t: &Tokens, module: Module, active: bool) -> egui::Response {
    let label = crate::i18n::tr(module.label());
    let font = t.semibold(14.0);
    let width = ui.painter().layout_no_wrap(label.to_string(), font.clone(), t.text).size().x;
    let (r, resp) = ui.allocate_exact_size(vec2(width + 28.0, MODULE_BAR_H), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, active, label));
    register(ui.ctx(), format!("module:{}", module.id()), r);
    let color = if active {
        t.text
    } else if resp.hovered() {
        t.text_label
    } else {
        t.text_dim
    };
    ui.painter().text(r.center(), Align2::CENTER_CENTER, label, font, color);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Develop's tools in a row under the histogram (`icon:crop`, `icon:remove`, `icon:redeye`, `icon:masking`): a tool's
/// button opens its panel under the row, above the develop panels, and again puts it away.
pub(super) fn tool_strip(app: &mut LightcraftApp, ui: &mut egui::Ui, t: &Tokens) {
    let has_photo = app.session.active().is_some();
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), TOOL_STRIP_H), Sense::hover());
    ui.painter().rect_filled(r, 0.0, t.chrome);
    let mut row = ui.new_child(egui::UiBuilder::new().max_rect(r.shrink2(vec2(14.0, 0.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
    row.spacing_mut().item_spacing.x = 6.0;
    for (id, icon, panel, tip) in [
        ("crop", Icon::Crop, RightPanel::Crop, "Crop & Rotate (C)"),
        ("remove", Icon::Eraser, RightPanel::Remove, "Remove (H)"),
        ("redeye", Icon::Eye, RightPanel::RedEye, "Red Eye"),
        ("masking", Icon::Mask, RightPanel::Masking, "Masking (M)"),
    ] {
        if icon_button(&mut row, id, icon, vec2(40.0, 34.0), app.ui.right == panel, has_photo, tip).clicked() {
            let _ = app.run(&format!("panel.{id}"), json!({}));
        }
    }
    ui.painter().hline(r.x_range(), r.bottom() - 0.5, egui::Stroke::new(1.0, t.divider));
}

#[cfg(test)]
mod tests;
