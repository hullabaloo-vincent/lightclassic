//! Lightroom Classic's Develop panels, in the right column of the Classic interface style: Histogram, the tools, then
//! Basic, Tone Curve, HSL / Color, Color Grading, Detail, Lens Corrections, Transform, Effects and Calibration.
//!
//! Every control here is the Edit panel's own: the same develop control ids, commands and widgets, arranged as
//! Classic arranges them. This is a child module of `edit` so that it can use that panel's private pieces (the
//! histogram, the curve editor, the mixers, the grading wheels) as they are. What the Edit panel draws inline in its
//! `show` (the Auto row, the profile row, white balance, the vignette style, the optics checkboxes) is drawn again
//! here, marked "as the Edit panel's": keep the two alike when either changes. `panels::classic::tests` checks
//! that no slider of the Edit panel is missing from these panels.

use super::*;
use crate::classic::DevelopPanel;

/// Height of a panel's header.
const HEADER_H: f32 = 32.0;

fn margin(top: i8, bottom: i8) -> egui::Margin {
    egui::Margin { left: 24, right: 22, top, bottom }
}

/// The Develop column: one scroll of the histogram, the tools (with the open tool's own panel under them) and the
/// develop panels.
pub(in crate::panels) fn show(app: &mut LightcraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Frame::NONE.fill(t.chrome).stroke(Stroke::new(1.0, t.divider)).show(ui, |ui| {
        let Some(id) = app.session.active() else {
            let r = ui.max_rect();
            crate::panels::empty_message(ui, r, "No photo selected", "Select a photo to edit");
            return;
        };
        egui::ScrollArea::vertical().id_salt("classic-develop").auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            histogram_panel(app, ui, id);
            crate::panels::classic::tool_strip(app, ui, &t);
            tool_drawer(app, ui, id);
            notices(app, ui, id);
            let (d, raw) = shown_settings(app, id);
            for panel in DevelopPanel::ALL {
                let open = app.ui.section_open(panel.open_id());
                let off: Vec<&str> = panel.sections().iter().copied().filter(|s| !d.section_enabled(s)).collect();
                // its own switch, or (for a panel without one) the way back from a section the Edit panel turned off
                let switch = if panel.has_switch() || !off.is_empty() { Some(off.is_empty()) } else { None };
                let (resp, toggled) = panel_header(ui, panel.id(), panel.title(), open, switch);
                if let Some(on) = toggled {
                    let sections = if on { off } else { panel.sections().to_vec() };
                    for section in sections {
                        let _ = app.run("develop.sectionEnabled", json!({"section": section, "enabled": on}));
                    }
                } else if resp.clicked() {
                    let _ = app.run("view.classicPanel", json!({"panel": panel.id()}));
                }
                if open {
                    body(app, ui, panel, &d, id, raw);
                    ui.add_space(10.0);
                }
            }
            ui.add_space(40.0);
        });
    });
}

fn body(app: &mut LightcraftApp, ui: &mut egui::Ui, panel: DevelopPanel, d: &DevelopSettings, id: PhotoId, raw: bool) {
    match panel {
        DevelopPanel::Basic => basic(app, ui, d, id, raw),
        DevelopPanel::ToneCurve => curve_editor(app, ui, id, d),
        DevelopPanel::Hsl => hsl(app, ui, d),
        DevelopPanel::ColorGrading => grading(app, ui, d, id),
        DevelopPanel::Detail => detail(app, ui, d, id),
        DevelopPanel::LensCorrections => lens_corrections(app, ui, d, id),
        DevelopPanel::Transform => crate::panels::right::geometry(app, ui, id),
        DevelopPanel::Effects => effects(app, ui, d),
        DevelopPanel::Calibration => calibration(app, ui, d),
    }
}

