@AGENTS.md

## This fork: the Classic interface style
This checkout is a fork (`origin` hullabaloo-vincent/lightclassic, `upstream` storytold/lightcraft) whose owner wants
Lightroom Classic's interface, not the cloud Lightroom layout upstream follows. Read
[`docs/classic-interface.md`](docs/classic-interface.md) before UI work.
- The window has two interface styles. **Classic** (modules, stacked panels: `crates/ui-egui/src/classic.rs`,
  `panels/classic/`, `panels/edit/classic.rs`) is what the desktop and web apps start in. **Modern** is upstream's
  layout and the library crate's own default, so headless snapshots, MCP and tests start there. Where the two would
  differ, design for Classic; never delete or restyle the Modern layout.
- Upstream is merged often. Put Classic work in new files, keep hooks in upstream's files to a few lines, and insert
  function boundaries rather than moving code. Every hook does nothing in the Modern style.
- A slider added to the Modern Edit or Crop panel must be added to the Classic panels too (a test fails otherwise).
  What the Edit panel draws inline is drawn again in `panels/edit/classic.rs`: repeat fixes there.
- Task ids for this work are `CLASSIC.n`. `plan/` (referenced above) does not exist in this fork: start from
  `docs/classic-interface.md` and `docs/gaps.md`.
- The owner's app is built with `scripts/Build LightCraft (macOS).command` (double-clickable: `dist/LightCraft.app`
  for this Mac, ad-hoc signed, optionally installed to Applications). It assembles the bundle as
  `packaging/macos/package.sh` does, from the same templates: when upstream changes the bundle's layout, signing or
  features there, repeat it in the script. Run without a terminal it only builds; never install or open the app
  for the owner unasked.
