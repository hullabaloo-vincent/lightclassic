//! The Classic interface style: its drawing (state and commands are in [`crate::classic`]).

use serde_json::json;

use crate::LightcraftApp;
use crate::classic::InterfaceStyle;
use crate::theme::Tokens;

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