/// A panel's header (`classicPanel:{id}`): the title at the right beside the triangle that folds the panel, and at
/// the left the switch that turns the panel's adjustments off (`classicSwitch:{id}`) where there is one. Returns the
/// header's response (a click folds) and the switch's new state when it was clicked.
fn panel_header(ui: &mut egui::Ui, id: &str, title: &str, open: bool, switch: Option<bool>) -> (egui::Response, Option<bool>) {
    let title = crate::i18n::tr(title);
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), HEADER_H), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::CollapsingHeader, true, open, title));
    register(ui.ctx(), format!("classicPanel:{id}"), r);
    let p = ui.painter();
    p.rect_filled(r, 0.0, if ui.rect_contains_pointer(r) { t.pressed } else { t.hover });
    p.hline(r.x_range(), r.bottom() - 0.5, Stroke::new(1.0, t.divider));
    let fold = Rect::from_center_size(pos2(r.right() - 20.0, r.center().y), vec2(12.0, 12.0));
    paint(p, fold, if open { Icon::ChevronDown } else { Icon::ChevronLeft }, t.text_label);
    p.text(pos2(r.right() - 36.0, r.center().y), Align2::RIGHT_CENTER, title, t.semibold(13.0), t.text);
    let mut toggled = None;
    if let Some(on) = switch {
        let eye = Rect::from_center_size(pos2(r.left() + 22.0, r.center().y), vec2(16.0, 16.0));
        let switched = ui.interact(eye, ui.id().with(("classicSwitch", id)), Sense::click());
        switched.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, on, title));
        register(ui.ctx(), format!("classicSwitch:{id}"), eye);
        paint(ui.painter(), eye, Icon::Eye, if on { t.icon } else { t.text_disabled });
        if !on {
            ui.painter().line_segment([eye.left_bottom(), eye.right_top()], Stroke::new(1.5, t.text_disabled));
        }
        if switched.clicked() {
            toggled = Some(!on);
        }
    }
    (resp, toggled)
}

/// The Histogram panel: its header folds it (`view.histogram`), and soft proofing sits under it as in the Edit panel.
fn histogram_panel(app: &mut LightcraftApp, ui: &mut egui::Ui, id: PhotoId) {
    let open = app.ui.histogram;
    let (resp, _) = panel_header(ui, "histogram", "Histogram", open, None);
    if resp.clicked() {
        let _ = app.run("view.classicPanel", json!({"panel": "histogram"}));
    }
    if open {
        histogram(app, ui, id);
    }
    if app.ui.soft_proof {
        soft_proofing(app, ui, id);
    }
}

/// The open tool's own panel, between the tool strip and the develop panels (which stay below it).
fn tool_drawer(app: &mut LightcraftApp, ui: &mut egui::Ui, id: PhotoId) {
    use crate::panels::{masking, right};
    match app.ui.right {
        RightPanel::Crop => right::crop_tool(app, ui, id),
        RightPanel::Remove => right::remove(app, ui, id),
        RightPanel::RedEye => right::red_eye(app, ui, id),
        RightPanel::Masking => masking::show(app, ui, id),
        RightPanel::None
        | RightPanel::Edit
        | RightPanel::Profiles
        | RightPanel::Versions
        | RightPanel::Activity
        | RightPanel::Keywords
        | RightPanel::Info => {
            return;
        }
    }
    divider(ui);
}

/// What the Edit panel says above its sections: a raw shown from its embedded preview, and Auto Sync.
fn notices(app: &mut LightcraftApp, ui: &mut egui::Ui, id: PhotoId) {
    let t = Tokens::get(ui.ctx());
    // as the Edit panel's
    if let Some(why) = app.session.catalog.photo(id).and_then(|p| p.preview_only.clone()) {
        egui::Frame::NONE.inner_margin(margin(10, 10)).show(ui, |ui| {
            crate::widgets::preview_only_notice(ui, "edit", &why);
        });
    }
    let n = app.session.selection.ids.len();
    if app.session.auto_sync && n > 1 {
        egui::Frame::NONE.inner_margin(margin(10, 10)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(crate::i18n::tr_format!("Auto Sync: edits apply to {n} photos", n = n)).color(t.accent).size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if text_button(ui, "autoSyncOff", crate::i18n::tr("Turn Off"), false).clicked() {
                        let _ = app.run("develop.autoSync", json!({"on": false}));
                    }
                });
            });
        });
    }
}

