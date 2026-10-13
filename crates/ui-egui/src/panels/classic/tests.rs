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

// ------------------------------------------------------------------------------------ Develop's panels

use crate::classic::DevelopPanel;

/// The demo library in the Classic Develop module.
fn develop() -> Headless {
    let mut app = app(true);
    app.run("module.develop", json!({})).expect("Develop opens");
    let mut h = Headless::new(app, SIZE, 1.0);
    assert!(h.settle(SETTLE), "Develop never comes to rest");
    h
}

fn exec(h: &mut Headless, command: &str, params: serde_json::Value) -> serde_json::Value {
    let r = h.request("engine.execute", json!({"command": command, "params": params}), T);
    assert_eq!(r["ok"], true, "{command}: {r}");
    h.settle(SETTLE);
    r["result"].clone()
}

fn sliders(h: &Headless) -> Vec<String> {
    h.app.widgets.iter().filter(|(w, _)| w.starts_with("slider:")).map(|(w, _)| w.clone()).collect()
}

fn unique(mut ids: Vec<String>) -> Vec<String> {
    ids.sort();
    ids.dedup();
    ids
}

/// Unfold everything either style folds: the Edit sections, the Classic panels and the flyouts; HDR on, for its
/// sliders.
fn open_everything(h: &mut Headless) {
    exec(h, "develop.hdr", json!({"enabled": true}));
    h.app.ui.histogram = true;
    h.app.ui.open_sections = ["light", "color", "effects", "detail", "optics", "calibration"].map(String::from).to_vec();
    for panel in DevelopPanel::ALL {
        if !h.app.ui.section_open(panel.open_id()) {
            h.app.ui.open_sections.push(panel.open_id().to_string());
        }
    }
    h.app.ui.open_flyouts = ["curve", "mixer", "pointColor", "grading", "defringe", "sdrRendition"].map(String::from).to_vec();
    assert!(h.settle(SETTLE), "the window never comes to rest with every panel open");
}

#[test]
fn develop_has_classics_panels_in_classics_order() {
    let h = develop();
    // the histogram, the tools under it, then the panels
    let histogram = rect(&h, "classicPanel:histogram");
    let tools = rect(&h, "icon:crop");
    assert!(histogram.bottom() <= tools.top() + 0.5, "the tools {tools:?} are under the histogram's header {histogram:?}");
    let mut above = tools.bottom() - 0.5;
    let order = ["basic", "toneCurve", "hsl", "colorGrading", "detail", "lensCorrections", "transform", "effects", "calibration"];
    assert_eq!(DevelopPanel::ALL.map(DevelopPanel::id), order);
    for id in order {
        let r = rect(&h, &format!("classicPanel:{id}"));
        assert!(r.top() >= above, "the {id} header {r:?} is not under what comes before it ({above})");
        above = r.bottom() - 0.5;
    }
    // not the Edit panel's sections
    for section in ["light", "color", "optics"] {
        assert!(widget(&h, &format!("section:{section}")).is_none(), "the Edit panel's {section} section is drawn in Classic Develop");
    }
}

#[test]
fn a_panel_folds_by_its_header_its_command_and_the_edit_sections_keys() {
    let mut h = develop();
    h.app.ui.open_sections = vec!["light".to_string()];
    h.settle(SETTLE);
    assert!(widget(&h, "slider:light.exposure").is_some(), "Basic is open");
    assert!(widget(&h, "slider:effects.clarity").is_some(), "Presence is part of Basic");

    click(&mut h, "classicPanel:basic");
    assert!(widget(&h, "slider:light.exposure").is_none(), "a click on its header folds Basic");
    let r = exec(&mut h, "view.classicPanel", json!({"panel": "basic", "open": true}));
    assert_eq!(r, json!({"panel": "basic", "open": true}));
    assert!(widget(&h, "slider:light.exposure").is_some());
    // the same again changes nothing
    exec(&mut h, "view.classicPanel", json!({"panel": "basic", "open": true}));
    assert!(widget(&h, "slider:light.exposure").is_some());

    // a panel with no Edit section of its own
    assert_eq!(exec(&mut h, "view.classicPanel", json!({"panel": "transform"}))["open"], true);
    assert!(widget(&h, "slider:geometry.vertical").is_some(), "Transform shows the geometry sliders");
    assert_eq!(exec(&mut h, "view.classicPanel", json!({"panel": "transform"}))["open"], false);
    assert!(widget(&h, "slider:geometry.vertical").is_none());

    // the Edit sections' commands (⌘1–5) fold the panels that stand for them
    exec(&mut h, "section.light", json!({}));
    assert!(widget(&h, "slider:light.exposure").is_none(), "section.light folds Basic");
    exec(&mut h, "section.optics", json!({}));
    assert!(widget(&h, "slider:optics.distortion").is_some(), "section.optics unfolds Lens Corrections");

    // the histogram's header folds it
    assert!(h.app.ui.histogram);
    assert_eq!(exec(&mut h, "view.classicPanel", json!({"panel": "histogram"}))["open"], false);
    assert!(!h.app.ui.histogram);
    click(&mut h, "classicPanel:histogram");
    assert!(h.app.ui.histogram);

    for bad in [json!({}), json!({"panel": "curves"}), json!({"panel": "basic", "open": "yes"}), json!({"panel": 4})] {
        let r = h.request("engine.execute", json!({"command": "view.classicPanel", "params": bad.clone()}), T);
        assert_eq!(r["ok"], false, "{bad} is refused: {r}");
    }
}

