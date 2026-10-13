# The Classic interface style

> **Last reviewed:** 2026-10-13 · **Last updated:** 2026-10-13 · **Change:** minor (Develop's Classic panels: Histogram, tools, Basic … Calibration)

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
| CLASSIC.3 | Develop right panels (Histogram, tools, Basic … Calibration) | done |
| CLASSIC.4 | Develop left panels (Navigator, Presets, Snapshots, History) | next |
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
   - Develop: Classic's panels (below).
4. **Toolbar** under the photo area, between the columns: view modes, rating and flags, zoom.
5. **Filmstrip** across the whole window (`panel:filmstrip`), in every view including the grids; View ▸ Filmstrip
   (F6, `/`) hides it. The loupe, Compare, Survey and Reference do not draw their own strip in this style.

The panels are fixed in place, as in Lightroom Classic. The Modern style's docking is not used here; its saved
arrangement is kept and comes back when the style is switched.

A grid and the filmstrip show the same photos at different sizes, and a photo has one thumbnail: in this style the
larger size and the higher priority of the two requests serve both (`Renderer::thumb_ask_merged`), and the grid
keeps the thumbnails of the photos in the filmstrip (`filmstrip_photos`). Without that each would re-render the
other's thumbnails on every frame.

## Develop's panels

The right column of Develop is one scroll (`crates/ui-egui/src/panels/edit/classic.rs`): the **Histogram**, the
**tools** in a row under it (`icon:crop`, `icon:remove`, `icon:redeye`, `icon:masking`), then the panels in Classic's
order. A tool's button opens its own panel under the row, above the develop panels, and pressing it again puts the
tool away. The profile browser takes the whole column while it is open.

Every control is the Edit panel's own: the same develop control ids, commands and slider widgets (`slider:<id>`), so
presets, copy and paste, History and scripts see no difference between the styles.

| Panel (`classicPanel:<id>`) | Built from |
|---|---|
| Histogram (`histogram`) | the Edit panel's histogram (clipping toggles, exposure line); soft proofing under it |
| Basic (`basic`) | Auto, B&W, HDR, Reset; the profile and its amount; white balance (mode, picker, Temp, Tint); Tone: Exposure, Contrast, Highlights, Shadows, Whites, Blacks (and the HDR sliders); Presence: Texture, Clarity, Dehaze, Vibrance, Saturation |
| Tone Curve (`toneCurve`) | the curve editor: point curves per channel, the parametric sliders, targeted adjustment |
| HSL / Color (`hsl`) | the Color Mixer (the B&W Mixer in black and white), and Point Color |
| Color Grading (`colorGrading`) | the wheels, Blending and Balance |
| Detail (`detail`) | Sharpening, AI Denoise, Noise Reduction |
| Lens Corrections (`lensCorrections`) | Remove Chromatic Aberration, the lens profile, manual Distortion and Vignetting, Defringe, lateral CA |
| Transform (`transform`) | Upright (with guides), Constrain Crop, Vertical, Horizontal, Rotate, Aspect, Scale, Offsets |
| Effects (`effects`) | the post-crop vignette and Grain |
| Calibration (`calibration`) | Shadows tint and the three primaries |

- **Folding.** A click on a header folds its panel; so does `view.classicPanel {panel, open?}`. A panel that stands
  for an Edit section shares that section's open state (Basic with Light, HSL / Color with Color, Lens Corrections
  with Optics, and Detail, Effects, Calibration with their namesakes), so the Edit sections' commands (`section.light`
  … `section.optics`, ⌘1–5) fold them too, and a panel is open in one style when it was in the other.
- **Switches.** Detail, Lens Corrections, Transform and Calibration have a switch at the left of their header
  (`classicSwitch:<id>`) that turns their adjustments off without losing them (`develop.sectionEnabled`). The other
  panels share a develop section with a neighbour (Texture, Clarity and Dehaze belong to Effects in the pipeline and
  to Basic on screen), so a switch there would silently turn off part of another panel; they show one only to turn
  back on a section that was switched off in the Modern Edit panel.

## Commands

| Command | Default key | Menu | Does |
|---|---|---|---|
| `module.library` | ⌘⌥1 | Window | Back to the Library, at the view and panel it was left with |
| `module.develop` | ⌘⌥2 | Window | The active photo in the loupe with the develop panels |
| `view.interface {style?}` | — | View (checkable) | `classic` or `modern`; without `style`, the other one |
| `view.classicPanel {panel, open?}` | — | — | Fold or unfold a Develop panel (`histogram`, `basic`, `toneCurve`, `hsl`, `colorGrading`, `detail`, `lensCorrections`, `transform`, `effects`, `calibration`); without `open`, the other state |

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

- Develop's left column is the Library's sources until CLASSIC.4, and Versions and History still open in the right
  column, in place of the panels.
- Basic, Tone Curve, HSL / Color, Color Grading and Effects have no switch of their own (see Switches above).
- What Classic's panels have and LightCraft has no control for yet: the HSL view of all eight colours per attribute
  (the mixer shows one colour at a time), a lens profile picker (make, model, profile), the fringe eyedropper, the
  process-version menu in Calibration (`develop.updateProcess` exists as a command), and the 1:1 detail preview.
- No solo mode (one panel open at a time) and no Classic panel keys (⌘1–9 by Classic's numbering, Tab, F5–F8).
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

The Develop panels are a child module of the Edit panel (`panels/edit/classic.rs`, declared by one line in
`edit.rs`), so they use its private pieces (histogram, curve editor, mixers, grading wheels) as they are. Two
function boundaries were added without moving code: `edit::shown_settings` (the settings as the panel shows them)
and, in `panels/right.rs`, `crop_tool` and `geometry` (the two halves of the Crop panel). What the Edit panel draws
inline (the Auto row, the profile row, white balance, the vignette style, the optics checkboxes) is drawn again in
`classic.rs`; a fix to one of those upstream has to be repeated there.

Tripwires stand in for merge conflicts. The frame matches on `ViewMode` without a wildcard, so a view added
upstream fails to compile until it is placed in the Classic frame. And
`every_slider_of_the_edit_and_crop_panels_is_in_the_classic_panels` fails when the Modern Edit or Crop panel gains a
slider that the Classic panels lack. The frame's tests also assert the Modern frame is drawn as before.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-13 | minor | CLASSIC.3: Develop's panels in Classic's order, their folding and switches, `view.classicPanel` |
| 2026-10-13 | minor | CLASSIC.2: the Classic window frame; a grid and the filmstrip share their thumbnails |
| 2026-10-13 | major | Created with CLASSIC.1: interface style, module state and commands |