/// Basic: treatment and Auto, the profile, white balance, tone and presence.
fn basic(app: &mut LightcraftApp, ui: &mut egui::Ui, d: &DevelopSettings, id: PhotoId, raw: bool) {
    let t = Tokens::get(ui.ctx());
    // Auto / B&W / HDR and Reset: as the Edit panel's header row
    let mut reset_clicked = false;
    egui::Frame::NONE.inner_margin(margin(12, 8)).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if text_button(ui, "auto", crate::i18n::tr("Auto"), false).clicked() {
                let _ = app.run("develop.auto", json!({}));
                app.toast(ui.ctx(), crate::i18n::tr("Auto settings applied"));
            }
            if text_button(ui, "bw", crate::i18n::tr("B&W"), crate::is_bw(d)).clicked() {
                let _ = app.run("develop.treatment", json!({}));
            }
            if text_button(ui, "hdr", crate::i18n::tr("HDR"), d.hdr.enabled)
                .on_hover_text(crate::i18n::tr("Edit in HDR: highlights above SDR white, for HDR exports and displays"))
                .clicked()
            {
                let _ = app.run("develop.hdr", json!({"enabled": !d.hdr.enabled}));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if text_button(ui, "reset", crate::i18n::tr("Reset"), false).on_hover_text(crate::i18n::tr("Reset all edits (Cmd+Shift+R)")).clicked()
                {
                    reset_clicked = true;
                }
            });
        });
    });
    // the profile row: as the Edit panel's
    egui::Frame::NONE.inner_margin(margin(6, 10)).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(crate::i18n::tr("Profile")).font(t.font(13.0)).color(t.text_dim));
            let name = app.session.profile_info(&d.profile.id).map(|(name, _)| name).unwrap_or("Color");
            let r = crate::widgets::dropdown(ui, "profile", crate::i18n::profile_label(&d.profile.id, name), t.font(15.0), t.text_label);
            egui::Popup::menu(&r).show(|ui| profile_menu(app, ui, d));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::widgets::icon_button(
                    ui,
                    "profileBrowser",
                    Icon::ProfileGrid,
                    vec2(28.0, 28.0),
                    false,
                    true,
                    crate::i18n::tr("Browse Profiles"),
                )
                .clicked()
                {
                    let _ = app.run("panel.profiles", json!({}));
                }
            });
        });
    });
    if d.profile.id != "lc.color" {
        control(app, ui, d, "profile.amount", true);
        ui.add_space(6.0);
    }
    if app.session.source_info(id).local_tone.is_some() {
        egui::Frame::NONE.inner_margin(margin(0, 10)).show(ui, |ui| {
            let mut on = d.profile.camera_local_tone;
            let r = ui.checkbox(&mut on, crate::i18n::tr("Camera local tone mapping"));
            register(ui.ctx(), "check:profile.cameraLocalTone", r.rect);
            if r.on_hover_text(crate::i18n::tr("Apply the local tone mapping stored in the raw (Apple ProRAW), as the camera renders it")).changed() {
                let _ = app.run("develop.merge", json!({"settings": {"profile": {"camera_local_tone": on}}, "label": "Camera Local Tone Mapping"}));
            }
        });
    }
    divider(ui);
    // white balance: as the Edit panel's Color section
    egui::Frame::NONE.inner_margin(margin(8, 2)).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(crate::i18n::tr("White Balance")).font(t.font(13.0)).color(t.text_dim));
            let r = crate::widgets::dropdown(ui, "wbMode", crate::i18n::tr(d.wb.mode.label()), t.font(14.0), t.text_label);
            egui::Popup::menu(&r).show(|ui| {
                for m in WbMode::ALL {
                    if m == WbMode::Custom {
                        continue;
                    }
                    if ui.selectable_label(d.wb.mode == m, m.label()).clicked() {
                        let mode = serde_json::to_value(m).unwrap_or_default();
                        let _ = app.run("develop.wb", json!({"mode": mode}));
                    }
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let active = app.ui.tool == "wbPicker";
                if crate::widgets::icon_button(
                    ui,
                    "wbPicker",
                    Icon::Picker,
                    vec2(28.0, 28.0),
                    active,
                    true,
                    crate::i18n::tr("White Balance Selector (W)"),
                )
                .clicked()
                {
                    if active {
                        app.ui.tool.clear();
                    } else if let Err(e) = app.run("tool.wbPicker", json!({})) {
                        app.toast(ui.ctx(), e);
                    }
                }
            });
        });
    });
    if raw {
        control(app, ui, d, "wb.temp", true);
        control(app, ui, d, "wb.tint", true);
    } else {
        let out = slider(ui, &REL_TEMP, k_to_rel(d.wb.temp), true, None);
        apply_slider_out(app, &REL_TEMP, out, |app, v| app.run("develop.set", json!({"control": "wb.temp", "value": rel_to_k(v)})));
        let out = slider(ui, &REL_TINT, d.wb.tint.clamp(-100.0, 100.0), true, None);
        apply_slider_out(app, &REL_TINT, out, |app, v| app.run("develop.set", json!({"control": "wb.tint", "value": v})));
    }
    sub_title(ui, crate::i18n::tr("Tone"));
    hdr_row(app, ui, d);
    for c in ["light.exposure", "light.contrast", "light.highlights", "light.shadows", "light.whites", "light.blacks"] {
        control(app, ui, d, c, true);
    }
    if d.hdr.enabled {
        ui.add_space(6.0);
        let open = app.ui.flyout_open("sdrRendition");
        if flyout_row(ui, "sdrRendition", "SDR Rendition", Icon::Sliders, open).clicked() {
            app.ui.toggle_flyout("sdrRendition");
        }
        if open {
            for c in ["hdr.sdrBrightness", "hdr.sdrContrast", "hdr.sdrHighlights", "hdr.sdrShadows", "hdr.sdrWhites", "hdr.sdrClarity"] {
                control(app, ui, d, c, true);
            }
        }
    }
    sub_title(ui, crate::i18n::tr("Presence"));
    for c in ["effects.texture", "effects.clarity", "effects.dehaze", "color.vibrance", "color.saturation"] {
        control(app, ui, d, c, true);
    }
    // A pending numeric edit commits on blur before the explicit Reset takes precedence.
    if reset_clicked {
        let _ = app.run("develop.reset", json!({}));
    }
}

