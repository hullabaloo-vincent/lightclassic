//! The Print module: the page with the photo on it, in the middle of the window, and its panels on the right
//! (Layout, Photo, Page, Print Job). State and commands are in [`crate::classic::print`].
//!
//! The page on screen is the file that will be written, scaled down: the photo's rectangle comes from the same
//! [`lightcraft_engine::export::Canvas`] the export uses, so what is seen is what is printed.

use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use lightcraft_catalog::PhotoId;
use lightcraft_develop::{ControlSpec, Section, Track};
use lightcraft_engine::export::{CANVAS_EDGE_RANGE, ExportOptions};
use serde_json::json;

use crate::LightcraftApp;
use crate::classic::print::{PAGE_COLOURS, PAGE_SHAPES, PrintFormat, PrintLayout};
use crate::render::Slot;
use crate::theme::Tokens;
use crate::widgets::{register, segmented, slider, text_button};

/// How near the middle of the page (as a fraction of it) a dragged photo snaps to it.
const SNAP: f32 = 0.012;

/// Change the page (`print.set`); what cannot be applied is said in a toast.
fn set(app: &mut LightcraftApp, ctx: &egui::Context, params: serde_json::Value) {
    if let Err(e) = app.run("print.set", params) {
        app.toast(ctx, e);
    }
}

/// The photo's size and place on the page, in page pixels: (width, height, left, top).
fn placed(app: &LightcraftApp, id: PhotoId) -> Option<(usize, usize, i64, i64)> {
    let photo = app.session.catalog.photo(id)?;
    let canvas = app.ui.classic.print.canvas;
    let (w, h) = lightcraft_engine::export::output_size(photo, &ExportOptions { canvas: Some(canvas), ..Default::default() });
    let (left, top) = canvas.photo_origin(w, h);
    Some((w, h, left, top))
}

/// The page, fitted into the middle of the window, with the active photo where the layout puts it (`printPage`,
/// `printPhoto`). Dragging the photo moves it; it stays on the page and catches on the page's centre lines.
pub(super) fn page(app: &mut LightcraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let area = ui.max_rect();
    app.canvas_rect = Some(area);
    app.image_rect = None;
    ui.painter().rect_filled(area, 0.0, t.canvas);
    let canvas = app.ui.classic.print.canvas;
    let room = area.shrink(36.0);
    if room.width() < 8.0 || room.height() < 8.0 {
        return;
    }
    let scale = (room.width() / canvas.width.max(1) as f32).min(room.height() / canvas.height.max(1) as f32);
    let page = Rect::from_center_size(room.center(), vec2(canvas.width as f32 * scale, canvas.height as f32 * scale));
    let [r, g, b] = canvas.background;
    ui.painter().rect_filled(page.translate(vec2(0.0, 3.0)).expand(2.0), 2.0, Color32::from_black_alpha(90));
    ui.painter().rect_filled(page, 0.0, Color32::from_rgb(r, g, b));
    register(ui.ctx(), "printPage", page);
    let Some(id) = app.session.active() else {
        ui.painter().text(
            page.center(),
            Align2::CENTER_CENTER,
            crate::i18n::tr("Choose a photo in the filmstrip"),
            t.font(13.0),
            Color32::from_gray(128),
        );
        return;
    };
    let Some((w, h, left, top)) = placed(app, id) else { return };
    let photo = Rect::from_min_size(page.min + vec2(left as f32 * scale, top as f32 * scale), vec2(w as f32 * scale, h as f32 * scale));
    register(ui.ctx(), "printPhoto", photo);
    // The loupe's render when it is of this photo, else the thumbnail (asked for at its largest).
    super::super::grid::request_thumb(app, id, 512, 9);
    let tex = app.renderer.textures.get(&Slot::Main).filter(|tex| tex.photo == id).or_else(|| app.renderer.thumb(id));
    let on_page = ui.painter().with_clip_rect(page);
    match tex {
        Some(tex) => {
            on_page.image(tex.tex.id(), photo, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        }
        None => {
            on_page.rect_filled(photo, 0.0, Color32::from_gray(96));
        }
    }
    let resp = ui.interact(photo.intersect(page), egui::Id::new("print-photo"), Sense::drag()).on_hover_cursor(egui::CursorIcon::Grab);
    if resp.hovered() || resp.dragged() {
        on_page.rect_stroke(photo, 0.0, Stroke::new(1.0, t.accent), StrokeKind::Inside);
    }
    if resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        // The drag is kept whole between frames, apart from the snapped place it shows: a photo caught on a centre
        // line then comes off it again when the pointer moves on.
        let key = egui::Id::new("print-photo-at");
        let delta = resp.drag_delta();
        let (x, y) = ui.data(|d| d.get_temp::<(f32, f32)>(key)).unwrap_or((canvas.x, canvas.y));
        let (x, y) = ((x + delta.x / page.width().max(1.0)).clamp(0.0, 1.0), (y + delta.y / page.height().max(1.0)).clamp(0.0, 1.0));
        ui.data_mut(|d| d.insert_temp(key, (x, y)));
        // on the page, and on its centre lines when near them
        let settle = |at: f32, (lo, hi): (f32, f32)| {
            let at = at.clamp(lo, hi);
            if (at - 0.5).abs() < SNAP { 0.5 } else { at }
        };
        let (sx, sy) = (settle(x, PrintLayout::centre_range(w, canvas.width)), settle(y, PrintLayout::centre_range(h, canvas.height)));
        if (sx, sy) != (canvas.x, canvas.y) {
            set(app, ui.ctx(), json!({"x": sx, "y": sy}));
        }
        let guide = Stroke::new(1.0, t.accent);
        if sx == 0.5 {
            ui.painter().line_segment([pos2(page.center().x, page.top()), pos2(page.center().x, page.bottom())], guide);
        }
        if sy == 0.5 {
            ui.painter().line_segment([pos2(page.left(), page.center().y), pos2(page.right(), page.center().y)], guide);
        }
    } else {
        ui.data_mut(|d| d.remove::<(f32, f32)>(egui::Id::new("print-photo-at")));
    }
}

