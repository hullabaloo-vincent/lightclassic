# The Classic interface style

> **Last reviewed:** 2026-10-13 · **Last updated:** 2026-10-13 · **Change:** minor (the Print module: a photo placed on a page, printed to an image file)

LightCraft's window has two arrangements, chosen in **View ▸ Classic Interface** or **Settings ▸ Interface ▸
Interface Style**:

- **Classic**: modules (Library | Develop), panels stacked down both sides, a filmstrip across the window. This is
  how Lightroom Classic is laid out, and it is what this fork's desktop and web apps start in.
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
| CLASSIC.4 | Develop left panels (Navigator, Presets, Snapshots, History) | done |
| CLASSIC.5 | Desktop and web apps start in Classic | done |
| CLASSIC.6 | HSL / Color with every colour, B&W panel; top bar without cloud and chat icons | done |
| PRINT.1, PRINT.2 | Print module: the photo on a page of a chosen shape and size, placed by hand, written as an image | done |

## Which style an app starts in

| Where | Starts in | Why |
|---|---|---|
| Desktop app, web app | Classic, the first time; after that, whatever was last chosen | `classic::host_default`, called on the settings they start with. It sets Classic only where no style was ever chosen (new settings, or a settings file from before the style existed), together with what Classic shows when first opened: both columns, and Presets and History unfolded under the Navigator. A choice of Modern is kept |
| `lightcraft-cli snapshot`, `lightcraft-cli mcp` (headless), tests, embedders | Modern | the library crate's own default (`interface: null`). Send `view.interface {"style": "classic"}` first to draw Classic |

The choice is saved with the other UI settings (`ui.json` → `classic.interface`).

## The window

Top to bottom (`crates/ui-egui/src/panels/classic/mod.rs`, `show`):

1. **Top bar**: menus, search and the status icons, shared with the Modern style, without its community (chat) and
   cloud icons. The cloud still appears as the warning it turns into while a save is failing.
2. **Module picker**: `Library | Develop` at the right of its own row; the module showing is in the brighter text
   (widgets `module:library`, `module:develop`).
3. **Left column, photo area, right column**, side by side. The columns are resized by dragging their inner edge
   (`panel:left_panel`, `panel:right_panel`; the widths are the Modern style's `leftWidth` / `rightWidth`). The left
   column is hidden with View ▸ My Photos Panel (F7); the right one shows whatever panel is open.
   - Library: the sources on the left; Info, Keywords, Versions or History on the right when open.
   - Develop: Navigator, Presets, Snapshots and History on the left, Classic's develop panels on the right (both
     below).
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
| HSL / Color (`hsl`), **B&W** in black and white | every colour at once (Red … Magenta) for one attribute, chosen by `button:hslView-hue` / `-sat` / `-lum`, or all three (`-all`); the targeted adjustment tool; Point Color under them. For a black-and-white photo the panel is titled B&W and holds the eight colours' share of the grey mix (`bw.*`), with Auto |
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

## Develop's left column

Four panels under headers that fold them, the triangle and title at the left as Classic's left panels have them
(`crates/ui-egui/src/panels/classic/develop_left.rs`):