/// HSL / Color: the colour (or black-and-white) mixer, and Point Color under it.
fn hsl(app: &mut LightcraftApp, ui: &mut egui::Ui, d: &DevelopSettings) {
    mixer(app, ui, d);
    if !crate::is_bw(d) {
        ui.add_space(6.0);
        let open = app.ui.flyout_open("pointColor");
        if flyout_row(ui, "pointColor", crate::i18n::tr("Point Color"), Icon::Picker, open).clicked() {
            app.ui.toggle_flyout("pointColor");
        }
        if open {
            point_color(app, ui, d);
        }
    }
}

/// Detail: sharpening, AI Denoise and noise reduction (as the Edit panel's Detail section).
fn detail(app: &mut LightcraftApp, ui: &mut egui::Ui, d: &DevelopSettings, id: PhotoId) {
    use crate::panels::denoise;
    sub_title(ui, crate::i18n::tr("Sharpening"));
    for c in ["detail.sharpenAmount", "detail.sharpenRadius", "detail.sharpenDetail", "detail.sharpenMasking"] {
        control(app, ui, d, c, c == "detail.sharpenAmount" || d.detail.sharpen_amount > 0.0);
    }
    sub_title(ui, crate::i18n::tr("AI Denoise"));
    let applicable = !matches!(app.session.denoise_photo_state(id), lightcraft_engine::denoise::PhotoState::NotApplicable);
    let on = d.enhance.denoise_enabled();
    denoise::detail_toggle(app, ui, id, on, applicable);
    control(app, ui, d, "enhance.denoise", applicable && on);
    denoise::detail_status(app, ui, id, if on { d.enhance.denoise } else { 0.0 });
    sub_title(ui, crate::i18n::tr("Noise Reduction"));
    for c in ["detail.nrLuminance", "detail.nrDetail", "detail.nrContrast", "detail.nrColor", "detail.nrColorDetail", "detail.nrColorSmoothness"] {
        control(app, ui, d, c, true);
    }
}