/// The tripwire for controls added upstream: whatever slider the Modern Edit and Crop panels show, the Classic
/// panels show too, and none twice.
#[test]
fn every_slider_of_the_edit_and_crop_panels_is_in_the_classic_panels() {
    let mut modern = {
        let mut app = app(false);
        app.ui.view = ViewMode::Detail;
        app.ui.right = RightPanel::Edit;
        Headless::new(app, SIZE, 1.0)
    };
    open_everything(&mut modern);
    let mut want = sliders(&modern);
    exec(&mut modern, "panel.crop", json!({}));
    want.extend(sliders(&modern));
    let want = unique(want);
    assert!(want.len() > 60, "the Modern panels show only {} sliders: is everything open?", want.len());

    let mut h = develop();
    open_everything(&mut h);
    let panels = sliders(&h);
    assert_eq!(unique(panels.clone()).len(), panels.len(), "a slider is drawn twice in the Classic panels: {panels:?}");
    // with the Crop tool open: its own sliders above the panels, and Transform's geometry still once
    exec(&mut h, "panel.crop", json!({}));
    assert_eq!(h.app.ui.right, RightPanel::Crop);
    let with_crop = sliders(&h);
    assert_eq!(unique(with_crop.clone()).len(), with_crop.len(), "a slider is drawn twice beside the Crop tool: {with_crop:?}");
    assert!(with_crop.iter().any(|s| s == "slider:crop.angle") && with_crop.iter().any(|s| s == "slider:light.exposure"));

    let have = unique(panels.into_iter().chain(with_crop).collect());
    let missing: Vec<&String> = want.iter().filter(|w| !have.contains(w)).collect();
    assert!(missing.is_empty(), "sliders of the Modern panels that the Classic panels lack: {missing:?}");
}

#[test]
fn a_slider_drag_in_a_classic_panel_is_one_undo_step() {
    let mut h = develop();
    h.app.ui.open_sections = vec!["light".to_string()];
    h.settle(SETTLE);
    let steps = |h: &mut Headless| exec(h, "history.list", json!({}))["history"].as_array().map_or(0, Vec::len);
    let exposure = |h: &Headless| h.app.session.active().and_then(|id| h.app.session.develop_of(id)).map_or(0.0, |d| d.light.exposure);
    let before = steps(&mut h);
    assert_eq!(exposure(&h), 0.0);
    let r = h.request("ui.dragWidget", json!({"id": "slider:light.exposure", "dx": 50.0, "steps": 8}), T);
    assert_eq!(r["ok"], true, "{r}");
    h.settle(SETTLE);
    assert!(exposure(&h) > 0.0, "the drag moved Exposure");
    assert_eq!(steps(&mut h), before + 1, "one History step for the whole drag");
}

#[test]
fn switches_turn_a_panels_section_off_and_bring_back_one_the_edit_panel_turned_off() {
    let mut h = develop();
    // every panel folded, so that every header is in view to click
    h.app.ui.open_sections.clear();
    h.settle(SETTLE);
    let enabled =
        |h: &Headless, section: &str| h.app.session.active().and_then(|id| h.app.session.develop_of(id)).is_some_and(|d| d.section_enabled(section));
    // the panels that are a whole develop section have a switch; Basic has none while everything is on
    for id in ["detail", "lensCorrections", "transform", "calibration"] {
        assert!(widget(&h, &format!("classicSwitch:{id}")).is_some(), "{id} has a switch");
    }
    for id in ["basic", "toneCurve", "hsl", "colorGrading", "effects"] {
        assert!(widget(&h, &format!("classicSwitch:{id}")).is_none(), "{id} shows a switch with nothing switched off");
    }
    click(&mut h, "classicSwitch:detail");
    assert!(!enabled(&h, "detail"), "the switch turns Detail off");
    click(&mut h, "classicSwitch:detail");
    assert!(enabled(&h, "detail"));

    // Effects switched off in the Modern Edit panel also silences Basic's Texture, Clarity and Dehaze: both panels
    // then show the way back
    exec(&mut h, "develop.sectionEnabled", json!({"section": "effects", "enabled": false}));
    assert!(!enabled(&h, "effects"));
    assert!(widget(&h, "classicSwitch:basic").is_some() && widget(&h, "classicSwitch:effects").is_some());
    click(&mut h, "classicSwitch:basic");
    assert!(enabled(&h, "effects"), "Basic's switch turns Effects back on");
    assert!(widget(&h, "classicSwitch:basic").is_none());
}

#[test]
fn a_tool_opens_under_the_tool_strip_above_the_panels() {
    let mut h = develop();
    h.app.ui.open_sections = vec!["light".to_string()];
    exec(&mut h, "panel.crop", json!({}));
    let (tools, angle, basic) = (rect(&h, "icon:crop"), rect(&h, "slider:crop.angle"), rect(&h, "classicPanel:basic"));
    assert!(tools.bottom() <= angle.top() && angle.bottom() <= basic.top(), "tools {tools:?}, Crop's angle {angle:?}, Basic {basic:?}");
    // the profile browser takes the column, and Develop's panels come back after it
    exec(&mut h, "panel.profiles", json!({}));
    assert_eq!((h.app.ui.right, h.app.ui.classic.module), (RightPanel::Profiles, Module::Develop));
    assert!(widget(&h, "classicPanel:basic").is_none());
    exec(&mut h, "panel.profiles", json!({}));
    assert_eq!(h.app.ui.right, RightPanel::Edit);
    assert!(widget(&h, "classicPanel:basic").is_some());
}
