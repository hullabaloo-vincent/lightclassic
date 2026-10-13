//! The Classic frame, headless: the module picker, the columns, the filmstrip across the window, and the Modern
//! frame left as it was.

use std::time::Duration;

use serde_json::json;

use crate::classic::Module;
use crate::headless::Headless;
use crate::state::{RightPanel, ViewMode};
use crate::{LightcraftApp, Services};

const T: Duration = Duration::from_secs(20);
const SETTLE: Duration = Duration::from_secs(120);
const SIZE: [f32; 2] = [1600.0, 1000.0];

fn app(classic: bool) -> LightcraftApp {
    let services = Services { png: None, ..Default::default() };
    let mut app = LightcraftApp::new(lightcraft_engine::Session::with_demo(), services);
    app.ui.settings.gpu = false;
    app.ui.left_panel = true;
    app.ui.filmstrip = true;
    app.ui.view = ViewMode::PhotoGrid;
    app.ui.right = RightPanel::None;
    if classic {
        app.run("view.interface", json!({"style": "classic"})).expect("the style is set");
    }
    app
}

fn frame(classic: bool) -> Headless {
    let mut h = Headless::new(app(classic), SIZE, 1.0);
    assert!(h.settle(SETTLE), "the window never comes to rest");
    h
}

fn widget(h: &Headless, id: &str) -> Option<egui::Rect> {
    h.app.widgets.iter().find(|(w, _)| w == id).map(|(_, r)| *r)
}

fn rect(h: &Headless, id: &str) -> egui::Rect {
    widget(h, id).unwrap_or_else(|| panic!("no {id} on screen"))
}

fn click(h: &mut Headless, id: &str) {
    let r = h.request("ui.clickWidget", json!({"id": id}), T);
    assert_eq!(r["ok"], true, "{id}: {r}");
    h.settle(SETTLE);
}

#[test]
fn the_classic_frame_has_a_module_picker_columns_and_a_filmstrip_across_the_window() {
    let mut h = frame(true);
    assert_eq!(h.app.ui.classic.module, Module::Library);

    // the module picker: Library | Develop at the right, under the top bar
    let (library, develop) = (rect(&h, "module:library"), rect(&h, "module:develop"));
    assert!(library.right() <= develop.left() + 1.5, "Library {library:?} is left of Develop {develop:?}");
    assert!(develop.right() > SIZE[0] - 40.0, "the picker sits at the right: {develop:?}");
    assert!(widget(&h, "icon:edit").is_none(), "the Modern tool strip is drawn in the Classic frame");

    // the filmstrip runs under both columns, the whole width of the window
    let film = rect(&h, "panel:filmstrip");
    assert!(film.left() < 0.5 && film.right() > SIZE[0] - 0.5 && film.bottom() > SIZE[1] - 0.5, "{film:?}");
    let left = rect(&h, "panel:left_panel");
    assert!(left.bottom() <= film.top() + 0.5 && left.top() >= library.bottom() - 0.5, "left column {left:?}");
    let cells: Vec<egui::Rect> = h.app.widgets.iter().filter(|(w, _)| w.starts_with("film:")).map(|(_, r)| *r).collect();
    assert!(!cells.is_empty(), "the filmstrip has no photos");
    assert!(cells.iter().all(|c| c.top() >= film.top() - 0.5), "a filmstrip cell outside the strip: {cells:?}");

    // the Library shows no develop panels
    assert!(widget(&h, "slider:light.exposure").is_none());
    assert!(widget(&h, "icon:crop").is_none());

    // Develop: the photo, its panels on the right with the tools above them, and still one filmstrip
    click(&mut h, "module:develop");
    assert_eq!((h.app.ui.classic.module, h.app.ui.view), (Module::Develop, ViewMode::Detail));
    let right = rect(&h, "panel:right_panel");
    assert!(right.right() > SIZE[0] - 0.5 && right.bottom() <= film.top() + 0.5, "right column {right:?}");
    let exposure = rect(&h, "slider:light.exposure");
    assert!(right.contains_rect(exposure), "Exposure {exposure:?} outside the right column {right:?}");
    for tool in ["icon:crop", "icon:remove", "icon:redeye", "icon:masking"] {
        let r = rect(&h, tool);
        assert!(right.contains_rect(r) && r.bottom() <= exposure.top(), "{tool} {r:?} is above the panels in the right column");
    }
    let mut cells: Vec<String> = h.app.widgets.iter().filter(|(w, _)| w.starts_with("film:")).map(|(w, _)| w.clone()).collect();
    let drawn = cells.len();
    cells.sort();
    cells.dedup();
    assert_eq!(cells.len(), drawn, "the loupe drew a second filmstrip inside the photo area");
    assert!(h.app.ui.filmstrip, "the filmstrip setting was lost while the loupe was drawn");

    // a tool opens in place of the panels and its button puts it away again
    click(&mut h, "icon:crop");
    assert_eq!((h.app.ui.right, h.app.ui.classic.module), (RightPanel::Crop, Module::Develop));
    click(&mut h, "icon:crop");
    assert_eq!((h.app.ui.right, h.app.ui.classic.module), (RightPanel::Edit, Module::Develop));

    // and back
    click(&mut h, "module:library");
    assert_eq!((h.app.ui.classic.module, h.app.ui.view), (Module::Library, ViewMode::PhotoGrid));
    assert!(widget(&h, "panel:right_panel").is_none(), "the Library has no panel open on the right");
}

