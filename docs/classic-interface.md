# The Classic interface style

> **Last reviewed:** 2026-10-13 · **Last updated:** 2026-10-13 · **Change:** major (created: interface style, Library / Develop modules and their commands)

LightCraft's window has two arrangements, chosen in **View ▸ Classic Interface** or **Settings ▸ Interface ▸
Interface Style**:

- **Classic**: modules (Library | Develop), panels stacked down both sides, a filmstrip across the window. This is
  how Lightroom Classic is laid out. It will be the default of this fork's desktop and web apps (CLASSIC.5); until
  then it is chosen by hand.
- **Modern**: one window with a tool strip and one panel at a time, as the cloud Lightroom desktop app is laid out.
  It is what upstream LightCraft draws, and it stays intact: the library crate's own default is Modern, so its tests
  and embedders are unaffected.

Both are layouts over the same commands, catalog and develop settings. Nothing about a photo or an edit depends on
the style.

## Status

| Step | What | State |
|---|---|---|
| CLASSIC.1 | Interface style, module state, `module.library` / `module.develop` / `view.interface` | done |
| CLASSIC.2 | Classic frame: module picker, side columns, full-width filmstrip | next |
| CLASSIC.3 | Develop right panels (Histogram, tools, Basic … Calibration) | planned |
| CLASSIC.4 | Develop left panels (Navigator, Presets, Snapshots, History) | planned |
| CLASSIC.5 | Desktop and web apps start in Classic | planned |

Until CLASSIC.2 the Classic style only changes how modules follow the view (below); the window is still drawn the
Modern way.

## Commands

| Command | Default key | Menu | Does |
|---|---|---|---|
| `module.library` | ⌘⌥1 | Window | Back to the Library, at the view and panel it was left with |
| `module.develop` | ⌘⌥2 | Window | The active photo in the loupe with the develop panels |
| `view.interface {style?}` | — | View (checkable) | `classic` or `modern`; without `style`, the other one |

They run from menus, keys, the control channel (`engine.execute`) and MCP (`--connect`), like every UI command.
`ui.inspect` → `ui.classic` reports `{interface, module, libraryView, libraryRight}`; `interface` is `null` until a
style is chosen. `ui.set {"classic": {"interface": "classic"}}` sets it too.

## How the module is decided

The module is derived from state the other commands already set, so no command means something different per style
(`crates/ui-egui/src/classic.rs`, `reconcile`):

| View | Panel beside it | Module |
|---|---|---|
| Photo Grid, Square Grid, Compare, Survey, People | any | Library |
| Detail, Reference | Edit, Profiles, Crop, Remove, Masking, Red Eye | Develop |
| Detail, Reference | none, Info, Keywords | Library (its loupe) |

In the Classic style two things are settled that the modules never show:

- **An edit tool beside a grid.** Going to a grid from Develop gives the Library its own panel back (Info, Keywords
  or none).
- **Develop without panels.** Closing the Edit panel in Develop brings it back; Versions and History open in
  Develop without leaving it.

So the existing keys land where a Classic user expects without a second keymap: G, N and ⇧C go to the Library; E, C,
H and M open the photo in Develop.

## Known limits

- Only ⌘⌥1 and ⌘⌥2 are Classic's own keys so far. The rest of the keymap is the Modern one (E is Edit, not Loupe; D
  is the loupe, not Develop; R is Radial Gradient, not Crop).
- Quick Develop (the Edit panel beside a grid with several photos selected) is not reachable in the Classic Library
  yet.
- Map, Book, Slideshow, Print and Web have no module.
- Wording is unchanged: Albums, My Photos, Versions.

## Keeping up with upstream

The Classic style is additive: new files (`crates/ui-egui/src/classic.rs`, `crates/ui-egui/src/panels/classic/`) and
hooks of a few lines in `state.rs`, `lib.rs`, `menus.rs`, `menubar.rs` and `panels/settings.rs`. The Modern layout
and the docking code are not edited.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-13 | major | Created with CLASSIC.1: interface style, module state and commands |
