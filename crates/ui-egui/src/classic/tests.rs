//! The interface style and the Classic modules: their commands, how the module follows the view and the panel
//! beside it, the menus' check marks, and what a settings file without the style means.

use std::time::Duration;

use serde_json::{Value, json};

use super::{InterfaceStyle, Module, host_default, style};
use crate::headless::Headless;
use crate::menubar::MenuNode;
use crate::state::{RightPanel, UiState, ViewMode};
use crate::{LightcraftApp, Services};

const T: Duration = Duration::from_secs(20);

fn app() -> LightcraftApp {
    let services = Services { png: None, ..Default::default() };
    let mut app = LightcraftApp::new(lightcraft_engine::Session::with_demo(), services);
    app.ui.settings.gpu = false;
    app
}

/// The demo library in the Classic style, on the photo grid.
fn classic() -> LightcraftApp {
    let mut app = app();
    app.run("view.interface", json!({"style": "classic"})).expect("the style is set");
    app.run("view.photoGrid", json!({})).expect("the grid opens");
    assert_eq!(app.ui.classic.module, Module::Library);
    app
}

fn run(app: &mut LightcraftApp, id: &str) {
    app.run(id, json!({})).unwrap_or_else(|e| panic!("{id}: {e}"));
}

fn key(h: &mut Headless, k: &str, mods: Value) {
    let mut p = mods;
    p["key"] = json!(k);
    let r = h.request("ui.key", p, T);
    assert_eq!(r["ok"], true, "{r}");
    h.step();
    h.step();
}

