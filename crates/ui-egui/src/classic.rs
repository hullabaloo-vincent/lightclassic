//! The interface style (Classic or Modern) and, with it, the module the window shows.
//!
//! Classic is the Lightroom Classic arrangement: a module picker (Library | Develop), panels stacked down both
//! sides and a filmstrip across the window. Modern is the single window with a tool strip. This file holds the state
//! and its commands (`view.interface`, `module.library`, `module.develop`, `view.classicPanel`); the drawing is in
//! `panels/classic` and, for the Develop panels, `panels/edit/classic.rs`.
//!
//! The module is derived from the state every other command already sets, so nothing means something different per
//! style: a grid, Compare, Survey or People is the Library; the loupe with an edit tool beside it is Develop
//! ([`reconcile`]). The two module commands are thin setters over the same state.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::LightcraftApp;
use crate::state::{RightPanel, UiState, ViewMode};

/// How the window is arranged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InterfaceStyle {
    /// One window: the photo, a tool strip and one panel at a time.
    #[default]
    Modern,
    /// Modules, with their panels stacked on both sides.
    Classic,
}

impl InterfaceStyle {
    pub const ALL: [Self; 2] = [Self::Classic, Self::Modern];

    pub fn id(self) -> &'static str {
        match self {
            Self::Modern => "modern",
            Self::Classic => "classic",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Modern => "Modern",
            Self::Classic => "Classic",
        }
    }
}

/// A Classic module. More (Map, Print…) can follow; a settings file naming one this build doesn't know is refused
/// like any other unknown value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Module {
    /// Browsing and organising: the grids, Compare, Survey, People and the loupe without an edit tool.
    #[default]
    Library,
    /// One photo with the develop panels.
    Develop,
}

impl Module {
    pub const ALL: [Self; 2] = [Self::Library, Self::Develop];

    pub fn id(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Develop => "develop",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Library => "Library",
            Self::Develop => "Develop",
        }
    }

    /// The command that enters it.
    pub fn command(self) -> &'static str {
        match self {
            Self::Library => "module.library",
            Self::Develop => "module.develop",
        }
    }
}

/// A panel of the Classic Develop column, in Classic's order ([`Self::ALL`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevelopPanel {
    Basic,
    ToneCurve,
    Hsl,
    ColorGrading,
    Detail,
    LensCorrections,
    Transform,
    Effects,
    Calibration,
}

impl DevelopPanel {
    pub const ALL: [Self; 9] = [
        Self::Basic,
        Self::ToneCurve,
        Self::Hsl,
        Self::ColorGrading,
        Self::Detail,
        Self::LensCorrections,
        Self::Transform,
        Self::Effects,
        Self::Calibration,
    ];

    /// Its name in `view.classicPanel` and in widget ids (`classicPanel:{id}`).
    pub fn id(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::ToneCurve => "toneCurve",
            Self::Hsl => "hsl",
            Self::ColorGrading => "colorGrading",
            Self::Detail => "detail",
            Self::LensCorrections => "lensCorrections",
            Self::Transform => "transform",
            Self::Effects => "effects",
            Self::Calibration => "calibration",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::ToneCurve => "Tone Curve",
            Self::Hsl => "HSL / Color",
            Self::ColorGrading => "Color Grading",
            Self::Detail => "Detail",
            Self::LensCorrections => "Lens Corrections",
            Self::Transform => "Transform",
            Self::Effects => "Effects",
            Self::Calibration => "Calibration",
        }
    }

    /// Where its open state lives in [`UiState::open_sections`]. A panel that stands for an Edit section shares that
    /// section's entry, so it is open in one style when it was in the other, and the Edit sections' commands
    /// (`section.light` … `section.optics`, ⌘1–5) fold it too.
    pub fn open_id(self) -> &'static str {
        match self {
            Self::Basic => "light",
            Self::ToneCurve => "toneCurve",
            Self::Hsl => "color",
            Self::ColorGrading => "colorGrading",
            Self::Detail => "detail",
            Self::LensCorrections => "optics",
            Self::Transform => "transform",
            Self::Effects => "effects",
            Self::Calibration => "calibration",
        }
    }

    /// The develop sections its controls belong to (`develop.sectionEnabled`): what switching the panel off turns
    /// off, and what must be on for all of it to take effect.
    pub fn sections(self) -> &'static [&'static str] {
        match self {
            Self::Basic => &["light", "color", "effects"],
            Self::ToneCurve => &["light"],
            Self::Hsl | Self::ColorGrading => &["color"],
            Self::Detail => &["detail"],
            Self::LensCorrections => &["optics"],
            Self::Transform => &["geometry"],
            Self::Effects => &["effects"],
            Self::Calibration => &["calibration"],
        }
    }

    /// Has a switch of its own: the panels whose controls are a whole develop section. The others share a section
    /// with another panel (Texture, Clarity and Dehaze are Effects' in the pipeline and Basic's on screen), so a
    /// switch on one would silently turn off part of the other; they only show one to turn a section back on.
    pub fn has_switch(self) -> bool {
        matches!(self, Self::Detail | Self::LensCorrections | Self::Transform | Self::Calibration)
    }
}