| Panel (`classicPanel:<id>`) | Shows | Open state |
|---|---|---|
| Navigator (`navigator`) | the whole photo (`classicNavigator`); zoomed in, the part the loupe shows is outlined and a click or drag there moves it; Fit and 100% under it | `view.navigator` (the Modern style's "Navigator while zoomed in") |
| Presets (`presets`) | the Presets column as the Modern style has it: groups, hover preview, the Amount slider; Create Preset and the options menu are in the header (`icon:presetCreate`, `icon:presetMore`) | `panel.presets` (⇧P) |
| Snapshots (`snapshots`) | the photo's versions: Create Version, named and automatic | `panel.versions` (⇧V) |
| History (`history`) | the photo's edit steps, newest first; a click goes back to a step | `panel.activity` |

- The Navigator is as tall as it needs; the three lists share the rest of the column (Presets and History twice
  Snapshots' share) and each scrolls by itself.
- In Develop, `panel.versions` and `panel.activity` fold Snapshots and History instead of opening a panel on the
  right, and bring a hidden left column back for the panel asked for. In the Library they open Versions and History
  on the right as before.
- With the Navigator panel there, the loupe does not draw its own mini map while zoomed in. Hide the left column (F7)
  and it does again.
- `view.classicPanel {panel, open?}` folds these four too.

## The Print module

Print lays the active photo out on a page and writes the page as an image file: Lightroom Classic's Print module
with "Print to: JPEG File". It is the third entry of the module picker (`module:print`, ⌘⌥6; File ▸ Print to File…,
⌘P, goes there too).

- **The page** is in the middle of the window (`printPage`), with the photo on it (`printPhoto`). Drag the photo to
  move it: it stays on the page and catches on the page's centre lines. The filmstrip and the sources on the left
  choose the photo.
- **Layout** (right column): the page's shape (1 : 1, 4 : 5, 5 : 4, 2 : 3, 3 : 2, 9 : 16, 16 : 9, A4, A4 wide) and
  its long edge in pixels.
- **Photo**: its size (the share of the page it is fitted inside), where its centre sits across and down the page,
  and buttons that centre it.
- **Page**: the background (white, light, grey, black; any colour by command).
- **Print Job**: JPEG (with quality), PNG or TIFF; the folder; **Print to File** writes one file for each selected
  photo, named `<name>-print`, in the background with a row in the activity stack.

What is on screen is what is written: the photo's rectangle comes from the same `Canvas` the export engine uses
(`crates/engine/src/export.rs`), and the layout is in fractions of the page, so it is the same picture at any page
size. The layout is saved with the other UI settings (`ui.classic.print`).

Print is not a view of its own: it shows over the view it was opened from and holds while that view and its panel
stand, so choosing another photo keeps it and any command that asks for a view or a panel (G, E, D, C…) leaves it.
It exists only in the Classic style; `module.print` says so in the Modern one.

## Commands

| Command | Default key | Menu | Does |
|---|---|---|---|
| `module.library` | ⌘⌥1 | Window | Back to the Library, at the view and panel it was left with |
| `module.develop` | ⌘⌥2 | Window | The active photo in the loupe with the develop panels |
| `module.print` | ⌘⌥6 | Window | The Print module (Classic style only) |
| `print.set {shape?, longEdge?, width?, height?, background?, size?, x?, y?, format?, quality?, dir?}` | — | — | Change the page: `shape` is a key of the list above (`1x1`, `4x5`, `5x4`, `2x3`, `3x2`, `9x16`, `16x9`, `a4`, `a4wide`), sizes are pixels, `size` 0.02..1, `x` / `y` 0..1, `background` a name (`white`, `light`, `grey`, `black`) or `[r, g, b]`, `format` `jpeg` / `png` / `tiff`. Nothing changes unless all of it can be applied |
| `print.export {dir?, path?, ids?, background?}` | ⌘P | File (Print to File…) | Write the page for each selected photo. From the menu outside Print it opens Print first |
| `view.interface {style?}` | — | View (checkable) | `classic` or `modern`; without `style`, the other one |
| `view.classicPanel {panel, open?}` | — | — | Fold or unfold a Develop panel: on the right `histogram`, `basic`, `toneCurve`, `hsl`, `colorGrading`, `detail`, `lensCorrections`, `transform`, `effects`, `calibration`; on the left `navigator`, `presets`, `snapshots`, `history`. Without `open`, the other state |

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
- **Develop without panels.** Closing the Edit panel in Develop brings it back; Versions and History fold the
  Snapshots and History panels of its left column.

So the existing keys land where a Classic user expects without a second keymap: G, N and ⇧C go to the Library; E, C,
H and M open the photo in Develop.

## Known limits

- Develop's left column has no Collections panel: change source in the Library (G), or move along the filmstrip.
- In a short window the three lists of the left column get little room each; fold the ones not in use.
- Basic, Tone Curve, HSL / Color, Color Grading and Effects have no switch of their own (see Switches above).
- What Classic's panels have and LightCraft has no control for yet: a lens profile picker (make, model, profile),
  the fringe eyedropper, the process-version menu in Calibration (`develop.updateProcess` exists as a command), and
  the 1:1 detail preview.
- No solo mode (one panel open at a time) and no Classic panel keys (⌘1–9 by Classic's numbering, Tab, F5–F8).
- The Library's right column is one panel at a time (Info, Keywords), not Classic's stack of Histogram, Quick
  Develop, Keywording, Keyword List and Metadata.
- Only ⌘⌥1 and ⌘⌥2 are Classic's own keys so far. The rest of the keymap is the Modern one (E is Edit, not Loupe; D
  is the loupe, not Develop; R is Radial Gradient, not Crop).
- Quick Develop (the Edit panel beside a grid with several photos selected) is not reachable in the Classic Library
  yet.
- Map, Book, Slideshow and Web have no module.
- Print is one photo to a page. No picture packages or contact sheets in the module (the contact-sheet PDF is still
  File ▸ Contact Sheet…), no margins or guides, no stroke border, no identity plate, no printer output or printer
  profiles, no templates; the page is 8 bits per channel in sRGB.
- Wording is unchanged: Albums, My Photos, Versions.

## Keeping up with upstream

The Classic style is additive: new files (`crates/ui-egui/src/classic.rs`, `crates/ui-egui/src/panels/classic/`) and
hooks of a few lines in `state.rs`, `lib.rs` (the frame is one branch in `LightcraftApp::ui`), `menus.rs`,
`menubar.rs`, `panels/settings.rs`, `panels/grid.rs` and `render.rs` (the shared thumbnails). The Modern layout and
the docking code are not edited, and every hook does nothing in the Modern style.

The Develop panels are a child module of the Edit panel (`panels/edit/classic.rs`, declared by one line in
`edit.rs`), so they use its private pieces (histogram, curve editor, mixers, grading wheels) as they are. Two
function boundaries were added without moving code: `edit::shown_settings` (the settings as the panel shows them)
and, in `panels/right.rs`, `crop_tool` and `geometry` (the two halves of the Crop panel) and `versions_body` and
`activity_body` (Versions and History without their titles). What the Edit panel draws
inline (the Auto row, the profile row, white balance, the vignette style, the optics checkboxes) is drawn again in
`classic.rs`; a fix to one of those upstream has to be repeated there.

Tripwires stand in for merge conflicts. The frame matches on `ViewMode` without a wildcard, so a view added
upstream fails to compile until it is placed in the Classic frame. And
`every_slider_of_the_edit_and_crop_panels_is_in_the_classic_panels` fails when the Modern Edit or Crop panel gains a
slider that the Classic panels lack. The frame's tests also assert the Modern frame is drawn as before, and `tests/classic_accessibility.rs` walks the screen-reader tree of both modules, every Develop
tool and every panel.

The Presets panel is the Modern one drawn whole and moved up by its title row, which is clipped away under the
Classic header (`PRESETS_TITLE_H` in `develop_left.rs`). If upstream changes that row's height,
`presets_has_its_buttons_in_the_header_and_its_list_under_it` fails.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-13 | minor | PRINT.2: the Print module (page preview, Layout / Photo / Page / Print Job panels, `module.print`, `print.set`, `print.export`) |
| 2026-10-13 | minor | CLASSIC.6: HSL / Color shows every colour per attribute and is the B&W panel in black and white; the top bar loses its cloud and chat icons |
| 2026-10-13 | minor | CLASSIC.5: the desktop and web apps start in Classic; accessibility audit of the Classic layout |
| 2026-10-13 | minor | CLASSIC.4: Develop's left column: Navigator, Presets, Snapshots, History |
| 2026-10-13 | minor | CLASSIC.3: Develop's panels in Classic's order, their folding and switches, `view.classicPanel` |
| 2026-10-13 | minor | CLASSIC.2: the Classic window frame; a grid and the filmstrip share their thumbnails |
| 2026-10-13 | major | Created with CLASSIC.1: interface style, module state and commands |
