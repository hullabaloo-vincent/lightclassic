# The Classic interface style

> **Last reviewed:** 2026-10-13 · **Last updated:** 2026-10-13 · **Change:** minor (the Classic window frame: module picker, columns, filmstrip across the window)

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
| CLASSIC.2 | Classic frame: module picker, side columns, full-width filmstrip | done |
| CLASSIC.3 | Develop right panels (Histogram, tools, Basic … Calibration) | next |
| CLASSIC.4 | Develop left panels (Navigator, Presets, Snapshots, History) | planned |
| CLASSIC.5 | Desktop and web apps start in Classic | planned |

## The window

Top to bottom (`crates/ui-egui/src/panels/classic/mod.rs`, `show`):

1. **Top bar**: menus, search and the status icons, shared with the Modern style.
2. **Module picker**: `Library | Develop` at the right of its own row; the module showing is in the brighter text
   (widgets `module:library`, `module:develop`).
3. **Left column, photo area, right column**, side by side. The columns are resized by dragging their inner edge
   (`panel:left_panel`, `panel:right_panel`; the widths are the Modern style's `leftWidth` / `rightWidth`). The left
   column is hidden with View ▸ My Photos Panel (F7); the right one shows whatever panel is open.
   - Library: the sources on the left, Info or Keywords on the right when open.
   - Develop: the tools in a row (`icon:crop`, `icon:remove`, `icon:redeye`, `icon:masking`) above the develop panels.
     A tool's button opens its panel in place of the develop panels; pressing it again puts the tool away.
4. **Toolbar** under the photo area, between the columns: view modes, rating and flags, zoom.
5. **Filmstrip** across the whole window (`panel:filmstrip`), in every view including the grids; View ▸ Filmstrip
   (F6, `/`) hides it. The loupe, Compare, Survey and Reference do not draw their own strip in this style.

The panels are fixed in place, as in Lightroom Classic. The Modern style's docking is not used here; its saved
arrangement is kept and comes back when the style is switched.

A grid and the filmstrip show the same photos at different sizes, and a photo has one thumbnail: in this style the
larger size and the higher priority of the two requests serve both (`Renderer::thumb_ask_merged`), and the grid
keeps the thumbnails of the photos in the filmstrip (`filmstrip_photos`). Without that each would re-render the
other's thumbnails on every frame.

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

- Develop's right column is still the Modern Edit panel (Light, Color, Effects, Detail, Optics, Calibration); Classic's
  own panels are CLASSIC.3. Its left column is the Library's sources until CLASSIC.4.
- The Library's right column is one panel at a time (Info, Keywords), not Classic's stack of Histogram, Quick
  Develop, Keywording, Keyword List and Metadata.
- Only ⌘⌥1 and ⌘⌥2 are Classic's own keys so far. The rest of the keymap is the Modern one (E is Edit, not Loupe; D
  is the loupe, not Develop; R is Radial Gradient, not Crop).
- Quick Develop (the Edit panel beside a grid with several photos selected) is not reachable in the Classic Library
  yet.
- Map, Book, Slideshow, Print and Web have no module.
- Wording is unchanged: Albums, My Photos, Versions.

## Keeping up with upstream

The Classic style is additive: new files (`crates/ui-egui/src/classic.rs`, `crates/ui-egui/src/panels/classic/`) and
hooks of a few lines in `state.rs`, `lib.rs` (the frame is one branch in `LightcraftApp::ui`), `menus.rs`,
`menubar.rs`, `panels/settings.rs`, `panels/grid.rs` and `render.rs` (the shared thumbnails). The Modern layout and
the docking code are not edited, and every hook does nothing in the Modern style.

Two tripwires stand in for merge conflicts. The frame matches on `ViewMode` without a wildcard, so a view added
upstream fails to compile until it is placed in the Classic frame. And the frame's tests assert the Modern frame is
drawn as before.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-13 | minor | CLASSIC.2: the Classic window frame; a grid and the filmstrip share their thumbnails |
| 2026-10-13 | major | Created with CLASSIC.1: interface style, module state and commands |