/// A panel's title band, as the Develop panels have theirs.
fn title(ui: &mut egui::Ui, id: &str, text: &str) {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
    register(ui.ctx(), format!("classicPanel:{id}"), r);
    ui.painter().rect_filled(r, 0.0, t.hover);
    ui.painter().hline(r.x_range(), r.bottom() - 0.5, Stroke::new(1.0, t.divider));
    ui.painter().text(pos2(r.right() - 20.0, r.center().y), Align2::RIGHT_CENTER, crate::i18n::tr(text), t.semibold(13.0), t.text);
}

fn padded(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::NONE.inner_margin(egui::Margin { left: 24, right: 22, top: 10, bottom: 10 }).show(ui, add);
}

/// A slider of the Print panels (`slider:print.<name>`); returns the value it was moved to.
fn number(ui: &mut egui::Ui, id: &'static str, label: &'static str, range: (f64, f64), step: f64, value: f64) -> Option<f64> {
    let spec =
        ControlSpec { id, label, section: Section::Profile, min: range.0, max: range.1, default: value, step, decimals: 0, track: Track::Plain };
    slider(ui, &spec, value.clamp(range.0, range.1), true, None).value
}

/// The Print panels: Layout (the page), Photo (its size and place), Page (the background) and Print Job (the file).
pub(super) fn panels(app: &mut LightcraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Frame::NONE.fill(t.chrome).stroke(Stroke::new(1.0, t.divider)).show(ui, |ui| {
        egui::ScrollArea::vertical().id_salt("classic-print").auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let layout = app.ui.classic.print.clone();
            let canvas = layout.canvas;

            title(ui, "printLayout", "Layout");
            padded(ui, |ui| {
                let shapes: Vec<(&str, &str)> = PAGE_SHAPES.iter().map(|(key, label, ..)| (*label, *key)).collect();
                let active = layout.shape().and_then(|key| PAGE_SHAPES.iter().position(|(k, ..)| *k == key));
                if let Some((key, ..)) = segmented(ui, "printShape", &shapes, active, 3).and_then(|i| PAGE_SHAPES.get(i)) {
                    set(app, ui.ctx(), json!({"shape": key}));
                }
                ui.add_space(8.0);
                ui.label(egui::RichText::new(format!("{} × {} px", canvas.width, canvas.height)).color(t.text_dim).size(12.0));
            });
            let longest = (1000.0, CANVAS_EDGE_RANGE.1.min(12_000) as f64);
            if let Some(v) = number(ui, "print.longEdge", "Long Edge", longest, 100.0, layout.long_edge() as f64) {
                set(app, ui.ctx(), json!({"longEdge": (v / 100.0).round() * 100.0}));
            }

            title(ui, "printPhoto", "Photo");
            ui.add_space(6.0);
            if let Some(v) = number(ui, "print.size", "Size", crate::classic::print::size_percent_range(), 1.0, (canvas.size * 100.0).round() as f64)
            {
                set(app, ui.ctx(), json!({"size": v / 100.0}));
            }
            if let Some(v) = number(ui, "print.x", "Across", (0.0, 100.0), 1.0, (canvas.x * 100.0).round() as f64) {
                set(app, ui.ctx(), json!({"x": v / 100.0}));
            }
            if let Some(v) = number(ui, "print.y", "Down", (0.0, 100.0), 1.0, (canvas.y * 100.0).round() as f64) {
                set(app, ui.ctx(), json!({"y": v / 100.0}));
            }
            padded(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    if text_button(ui, "printCenter", crate::i18n::tr("Center"), canvas.x == 0.5 && canvas.y == 0.5).clicked() {
                        set(app, ui.ctx(), json!({"x": 0.5, "y": 0.5}));
                    }
                    if text_button(ui, "printCenterAcross", crate::i18n::tr("Across"), canvas.x == 0.5).clicked() {
                        set(app, ui.ctx(), json!({"x": 0.5}));
                    }
                    if text_button(ui, "printCenterDown", crate::i18n::tr("Down"), canvas.y == 0.5).clicked() {
                        set(app, ui.ctx(), json!({"y": 0.5}));
                    }
                });
            });

            title(ui, "printPage", "Page");
            padded(ui, |ui| {
                let colours: Vec<(&str, &str)> = PAGE_COLOURS.iter().map(|(key, label, _)| (*label, *key)).collect();
                let active = PAGE_COLOURS.iter().position(|(_, _, rgb)| *rgb == canvas.background);
                if let Some((key, ..)) = segmented(ui, "printBackground", &colours, active, 4).and_then(|i| PAGE_COLOURS.get(i)) {
                    set(app, ui.ctx(), json!({"background": key}));
                }
            });

            title(ui, "printJob", "Print Job");
            padded(ui, |ui| {
                let formats: Vec<(&str, &str)> = PrintFormat::ALL.iter().map(|f| (f.label(), f.id())).collect();
                let active = PrintFormat::ALL.iter().position(|f| *f == layout.format);
                if let Some(format) = segmented(ui, "printFormat", &formats, active, 3).and_then(|i| PrintFormat::ALL.get(i)) {
                    set(app, ui.ctx(), json!({"format": format.id()}));
                }
            });
            if layout.format == PrintFormat::Jpeg
                && let Some(v) = number(ui, "print.quality", "Quality", (1.0, 100.0), 1.0, layout.quality as f64)
            {
                set(app, ui.ctx(), json!({"quality": v.round()}));
            }
            padded(ui, |ui| {
                let folder = crate::classic::print::folder(app);
                ui.label(egui::RichText::new(crate::i18n::tr("Folder")).color(t.text_dim).size(12.0));
                let shown = if folder.is_empty() { crate::i18n::tr("None chosen").to_string() } else { folder.clone() };
                let r = ui.add(egui::Label::new(egui::RichText::new(shown).color(t.text_label).size(12.0)).truncate());
                register(ui.ctx(), "printFolder", r.rect);
                r.on_hover_text(folder);
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    if app.services.pick_folder.is_some()
                        && text_button(ui, "printChoose", crate::i18n::tr("Choose…"), false).clicked()
                        && let Some(dir) = app.services.pick_folder.as_mut().and_then(|pick| pick())
                    {
                        set(app, ui.ctx(), json!({"dir": dir}));
                    }
                    let n = app.session.selection.ids.len().max(usize::from(app.session.active().is_some()));
                    if text_button(ui, "printToFile", crate::i18n::tr("Print to File"), false).clicked() {
                        match app.run("print.export", json!({"dir": crate::classic::print::folder(app)})) {
                            Ok(_) => app.toast(
                                ui.ctx(),
                                if n > 1 {
                                    crate::i18n::tr("Printing the pages to files…")
                                } else {
                                    crate::i18n::tr("Printing the page to a file…")
                                },
                            ),
                            Err(e) => app.toast(ui.ctx(), e),
                        }
                    }
                });
            });
            ui.add_space(30.0);
        });
    });
}
