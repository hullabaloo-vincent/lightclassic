//! The accessibility audit of `accessibility.rs` over the Classic interface style: the module picker, the Library,
//! Develop with each tool and with every panel unfolded, and Print. Fails on an interactive node with no name or a slider without
//! its value and range.
use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::{Harness, kittest::NodeT};
use lightcraft_ui_egui::classic::{DevelopPanel, InterfaceStyle, LeftPanel, Module};
use lightcraft_ui_egui::{LightcraftApp, Services};
use serde_json::json;

/// The demo library as a host starts it (Classic), after `commands`.
fn harness(commands: &[(&str, serde_json::Value)]) -> Harness<'static, LightcraftApp> {
    let mut app = LightcraftApp::new(lightcraft_engine::Session::with_demo(), Services { png: None, ..Default::default() });
    lightcraft_ui_egui::classic::host_default(&mut app.ui);
    assert_eq!(lightcraft_ui_egui::classic::style(&app.ui), InterfaceStyle::Classic);
    for (id, params) in commands {
        app.run(id, params.clone()).unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    let mut h = Harness::builder().with_size(vec2(1500.0, 950.0)).build_ui_state(
        |ui, app: &mut LightcraftApp| {
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(10);
    h
}

fn interactive(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::CheckBox
            | Role::Switch
            | Role::RadioButton
            | Role::Slider
            | Role::SpinButton
            | Role::TextInput
            | Role::MultilineTextInput
            | Role::SearchInput
            | Role::ComboBox
            | Role::EditableComboBox
            | Role::Tab
            | Role::Link
            | Role::MenuItem
            | Role::MenuItemCheckBox
            | Role::MenuItemRadio
            | Role::ListBoxOption
    )
}

/// Every problem found in the harness's current tree, one line each.
fn audit(h: &Harness<'_, LightcraftApp>, place: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let mut count = 0;
    for node in h.root().children_recursive() {
        let n = node.accesskit_node();
        let role = n.role();
        if !interactive(role) {
            continue;
        }
        count += 1;
        let name = n.label().unwrap_or_default();
        let name = if name.trim().is_empty() { n.value().unwrap_or_default() } else { name };
        if name.trim().is_empty() {
            problems.push(format!("{place}: unnamed {role:?} at {:?}", n.bounding_box()));
        }
        if role == Role::Slider && (n.numeric_value().is_none() || n.min_numeric_value().is_none() || n.max_numeric_value().is_none()) {
            problems.push(format!("{place}: slider {name:?} lacks value/range"));
        }
    }
    assert!(count > 5, "{place}: the tree has interactive nodes ({count})");
    problems
}

/// The names in the tree, for the things a screen reader must be able to find.
fn names(h: &Harness<'_, LightcraftApp>) -> Vec<String> {
    h.root().children_recursive().filter_map(|n| n.accesskit_node().label()).collect()
}

#[test]
fn every_interactive_node_has_a_name_in_both_classic_modules() {
    let none = json!({});
    let mut problems = Vec::new();

    // the Library: its grids, its loupe and its panels
    for (place, commands) in [
        ("Library/PhotoGrid", vec![("view.photoGrid", none.clone())]),
        ("Library/SquareGrid", vec![("view.squareGrid", none.clone())]),
        ("Library/Info", vec![("view.photoGrid", none.clone()), ("panel.info", none.clone())]),
        ("Library/Keywords", vec![("view.photoGrid", none.clone()), ("panel.keywords", none.clone())]),
        ("Library/Loupe", vec![("module.library", none.clone()), ("view.detail", none.clone())]),
    ] {
        let h = harness(&commands);
        assert_eq!(h.state().ui.classic.module, Module::Library, "{place}");
        let names = names(&h);
        for module in Module::ALL {
            assert!(names.iter().any(|n| n == module.label()), "{place}: the module picker names {}", module.label());
        }
        problems.extend(audit(&h, place));
    }

    // Develop: as it opens, with each tool, and with every panel unfolded on both sides
    let unfold: Vec<(&str, serde_json::Value)> = std::iter::once("histogram")
        .chain(DevelopPanel::ALL.map(DevelopPanel::id))
        .chain(LeftPanel::ALL.map(LeftPanel::id))
        .map(|panel| ("view.classicPanel", json!({"panel": panel, "open": true})))
        .collect();
    for (place, tool) in [
        ("Develop", None),
        ("Develop/Crop", Some("panel.crop")),
        ("Develop/Remove", Some("panel.remove")),
        ("Develop/Masking", Some("panel.masking")),
        ("Develop/RedEye", Some("panel.redeye")),
        ("Develop/Profiles", Some("panel.profiles")),
    ] {
        let mut commands = vec![("module.develop", none.clone())];
        commands.extend(tool.map(|t| (t, none.clone())));
        let h = harness(&commands);
        assert_eq!(h.state().ui.classic.module, Module::Develop, "{place}");
        problems.extend(audit(&h, place));
    }
    let mut commands = vec![("module.develop", none.clone())];
    commands.extend(unfold);
    let h = harness(&commands);
    let names = names(&h);
    for title in DevelopPanel::ALL.map(DevelopPanel::title).into_iter().chain(LeftPanel::ALL.map(LeftPanel::title)).chain(["Histogram"]) {
        assert!(names.iter().any(|n| n == title), "Develop: the {title} panel's header is named");
    }
    problems.extend(audit(&h, "Develop/every panel"));

    // Print: the page and its panels
    let h = harness(&[("module.print", none.clone())]);
    assert_eq!(h.state().ui.classic.module, Module::Print);
    problems.extend(audit(&h, "Print"));

    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{} accessibility problems:\n{}", problems.len(), problems.join("\n"));
}