/// Lens Corrections: the profile, manual distortion and vignetting, defringe and lateral CA (as the Edit panel's
/// Optics section).
fn lens_corrections(app: &mut LightcraftApp, ui: &mut egui::Ui, d: &DevelopSettings, id: PhotoId) {
    let has_lens = app.session.catalog.photo(id).is_some_and(|p| p.embedded_lens.is_some());
    egui::Frame::NONE.inner_margin(margin(8, 4)).show(ui, |ui| {
        let mut ca = d.optics.remove_ca;
        if ui.checkbox(&mut ca, crate::i18n::tr("Remove Chromatic Aberration")).changed() {
            let _ = app.run("develop.merge", json!({"settings": {"optics": {"remove_ca": ca}}, "label": "Remove CA"}));
        }
        let mut lp = d.optics.lens_profile;
        if ui.checkbox(&mut lp, crate::i18n::tr("Enable Lens Corrections")).changed() {
            let _ = app.run("develop.merge", json!({"settings": {"optics": {"lens_profile": lp}}, "label": "Lens Corrections"}));
        }
        if lp && !has_lens {
            ui.label(
                egui::RichText::new(crate::i18n::tr("No lens data embedded in this file (DNG lens opcodes only)."))
                    .size(11.0)
                    .color(Tokens::get(ui.ctx()).text_dim),
            );
        }
    });
    if d.optics.lens_profile {
        for c in ["optics.profileDistortion", "optics.profileVignetting"] {
            control(app, ui, d, c, has_lens);
        }
    }
    sub_title(ui, crate::i18n::tr("Manual"));
    for c in ["optics.distortion", "optics.vignetting"] {
        control(app, ui, d, c, true);
    }
    control(app, ui, d, "optics.vignettingMidpoint", d.optics.vignetting != 0.0);
    ui.add_space(6.0);
    let open = app.ui.flyout_open("defringe");
    if flyout_row(ui, "defringe", crate::i18n::tr("Defringe"), Icon::Picker, open).clicked() {
        app.ui.toggle_flyout("defringe");
    }
    if open {
        control(app, ui, d, "optics.defringePurple", true);
        let on = d.optics.defringe_purple_amount > 0.0;
        control(app, ui, d, "optics.defringePurpleHueLo", on);
        control(app, ui, d, "optics.defringePurpleHueHi", on);
        control(app, ui, d, "optics.defringeGreen", true);
        let on = d.optics.defringe_green_amount > 0.0;
        control(app, ui, d, "optics.defringeGreenHueLo", on);
        control(app, ui, d, "optics.defringeGreenHueHi", on);
        sub_title(ui, crate::i18n::tr("Lateral Chromatic Aberration"));
        control(app, ui, d, "optics.caRed", true);
        control(app, ui, d, "optics.caBlue", true);
    }
}

/// Effects: the post-crop vignette and grain (the Edit panel's Effects section without Texture, Clarity and Dehaze,
/// which Classic keeps in Basic).
fn effects(app: &mut LightcraftApp, ui: &mut egui::Ui, d: &DevelopSettings) {
    sub_title(ui, crate::i18n::tr("Vignette"));
    {
        use lightcraft_develop::VignetteStyle as V;
        let styles = [
            (V::HighlightPriority, "Highlight", "highlightPriority"),
            (V::ColorPriority, "Color", "colorPriority"),
            (V::PaintOverlay, "Paint", "paintOverlay"),
        ];
        let items: Vec<(&str, &str)> = styles.iter().map(|(_, l, k)| (*l, *k)).collect();
        let active = styles.iter().position(|(v, _, _)| *v == d.vignette.style);
        let mut chosen = None;
        egui::Frame::NONE.inner_margin(margin(2, 4)).show(ui, |ui| {
            chosen = crate::widgets::segmented(ui, "vignetteStyle", &items, active, 3);
        });
        if let Some((style, _, _)) = chosen.and_then(|i| styles.get(i)) {
            let _ = app.run("develop.merge", json!({"settings": {"vignette": {"style": serde_json::to_value(style).unwrap_or_default()}}}));
        }
    }
    for c in ["vignette.amount", "vignette.midpoint", "vignette.feather", "vignette.roundness", "vignette.highlights"] {
        control(app, ui, d, c, c == "vignette.amount" || d.vignette.amount != 0.0);
    }
    sub_title(ui, crate::i18n::tr("Grain"));
    for c in ["grain.amount", "grain.size", "grain.roughness"] {
        control(app, ui, d, c, c == "grain.amount" || d.grain.amount != 0.0);
    }
}

/// Calibration: the shadow tint and the three primaries (as the Edit panel's Calibration section).
fn calibration(app: &mut LightcraftApp, ui: &mut egui::Ui, d: &DevelopSettings) {
    sub_title(ui, crate::i18n::tr("Shadows"));
    control(app, ui, d, "calibration.shadowsTint", true);
    for (title, k) in [("Red Primary", "red"), ("Green Primary", "green"), ("Blue Primary", "blue")] {
        sub_title(ui, title);
        control(app, ui, d, &format!("calibration.{k}Hue"), true);
        control(app, ui, d, &format!("calibration.{k}Sat"), true);
    }
}
