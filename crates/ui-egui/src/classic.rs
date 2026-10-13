//! The interface style (Classic or Modern) and, with it, the module the window shows.
//!
//! Classic is the Lightroom Classic arrangement: a module picker (Library | Develop), panels stacked down both
//! sides and a filmstrip across the window. Modern is the single window with a tool strip. This file holds the state
//! and its commands (`view.interface`, `module.library`, `module.develop`); the drawing is in `panels/classic`.
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
        // Develop keeps its panels: closing the one shown brings Edit back
        if ui.right == RightPanel::None {
            ui.right = RightPanel::Edit;
        }
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
        _ => None,
    }
}

#[cfg(test)]
mod tests;