/// A panel of the Classic Develop module's left column, top to bottom ([`Self::ALL`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftPanel {
    Navigator,
    Presets,
    /// The photo's versions.
    Snapshots,
    History,
}

impl LeftPanel {
    pub const ALL: [Self; 4] = [Self::Navigator, Self::Presets, Self::Snapshots, Self::History];

    /// Its name in `view.classicPanel` and in widget ids (`classicPanel:{id}`).
    pub fn id(self) -> &'static str {
        match self {
            Self::Navigator => "navigator",
            Self::Presets => "presets",
            Self::Snapshots => "snapshots",
            Self::History => "history",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Navigator => "Navigator",
            Self::Presets => "Presets",
            Self::Snapshots => "Snapshots",
            Self::History => "History",
        }
    }

    /// Unfolded? The Navigator and Presets follow the settings the Modern style has for them (`view.navigator`,
    /// `panel.presets`); Snapshots and History are kept with the open sections.
    pub fn open(self, ui: &UiState) -> bool {
        match self {
            Self::Navigator => ui.navigator,
            Self::Presets => ui.presets,
            Self::Snapshots | Self::History => ui.section_open(self.id()),
        }
    }

    fn set_open(self, ui: &mut UiState, open: bool) {
        match self {
            Self::Navigator => ui.navigator = open,
            Self::Presets => ui.presets = open,
            // not `toggle_section`: single-panel mode is about the develop panels, and would fold those
            Self::Snapshots | Self::History => {
                ui.open_sections.retain(|s| s != self.id());
                if open {
                    ui.open_sections.push(self.id().to_string());
                }
            }
        }
    }
}

/// `view.classicPanel {panel, open?}`: fold or unfold a panel of the Classic Develop module (`histogram`, a
/// [`DevelopPanel::id`] or a [`LeftPanel::id`]); without `open`, the other state.
fn panel_command(app: &mut LightcraftApp, p: &Value) -> Result<Value, String> {
    let known = || {
        std::iter::once("histogram")
            .chain(DevelopPanel::ALL.map(DevelopPanel::id))
            .chain(LeftPanel::ALL.map(LeftPanel::id))
            .collect::<Vec<_>>()
            .join("|")
    };
    let name = p.get("panel").and_then(Value::as_str).ok_or_else(|| format!("view.classicPanel: `panel` is one of {}", known()))?;
    let open = match p.get("open") {
        None | Some(Value::Null) => None,
        Some(Value::Bool(open)) => Some(*open),
        Some(_) => return Err("view.classicPanel: `open` is true or false".to_string()),
    };
    if name == "histogram" {
        app.ui.histogram = open.unwrap_or(!app.ui.histogram);
        return Ok(json!({"panel": name, "open": app.ui.histogram}));
    }
    if let Some(panel) = LeftPanel::ALL.into_iter().find(|panel| panel.id() == name) {
        let open = open.unwrap_or(!panel.open(&app.ui));
        panel.set_open(&mut app.ui, open);
        return Ok(json!({"panel": name, "open": panel.open(&app.ui)}));
    }
    let panel = DevelopPanel::ALL
        .into_iter()
        .find(|panel| panel.id() == name)
        .ok_or_else(|| format!("view.classicPanel: unknown panel `{name}` ({})", known()))?;
    let id = panel.open_id();
    if open.unwrap_or(!app.ui.section_open(id)) != app.ui.section_open(id) {
        app.ui.toggle_section(id);
    }
    Ok(json!({"panel": name, "open": app.ui.section_open(id)}))
}

/// What [`UiState`] keeps for the interface style.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ClassicState {
    /// The chosen style. `None`: never chosen, which is Modern until the host says otherwise ([`host_default`]).
    pub interface: Option<InterfaceStyle>,
    /// The module showing ([`reconcile`] keeps it true to the view and the panel beside it, in both styles).
    pub module: Module,
    /// The Library's view and right-hand panel, to come back to from Develop.
    pub library_view: ViewMode,
    pub library_right: RightPanel,
}

/// The style the window is drawn in.
pub fn style(ui: &UiState) -> InterfaceStyle {
    ui.classic.interface.unwrap_or_default()
}

/// The window is drawn in the Classic style.
pub fn active(app: &LightcraftApp) -> bool {
    style(&app.ui) == InterfaceStyle::Classic
}

/// The application's starting style where none was ever chosen: Classic, with the left panel it always shows. A
/// style the user picked (Modern included) is left alone. Hosts call it on the state they start with; the library
/// crate's own default stays Modern, which is what its tests and embedders build on.
pub fn host_default(ui: &mut UiState) {
    if ui.classic.interface.is_none() {
        ui.classic.interface = Some(InterfaceStyle::Classic);
        ui.left_panel = true;
    }
}

fn loupe(view: ViewMode) -> bool {
    matches!(view, ViewMode::Detail | ViewMode::Reference)
}

