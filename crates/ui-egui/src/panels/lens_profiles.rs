//! Lens profiles in the window: what the Lens Corrections panel says about a photo's lens data, the offer to
//! download the lens database, and the upkeep that gives photos already in the library their profile.
//!
//! The database is the lensfun project's (CC BY-SA 3.0), fetched by `optics.downloadLensProfiles` into the
//! settings folder; the engine side is `lightcraft_engine::lens_profiles`. Both interface styles draw the same note.

use egui::RichText;
use lightcraft_catalog::PhotoId;
use lightcraft_engine::lens_profiles::{self as store, DownloadOutcome};
use serde_json::json;

use crate::LightcraftApp;
use crate::i18n::tr;
use crate::theme::Tokens;

/// The photos whose files were read for this database: (photo, database generation).
#[derive(Clone, Default)]
struct Looked(std::collections::HashSet<(u64, u64)>);

fn line(ui: &mut egui::Ui, text: impl Into<String>) {
    let dim = Tokens::get(ui.ctx()).text_dim;
    ui.add(egui::Label::new(RichText::new(text).size(11.0).color(dim)).wrap());
}

/// Under "Enable Lens Corrections": which lens the photo's lens data is for and what it corrects, or why there is
/// none and what to do about it. `on`: the correction is switched on.
pub(crate) fn note(app: &mut LightcraftApp, ui: &mut egui::Ui, id: PhotoId, on: bool) {
    let Some(p) = app.session.catalog.photo(id) else { return };
    let lens_name = p.meta.lens.trim().to_string();
    if let Some(lens) = &p.embedded_lens {
        let c = store::corrects(lens);
        let parts: Vec<&str> = [(c.distortion, "distortion"), (c.fringing, "colour fringing"), (c.vignetting, "vignetting")]
            .into_iter()
            .filter(|(has, _)| *has)
            .map(|(_, what)| tr(what))
            .collect();
        if !lens_name.is_empty() {
            line(ui, lens_name);
        }
        if !parts.is_empty() {
            line(ui, format!("{} {}", tr("Lens profile corrects:"), parts.join(", ")));
        }
        return;
    }
    if !p.develops_raw() {
        if on {
            line(ui, tr("Lens profiles apply to raw photos; this file was processed in the camera."));
        }
        return;
    }
    if app.session.lens_profiles.downloading() {
        line(ui, tr("Downloading lens profiles…"));
        return;
    }
    if store::database().is_none() {
        offer(app, ui);
        return;
    }
    // the database is there: read this photo's file once, in case it was catalogued before the database came
    let key = (id.0, store::generation());
    let dom = egui::Id::new("lens-profiles-looked");
    let looked = ui.ctx().data(|d| d.get_temp::<Looked>(dom)).is_some_and(|l| l.0.contains(&key));
    if !looked {
        ui.ctx().data_mut(|d| d.get_temp_mut_or_default::<Looked>(dom).0.insert(key));
        if app.session.execute("optics.findLensProfiles", &json!({"ids": [id.0]})).is_ok_and(|r| r["changed"].as_u64().unwrap_or(0) > 0) {
            ui.ctx().request_repaint();
            return;
        }
    }
    if lens_name.is_empty() {
        line(ui, tr("This photo does not name its lens, so no lens profile can be picked."));
    } else {
        line(ui, format!("{} {lens_name}", tr("The lens database has no profile for")));
    }
}

/// Whether the photo's lens data holds what the amount slider `control` scales: a profile with distortion only
/// (as most of the database's zoom lenses have) leaves Profile Vignetting with nothing to scale.
pub(crate) fn scales(app: &LightcraftApp, id: PhotoId, control: &str) -> bool {
    let Some(c) = app.session.catalog.photo(id).and_then(|p| p.embedded_lens.as_ref().map(store::corrects)) else { return false };
    if control == "optics.profileVignetting" { c.vignetting } else { c.distortion || c.fringing }
}

/// No lens database: say where profiles come from and offer the download.
fn offer(app: &mut LightcraftApp, ui: &mut egui::Ui) {
    if store::env_dir().is_some() {
        line(ui, tr("No lens profiles were found in the LIGHTCRAFT_LENSFUN_DIR folder."));
        return;
    }
    line(ui, tr("No lens profile for this photo. Lens profiles are a separate download: the lensfun database (CC BY-SA), about 5 MB."));
    // (the web app has no folder to keep them in and no download command)
    if store::host_dir().is_none() || cfg!(target_arch = "wasm32") {
        return;
    }
    ui.add_space(2.0);
    let r = ui.button(tr("Download Lens Profiles"));
    crate::widgets::register(ui.ctx(), "button:lensProfilesDownload", r.rect);
    if r.clicked()
        && let Err(e) = app.run("optics.downloadLensProfiles", json!({}))
    {
        app.toast_error(ui.ctx(), e);
    }
}

/// Once a frame: say how a download ended, record what a background lookup found, and start the lookup that gives
/// the library's photos their profiles whenever the library or the database is new to this window.
pub(crate) fn pump(app: &mut LightcraftApp, ctx: &egui::Context) {
    match app.session.lens_profiles.take_outcome() {
        Some(DownloadOutcome::Done { lenses, .. }) => app.toast(ctx, format!("{} {lenses}", tr("Lens profiles installed:"))),
        Some(DownloadOutcome::Cancelled) => app.toast(ctx, tr("Lens profile download cancelled")),
        Some(DownloadOutcome::Failed(e)) => app.toast_error(ctx, e),
        None => {}
    }
    if let Some(gained) = app.session.finish_lens_scan()
        && gained > 0
    {
        app.toast(ctx, format!("{} {gained}", tr("Lens profiles applied to photos:")));
    }
    if app.session.lens_profiles.downloading() || app.session.lens_profiles.scanning() {
        ctx.request_repaint_after(std::time::Duration::from_millis(300));
        return;
    }
    let key = (app.session.library_generation(), store::generation());
    let dom = egui::Id::new("lens-profiles-caught-up");
    if ctx.data(|d| d.get_temp::<(u64, u64)>(dom)) == Some(key) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(dom, key));
    if store::folder_exists()
        && app.session.execute("optics.findLensProfiles", &json!({"all": true, "background": true})).is_ok_and(|r| r["started"] == true)
    {
        ctx.request_repaint_after(std::time::Duration::from_millis(300));
    }
}

#[cfg(test)]
#[path = "lens_profiles_tests.rs"]
mod tests;