/// The checked state of `id` in the menu bar's `menu` (panics when it isn't listed there).
fn menu_check(app: &LightcraftApp, menu: &str, id: &str) -> Option<bool> {
    let bar = crate::menubar::menu_bar(app);
    let (_, items) = bar.iter().find(|(title, _)| title == menu).unwrap_or_else(|| panic!("no {menu} menu"));
    items
        .iter()
        .find_map(|n| match n {
            MenuNode::Item { id: item, checked, .. } if item == id => Some(*checked),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{id} is not in the {menu} menu"))
}

#[test]
fn the_library_crate_starts_modern_and_a_host_starts_classic() {
    // a settings file from before the style existed
    let mut ui: UiState = serde_json::from_value(json!({})).expect("an empty settings file loads");
    assert_eq!(ui.classic.interface, None);
    assert_eq!(style(&ui), InterfaceStyle::Modern, "the crate's own default is Modern");
    ui.left_panel = false;
    host_default(&mut ui);
    assert_eq!(style(&ui), InterfaceStyle::Classic);
    assert!(ui.left_panel, "Classic starts with its left panel");
    assert!(super::LeftPanel::Presets.open(&ui) && super::LeftPanel::History.open(&ui), "and with Presets and History unfolded in it");

    // a style the user chose survives saving, loading and the host's default
    ui.classic.interface = Some(InterfaceStyle::Modern);
    ui.left_panel = false;
    let saved = serde_json::to_value(&ui).expect("the state serializes");
    assert_eq!(saved["classic"]["interface"], "modern");
    let mut loaded: UiState = serde_json::from_value(saved).expect("the state loads");
    host_default(&mut loaded);
    assert_eq!(style(&loaded), InterfaceStyle::Modern);
    assert!(!loaded.left_panel);
}

#[test]
fn view_interface_sets_toggles_and_refuses_unknown_styles() {
    let mut app = app();
    assert!(!super::active(&app));
    let r = app.run("view.interface", json!({"style": "classic"})).expect("classic");
    assert_eq!(r["interface"], "classic");
    assert!(super::active(&app));
    let r = app.run("view.interface", json!({})).expect("toggle");
    assert_eq!(r["interface"], "modern");
    assert_eq!(app.ui.classic.interface, Some(InterfaceStyle::Modern), "a toggle is a choice, kept as one");
    run(&mut app, "view.interface");
    assert!(super::active(&app));

    for bad in [json!({"style": "retro"}), json!({"style": 3})] {
        let e = app.run("view.interface", bad.clone()).expect_err("an unknown style is refused");
        assert!(e.contains("classic|modern"), "{bad}: {e}");
        assert!(super::active(&app), "{bad}: the style is unchanged");
    }
}

#[test]
fn module_commands_switch_between_library_and_develop() {
    let mut app = classic();
    run(&mut app, "panel.info");
    assert_eq!((app.ui.view, app.ui.right, app.ui.classic.module), (ViewMode::PhotoGrid, RightPanel::Info, Module::Library));

    let r = app.run("module.develop", json!({})).expect("develop");
    assert_eq!(r["module"], "develop");
    assert_eq!((app.ui.view, app.ui.right, app.ui.classic.module), (ViewMode::Detail, RightPanel::Edit, Module::Develop));

    // back to the Library as it was left: its view and its panel
    let r = app.run("module.library", json!({})).expect("library");
    assert_eq!(r["module"], "library");
    assert_eq!((app.ui.view, app.ui.right, app.ui.classic.module), (ViewMode::PhotoGrid, RightPanel::Info, Module::Library));

    // already there: nothing moves
    run(&mut app, "module.library");
    assert_eq!((app.ui.view, app.ui.right), (ViewMode::PhotoGrid, RightPanel::Info));
}

#[test]
fn the_module_follows_the_existing_view_and_panel_commands() {
    let mut app = classic();
    // an edit tool opens the photo in Develop
    run(&mut app, "panel.crop");
    assert_eq!((app.ui.view, app.ui.right, app.ui.classic.module), (ViewMode::Detail, RightPanel::Crop, Module::Develop));
    // a grid is the Library, and no edit tool stays beside it
    run(&mut app, "view.gridToggle");
    assert_eq!(app.ui.classic.module, Module::Library);
    assert!(!app.ui.right.is_edit_tool(), "{:?} beside the grid", app.ui.right);

    // the loupe without an edit tool is the Library's loupe, and it is remembered as the Library's view
    run(&mut app, "view.detail");
    assert_eq!((app.ui.view, app.ui.classic.module), (ViewMode::Detail, Module::Library));
    run(&mut app, "module.develop");
    assert_eq!(app.ui.classic.module, Module::Develop);
    run(&mut app, "module.library");
    assert_eq!((app.ui.view, app.ui.classic.module), (ViewMode::Detail, Module::Library));

    // Develop keeps its panels: closing Edit (E again) brings it back instead of leaving the module
    run(&mut app, "panel.edit");
    assert_eq!((app.ui.right, app.ui.classic.module), (RightPanel::Edit, Module::Develop));
    run(&mut app, "panel.edit");
    assert_eq!((app.ui.right, app.ui.classic.module), (RightPanel::Edit, Module::Develop));
    // Versions and History are panels of Develop's left column: their commands unfold those
    run(&mut app, "panel.versions");
    assert_eq!((app.ui.right, app.ui.classic.module), (RightPanel::Edit, Module::Develop));
    assert!(super::LeftPanel::Snapshots.open(&app.ui) && app.ui.left_panel);
    // Info is the Library's
    run(&mut app, "panel.info");
    assert_eq!((app.ui.right, app.ui.classic.module), (RightPanel::Info, Module::Library));
}

#[test]
fn leaving_develop_puts_the_tool_away() {
    let mut app = classic();
    run(&mut app, "panel.remove");
    assert_eq!(app.ui.tool, "remove");
    run(&mut app, "module.library");
    assert_eq!(app.ui.classic.module, Module::Library);
    assert!(app.ui.tool.is_empty(), "the Remove tool is still active in the Library: {:?}", app.ui.tool);
}

#[test]
fn the_modern_style_names_the_module_and_changes_nothing_else() {
    // the crate's starting state: the loupe with Edit beside it
    let mut app = app();
    super::reconcile(&mut app);
    assert_eq!((app.ui.view, app.ui.right, app.ui.classic.module), (ViewMode::Detail, RightPanel::Edit, Module::Develop));
    // Modern keeps Edit beside the grid (Quick Develop); the module is still named, for agents
    run(&mut app, "view.photoGrid");
    assert_eq!((app.ui.view, app.ui.right, app.ui.classic.module), (ViewMode::PhotoGrid, RightPanel::Edit, Module::Library));
    // the module commands work there too
    run(&mut app, "module.develop");
    assert_eq!((app.ui.view, app.ui.right, app.ui.classic.module), (ViewMode::Detail, RightPanel::Edit, Module::Develop));
    run(&mut app, "module.library");
    assert_eq!((app.ui.view, app.ui.classic.module), (ViewMode::PhotoGrid, Module::Library));
    assert_eq!(style(&app.ui), InterfaceStyle::Modern);
}

#[test]
fn a_settings_file_cannot_put_an_edit_tool_beside_the_library() {
    let mut app = classic();
    app.ui.classic.library_right = RightPanel::Masking;
    run(&mut app, "module.develop");
    run(&mut app, "module.library");
    assert_eq!((app.ui.right, app.ui.classic.module), (RightPanel::None, Module::Library));
}

#[test]
fn the_menus_list_the_modules_and_the_style_with_check_marks() {
    let mut app = classic();
    assert_eq!(menu_check(&app, "View", "view.interface"), Some(true));
    assert_eq!(menu_check(&app, "Window", "module.library"), Some(true));
    assert_eq!(menu_check(&app, "Window", "module.develop"), Some(false));
    run(&mut app, "module.develop");
    assert_eq!(menu_check(&app, "Window", "module.library"), Some(false));
    assert_eq!(menu_check(&app, "Window", "module.develop"), Some(true));
    run(&mut app, "view.interface");
    assert_eq!(menu_check(&app, "View", "view.interface"), Some(false));
}

#[test]
fn keys_and_the_control_channel_reach_the_modules() {
    let mut h = Headless::new(classic(), [1400.0, 900.0], 1.0);
    h.settle(Duration::from_secs(120));
    let module = |h: &mut Headless| h.request("ui.inspect", json!({}), T)["result"]["ui"]["classic"]["module"].clone();
    assert_eq!(module(&mut h), "library");

    // Classic's own chords
    key(&mut h, "2", json!({"cmd": true, "alt": true}));
    assert_eq!(module(&mut h), "develop");
    assert_eq!(h.app.ui.view, ViewMode::Detail);
    key(&mut h, "1", json!({"cmd": true, "alt": true}));
    assert_eq!(module(&mut h), "library");

    // the keys there already: E opens the photo in Develop, G goes back to the grid
    key(&mut h, "E", json!({}));
    assert_eq!(module(&mut h), "develop");
    key(&mut h, "G", json!({}));
    assert_eq!(module(&mut h), "library");
    assert!(matches!(h.app.ui.view, ViewMode::PhotoGrid | ViewMode::SquareGrid));

    // state set directly (`ui.set`) is settled by the next frame
    let r = h.request("ui.set", json!({"view": "detail", "right": "masking"}), T);
    assert_eq!(r["ok"], true, "{r}");
    h.step();
    assert_eq!(module(&mut h), "develop");
    let r = h.request("ui.set", json!({"classic": {"interface": "modern"}}), T);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(style(&h.app.ui), InterfaceStyle::Modern);
    let r = h.request("engine.execute", json!({"command": "module.library", "params": {}}), T);
    assert_eq!(r["ok"], true, "{r}");
    h.step();
    assert_eq!(module(&mut h), "library");
}

/// Show Clipping (J, or a click on the histogram) marks the clipped pixels themselves in the loupe, in both styles.
/// It used to draw a red or blue frame around the photo and leave the pixels alone.
#[test]
fn show_clipping_paints_the_clipped_pixels_in_the_loupe() {
    use lightcraft_pipeline::Overlay;
    use lightcraft_pipeline::visualize::CLIPPED_WHITE;
    let settle = Duration::from_secs(120);
    let painted = |img: &egui::ColorImage, area: egui::Rect| {
        let [w, h] = img.size;
        let (x0, x1) = (area.left().max(0.0) as usize, (area.right() as usize).min(w));
        let (y0, y1) = (area.top().max(0.0) as usize, (area.bottom() as usize).min(h));
        let red = egui::Color32::from_rgb(CLIPPED_WHITE[0], CLIPPED_WHITE[1], CLIPPED_WHITE[2]);
        (y0..y1).flat_map(|y| (x0..x1).map(move |x| (x, y))).filter(|(x, y)| img.pixels[y * w + x] == red).count()
    };
    for style in ["modern", "classic"] {
        let mut app = app();
        app.run("view.interface", json!({"style": style})).expect("the style is set");
        app.ui.view = ViewMode::Detail;
        app.ui.right = RightPanel::Edit;
        // four stops over: large parts of the photo are blown
        app.run("develop.set", json!({"control": "light.exposure", "value": 4.0})).expect("exposure");
        let mut h = Headless::new(app, [1400.0, 900.0], 1.0);
        assert!(h.settle(settle), "{style}: the loupe never comes to rest");
        let develop = |h: &Headless| h.app.session.active().and_then(|id| h.app.session.develop_of(id)).expect("a photo is open");
        assert_eq!(crate::panels::detail::view_overlay(&h.app, &develop(&h)), Overlay::None);
        let before = h.snapshot(settle);
        let photo = h.app.image_rect.expect("the loupe shows the photo");
        assert_eq!(painted(&before, photo), 0, "{style}: clipping is marked before it is asked for");

        key(&mut h, "J", json!({}));
        assert!(h.app.ui.show_clipping, "{style}: J turns Show Clipping on");
        assert_eq!(crate::panels::detail::view_overlay(&h.app, &develop(&h)), Overlay::Clipping);
        assert!(h.settle(settle));
        let after = h.snapshot(settle);
        let photo = h.app.image_rect.expect("the loupe shows the photo");
        let marked = painted(&after, photo);
        assert!(marked > 500, "{style}: only {marked} blown pixels are marked in the photo");
        // nothing is drawn around the photo: the pixels beside it are as they were
        let around = photo.expand(4.0);
        let [w, _] = after.size;
        let edge: Vec<(usize, usize)> = (around.top() as usize..around.bottom() as usize).map(|y| (around.left() as usize, y)).collect();
        assert!(edge.iter().all(|(x, y)| after.pixels[y * w + x] == before.pixels[y * w + x]), "{style}: a frame is drawn around the photo");

        // a mask being edited keeps the loupe: its overlay comes first
        key(&mut h, "J", json!({}));
        assert!(!h.app.ui.show_clipping);
        assert_eq!(crate::panels::detail::view_overlay(&h.app, &develop(&h)), Overlay::None);
    }
}