/// Make the module agree with the view and the panel beside it. Runs after every UI command and once a frame (state
/// also arrives through `ui.set` and settings files).
///
/// In the Modern style it only names the module. In Classic it also settles what the two modules never show: an
/// edit tool beside a grid (the Library gets its own panel back), and Develop without its panels.
pub(crate) fn reconcile(app: &mut LightcraftApp) {
    let ui = &mut app.ui;
    let was = ui.classic.module;
    if style(ui) == InterfaceStyle::Modern {
        ui.classic.module = if loupe(ui.view) && ui.right.is_edit_tool() { Module::Develop } else { Module::Library };
        if ui.classic.module == Module::Library {
            ui.classic.library_view = ui.view;
            if !ui.right.is_edit_tool() {
                ui.classic.library_right = ui.right;
            }
        }
        return;
    }
    let module = if !loupe(ui.view) {
        if ui.right.is_edit_tool() {
            ui.right = library_right(ui);
        }
        Module::Library
    } else if ui.right.is_edit_tool() {
        Module::Develop
    } else if was == Module::Develop && matches!(ui.right, RightPanel::None | RightPanel::Versions | RightPanel::Activity) {
        // Versions and History are panels of Develop's left column: their commands fold those, and show the column
        // when they unfold one in it
        let left = match ui.right {
            RightPanel::Versions => Some(LeftPanel::Snapshots),
            RightPanel::Activity => Some(LeftPanel::History),
            _ => None,
        };
        if let Some(panel) = left {
            let open = !(panel.open(ui) && ui.left_panel);
            panel.set_open(ui, open);
            ui.left_panel |= open;
        }
        // Develop keeps its panels: closing the one shown brings Edit back
        ui.right = RightPanel::Edit;
        Module::Develop
    } else {
        Module::Library
    };
    if module == Module::Library {
        ui.classic.library_view = ui.view;
        ui.classic.library_right = ui.right;
    }
    ui.classic.module = module;
}

/// The Library's remembered panel; never an edit tool (a settings file could say so), which would be Develop.
fn library_right(ui: &UiState) -> RightPanel {
    if ui.classic.library_right.is_edit_tool() { RightPanel::None } else { ui.classic.library_right }
}

/// Go to `module`, in either style: Develop is the loupe with an edit tool, the Library the view left for it.
fn enter(app: &mut LightcraftApp, module: Module) {
    reconcile(app);
    let from = app.ui.classic.module;
    let ui = &mut app.ui;
    match module {
        Module::Develop => {
            if !loupe(ui.view) {
                ui.view = ViewMode::Detail;
            }
            if !ui.right.is_edit_tool() {
                ui.right = RightPanel::Edit;
            }
        }
        Module::Library if from == Module::Develop => {
            ui.view = ui.classic.library_view;
            ui.right = library_right(ui);
            // as closing a tool's panel does: the tool goes with it and an open edit step ends
            ui.tool.clear();
            let _ = app.session.end_interaction();
        }
        Module::Library => {}
    }
    app.ui.classic.module = module;
}

/// The style and module commands; `None` for any other id.
pub(crate) fn command(app: &mut LightcraftApp, id: &str, p: &Value) -> Option<Result<Value, String>> {
    let module = match id {
        "module.library" => Module::Library,
        "module.develop" => Module::Develop,
        "view.interface" => return Some(interface(app, p)),
        "view.classicPanel" => return Some(panel_command(app, p)),
        _ => return None,
    };
    enter(app, module);
    Some(Ok(json!({"module": app.ui.classic.module})))
}

/// `view.interface {style?}`: `classic` or `modern`; without `style`, the other one.
fn interface(app: &mut LightcraftApp, p: &Value) -> Result<Value, String> {
    let style = match p.get("style") {
        None | Some(Value::Null) => match style(&app.ui) {
            InterfaceStyle::Classic => InterfaceStyle::Modern,
            InterfaceStyle::Modern => InterfaceStyle::Classic,
        },
        Some(v) => {
            let known = || InterfaceStyle::ALL.map(InterfaceStyle::id).join("|");
            let name = v.as_str().ok_or_else(|| format!("view.interface: `style` is a string ({})", known()))?;
            InterfaceStyle::ALL.into_iter().find(|s| s.id() == name).ok_or_else(|| format!("view.interface: unknown `{name}` ({})", known()))?
        }
    };
    app.ui.classic.interface = Some(style);
    reconcile(app);
    Ok(json!({"interface": style, "module": app.ui.classic.module}))
}

/// The check mark of a style or module command in the menus; `None` for any other id.
pub(crate) fn checked(app: &LightcraftApp, id: &str) -> Option<bool> {
    match id {
        "view.interface" => Some(active(app)),
        "module.library" => Some(app.ui.classic.module == Module::Library),
        "module.develop" => Some(app.ui.classic.module == Module::Develop),
        // in Classic Develop these fold a panel of the left column instead of opening one on the right
        "panel.versions" | "panel.activity" if active(app) && app.ui.classic.module == Module::Develop => {
            let panel = if id == "panel.versions" { LeftPanel::Snapshots } else { LeftPanel::History };
            Some(panel.open(&app.ui) && app.ui.left_panel)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