#[test]
fn the_filmstrip_and_the_left_column_can_be_hidden() {
    let mut h = frame(true);
    let r = h.request("engine.execute", json!({"command": "view.filmstrip", "params": {}}), T);
    assert_eq!(r["ok"], true, "{r}");
    h.settle(SETTLE);
    assert!(!h.app.ui.filmstrip);
    assert!(widget(&h, "panel:filmstrip").is_none());
    assert!(!h.app.widgets.iter().any(|(w, _)| w.starts_with("film:")), "filmstrip cells drawn while it is hidden");
    let r = h.request("engine.execute", json!({"command": "view.leftPanel", "params": {}}), T);
    assert_eq!(r["ok"], true, "{r}");
    h.settle(SETTLE);
    assert!(widget(&h, "panel:left_panel").is_none());
}

#[test]
fn the_modern_frame_is_as_it_was() {
    let mut h = frame(false);
    assert!(widget(&h, "module:library").is_none() && widget(&h, "module:develop").is_none());
    assert!(widget(&h, "panel:filmstrip").is_none());
    assert!(widget(&h, "icon:edit").is_some(), "the tool strip");
    // switching styles redraws the other frame at once, both ways
    let r = h.request("engine.execute", json!({"command": "view.interface", "params": {"style": "classic"}}), T);
    assert_eq!(r["ok"], true, "{r}");
    h.settle(SETTLE);
    assert!(widget(&h, "module:library").is_some() && widget(&h, "icon:edit").is_none());
    let r = h.request("engine.execute", json!({"command": "view.interface", "params": {"style": "modern"}}), T);
    assert_eq!(r["ok"], true, "{r}");
    h.settle(SETTLE);
    assert!(widget(&h, "module:library").is_none() && widget(&h, "icon:edit").is_some());
}

/// A photo has one thumbnail. With a grid and the filmstrip on screen together, each asking for its own size had the
/// other's rendered again on every frame, and the window never came to rest.
#[test]
fn the_grid_and_the_filmstrip_share_their_thumbnails() {
    let mut h = frame(true);
    assert!(h.app.widgets.iter().any(|(w, _)| w.starts_with("film:")), "no filmstrip beside the grid");
    let built = h.app.renderer.thumb_jobs_built;
    // one request each while they load, not one a frame: each asked once, and the photos in both places once more
    // when the grid's larger size replaces the filmstrip's
    let photos = h.app.session.catalog.len();
    assert!(built > 0 && built <= 2 * photos, "{built} thumbnail requests for {photos} photos");
    for _ in 0..40 {
        h.step();
    }
    assert_eq!(h.app.renderer.thumb_jobs_built, built, "thumbnails are rendered again on every frame");
    // a larger grid asks for larger thumbnails, once
    let r = h.request("ui.set", json!({"thumbSize": 420.0}), T);
    assert_eq!(r["ok"], true, "{r}");
    assert!(h.settle(SETTLE), "the window never comes to rest with larger thumbnails");
    let built = h.app.renderer.thumb_jobs_built;
    for _ in 0..40 {
        h.step();
    }
    assert_eq!(h.app.renderer.thumb_jobs_built, built, "thumbnails are rendered again on every frame at the larger size");
}
