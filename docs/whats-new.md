# What's new in LightCraft

## In this fork (the Lightroom Classic interface)

#### 2026-10-15

### Black & white: the colour sliders work
- In black and white, the colour sliders (Red … Magenta in the B&W panel) now change the photo as much as you would
  expect: Blue −100 turns a blue sky dark grey, close to black, as a red filter does; +100 makes it nearly white.
  Before, −100 hardly changed it.
- A slider changes colourful areas most and leaves greys alone. Auto picks values to match.
- Black-and-white photos you have already edited with these sliders (or with the Red Filter and Rich Mono profiles
  or the High Contrast B&W preset) look stronger now.

#### 2026-10-13

### Print: a photo on a page
- The Classic interface has a Print module (Library | Develop | Print, ⌘⌥6; File ▸ Print Page to File…, ⌘⌥P). It shows a
  page with the photo on it: choose the page's shape and size, set how large the photo is, and drag it where you
  want it. It stays on the page and catches on the centre lines.
- Print to File writes the page as a JPEG, PNG or TIFF, one file for each selected photo, with the background colour
  you chose (white, light, grey or black). What you see on the page is what the file holds.
- The same is there for scripts: `app.export` takes a `canvas` (its size, background, the photo's share and place).
#### 2026-10-13

### Classic interface: colours, black and white, and a quieter top bar
- HSL / Color shows all eight colours at once, for Hue, Saturation or Luminance, or for all three (All). It showed
  one colour at a time.
- For a black-and-white photo that panel is titled B&W and holds one slider per colour (how light the reds, the
  blues… turn out in the grey), with Auto and the tool to drag on the photo.
- The top bar no longer has the cloud icon or the community (chat) icon. If a save is failing, the warning still
  appears in the cloud's place.

#### 2026-10-13

### Classic interface
- View ▸ Classic Interface (also Settings ▸ Interface ▸ Interface Style) arranges the window the way Lightroom
  Classic does: a Library | Develop module picker, panels down both sides, the toolbar under the photo and a
  filmstrip across the whole window, in the Library's grids too. ⌘⌥1 and ⌘⌥2 switch modules, and the keys there
  already land where you expect: G, N and ⇧C go to the Library; E, C, H and M open the photo in Develop.
- Develop has Classic's panels in Classic's order: Histogram, the tools (Crop, Remove, Red Eye, Masking) in a row
  under it, then Basic, Tone Curve, HSL / Color, Color Grading, Detail, Lens Corrections, Transform, Effects and
  Calibration. A click on a header folds its panel; Detail, Lens Corrections, Transform and Calibration have a switch
  that turns their adjustments off without losing them. A tool opens under the tool row, with the panels still below.
- On Develop's left are Navigator, Presets, Snapshots (your versions) and History. The Navigator always shows the
  whole photo; zoomed in, it outlines what the loupe shows, and a click or drag there moves it. ⇧P, ⇧V and Window ▸
  History fold the three lists.
- LightCraft starts in the Classic arrangement, with the Navigator, Presets and History open beside the photo. View ▸
  Classic Interface switches to the Modern one, which is unchanged; the choice is remembered.

## Unreleased

### Flatpak updates
- Help ▸ Check for Updates can now reach the release service and update standalone Flatpak bundles.
  This permissions repair must first arrive through your existing host-side update method. Flatpaks
  managed by a software repository continue to use its package manager, and launcher-pinned versions
  keep their selected version.

### Export Effect Presets leaves your originals alone
- Export Effect Presets… (`mask.exportEffects`) now refuses a path that is a photo's original or its XMP sidecar, as
  the other exports do, and writes the file atomically. It used to write the presets over whatever file it was given,
  and a failed write (a full disk) could leave a cut-off file where an earlier export was.

### Film grain that looks like film (process V7)
- Grain now comes from a simulated emulsion of crystal layers (after Aurélien Pierre's model): dense, soft and cloudy
  rather than dots on top of the photo. It fades out towards black and white as on a print, so shadows keep their depth
  and blown highlights stay clean, and the fit-to-window preview shows the grain you will export. Amount, Size and
  Roughness keep their meaning and Lightroom Classic's strength and grain size. This is rendering process 7: new photos
  (and Reset) get it, photos you already edited keep their grain until you choose Photo ▸ Update to Current Process.
  See [film grain](grain.md).

### Colour noise reduction (process V6)
- Colour noise reduction removes colour noise at the raw default 25, where it used to leave most of it, and keeps
  colour edges at high values instead of bleeding them into fringes (issue #815). It filters the colour differences
  R−Y and B−Y, leaving brightness exactly as it was, along edges found in a denoised guide, and takes isolated
  coloured specks out. New photos get it; photos edited before keep their look until Update to Current Process. A zoomed
  view is smoothed like the same part of the whole photo and the export.

### The web build can be embedded
- A page framing LightCraft in an iframe can open it with `?host=parent` and speak to it over `postMessage`, the
  protocol PdfCraft, GridCraft and DeckCraft share: it hands photos in, receives exports and backups instead of
  downloads, runs commands and hears whether closing would lose unsaved changes (issue #2270; `docs/web.md`).
  A parent on another origin, or a page with an opaque origin, keeps the normal downloads.
- A page can also run LightCraft in its own document, with no iframe: `data-host="page"` on the canvas carries the same
  messages as `CustomEvent`s on the canvas instead of `postMessage` (issue #2270; `docs/web.md`). `?host=parent` wins
  when a page asks for both.

### A library is written only to the folder it was opened from
- LightCraft wrote a library's files to wherever the library's path led at that moment, not to the folder it had opened.
  When a share was mounted over the library's folder, or the folder was moved away and another put in its place while
  the library was open, it wrote into that other folder: it could replace another library's catalog (`catalog.snap`
  and `catalog.log`) with its own, and closing the library removed the note another program had left in
  `catalog.lock.owner` to say that it has the library open (issue #3240).
- LightCraft now remembers the folder it opened (its device and inode number) and, before every save, checks that the
  library's path still leads to it. If it doesn't, nothing is written: not the catalog, its log, the library's
  settings, the lock note or a thumbnail. The not-saved warning says why, names the path and asks you to put the
  original folder back; your changes are kept in memory, quitting asks first as it does for any unsaved change, and
  saving goes on by itself once the original folder is at its path again. That holds for photos, albums and presets; a
  preference that could not be saved is reported where you changed it, and is written with the next preference you
  save. Opening the library at that path again takes the lock of the folder that is there now, so it is refused while
  another program has that library open.
- Saving keeps working after a drive or share is mounted again. A volume that is mounted again gets a new device
  number, and some file systems number their folders anew at every mount (FAT and exFAT drives on Linux, sshfs). The
  folder at the path still counts as the one that was opened when it holds exactly the lock note this session wrote
  into the folder it opened (the note holds a value no other lock's note has), or when it is the same volume mounted
  again: the same inode number and the same creation time under another device number (a system that doesn't know
  when a folder was created can't tell that from another folder). On the same device an inode number is given again
  to a folder made later, so two creation times that are known and differ are two folders. A copy of the library's
  folder made while the library was open holds that note too and counts as the library, and so does a clone of its
  volume; a share mounted over the folder, with another library in it or with nothing, never does. A copy lacks the
  changes made after it was made: they are written to it by the next snapshot, which holds them all, and until then
  they are kept in memory and reported as not saved. If neither tells, the folder counts as another one and nothing is
  written.
- Dropping the lock removes the lock note only if it is the one this session wrote, with the bytes it wrote, in the
  folder it locked or one that holds that note; another program's note is left alone.
- Import ▸ Move removes the originals from the card only after the library's catalog is on disk, not just handed over
  to be written: when the library can't be saved, the originals stay on the card.
- On Windows a folder's identity isn't available to the program yet (Rust's standard library offers it only on its
  nightly builds), so there the check notices a path that no longer leads to a folder, not one that leads to another
  folder; the lock note is protected there too.
- On every system, files outside the library's catalog and settings are still written by path, some of them over what
  is there: originals copied into the library (always under a new name), Smart Previews, profiles, the face-scan
  record (`focus-scanned.bin`), the Lightroom import's `Interop` folder, and what is done to the photos stored in the
  library's `Originals` folder (their sidecars, Save Metadata to File, rename, delete from disk, duplicate and
  conversion to DNG).

### Remove Hot Pixels
- Detail ▸ Noise Reduction has a new box, Remove Hot Pixels (raws only), on for newly imported raws (issue #1345).
  At high ISO single photosites read far brighter than all their neighbours and turn into small red, green or blue
  dots that noise reduction only makes fainter. They are now replaced before the raw is developed. A point of light
  spread over neighbouring photosites, such as a star, is kept; untick the box for star fields that need every
  single-photosite point. Photos edited earlier keep their look: the box is off for them.
- A raw that would have more than 0.2 % of its photosites replaced is left unchanged: such a photo doesn't look like
  hot pixels on a sensor. Stars and other points of light that are kept don't count, so a dense star field still
  loses its hot pixels. Settings ▸ Performance ▸ Hot pixel limit sets the share (0.05 % to 1 %).

### Raising Exposure keeps raw highlights (process 5)
- Raising Exposure on a raw no longer turns the brightest tones the sensor recorded white (issue #1632). The tone
  curve now eases into white a stop at a time instead of reaching it within about a stop of the sensor's white: at
  Exposure +1 and +2 a sky, a lit wall or a face keeps its detail where it used to go flat white. At Exposure 0 the
  brightest tones render a few codes darker (on average 1 code or less on the photos measured), and fully clipped
  areas such as a blown sky stay slightly below pure white until Exposure lifts them further. A DNG whose own tone
  curve already reaches white at the sensor's white renders as before. This is rendering
  process 5: new photos (and Reset) get it, photos you already edited keep their look until you choose Photo ▸ Update
  to Current Process. See [process versions](process-versions.md).

## 2026-11-02

### The Windows title bar is readable from the moment the window opens
- When LightCraft's appearance differs from the one set in Windows (the dark default on a light Windows, for one), the
  title bar kept the system's colour with the title in LightCraft's, so the title looked blank until focus moved to
  another window. It now takes LightCraft's colours as soon as the window has focus.

## 2026-11-01

### Korean and Vietnamese text in exports draws whole
- macOS often stores Korean file names decomposed (NFD), each syllable as its jamo. Contact-sheet captions, export
  watermarks, slideshow overlays and print page text drew those as loose jamo (`ㅈㅔㅈㅜㄷㅗ` for `제주도`), and
  decomposed Vietnamese letters lost their stacked marks. Text is now composed before it is laid out, so both draw as
  they do in the app (issue #3480, #3277).

## 2026-10-31

### Find Similar Photos no longer freezes the window
- The first search reads every photo in the library, and it did that on the thread that draws the window, so in a large
  library the app stopped responding and Windows offered to close it (issue #3468). The photos are now read in the
  background: a "Find Similar Photos" row in the activity stack counts them, you can keep working meanwhile, and its ✕
  stops the search. A stopped search leaves the view as it was, and the next one goes on from what was read. The menu
  item is off while a search runs, and opening another library stops it. When a search ends the grid shows the photos it
  found, also from the Photo menu, and a message says how many. `library.findSimilar` from the control channel and
  `mcp --connect` still gets `{photos}`, now when the search ends; a caller that stops waiting (the control server
  gives a request 60 s) can ask again once that search has ended (while it runs, a new request is refused as already
  running), and what was read isn't read again.

## 2026-11-01

### Import a Lightroom catalog backup as it is
- File ▸ Import Lightroom Catalog… also takes the `.zip` backups Lightroom Classic writes in its `Backups` folder
  (issue #2592), for when Lightroom and its catalog are gone and only the backups and the photos are left. LightCraft
  unpacks the catalog into a temporary folder in the library, imports it and deletes the copy; the zip is not
  changed. Open Library given a `Backups` folder, or one of its dated folders, offers to import the backup.
- Importing a backup of a catalog you already imported no longer adds every collection a second time: collections
  are matched by the catalog's own id, which its backups keep, instead of by the catalog file's path.

## 2026-10-30

### Lightroom catalogs with a print inside a collection import their collections
- Lightroom Classic can keep a saved print, book, slideshow or web gallery inside a plain collection, but a
  LightCraft album sits only inside a folder. Importing such a catalog stopped at the end with "parent must be an
  existing folder" and brought over no collections, and photos whose originals were offline were lost with them
  (issues #2165, #2532). Such a collection now joins the nearest collection set above it, or the top level, and the
  import result names it under warnings; so does one whose parent is system-only or gone from the catalog.

### Lens distortion correction for the Sony A7 III
- Raws from the Sony A7 III (ILCE-7M3) now get the camera's own distortion correction, read from the file, like
  the A7R IVA's: straight lines stay straight at wide angles (the FE 16–35 F4 ZA at 16 mm shifts detail near the
  corners by up to about 40 px on a 24 MP photo) and at long ones (the FE 28–70 at 70 mm). Measured against the
  camera's JPEG, the median error is well under a pixel (up to about 1.5 px in the 16 mm corners). Optics ▸ Enable Lens Corrections turns it off and Profile Distortion scales
  it; photos imported before this pick it up with Photo ▸ Reload from Disk. Vignetting and chromatic aberration
  from Sony files are not applied yet.

### Lens corrections without grey corners
- Lens profiles, the camera's own distortion correction, manual Distortion and Chromatic Aberration now crop just
  inside the corrected photo, as in Lightroom, so no grey corners or edge marks show. Perspective corrections still
  need Constrain Crop to do the same.

### The Flatpak may talk to the accessibility bus
- The Flatpak package now has access to the accessibility bus (`org.a11y.Bus`) and no other new permission; its network, Pictures folder and GPU access are unchanged. Without it the sandbox hid the bus, which AccessKit needs to reach a screen reader such as Orca; this has not been tried with Orca yet. The Flatpak build is the only one affected.

### A keyboard shortcut for Clear Color Label
- Clear Color Label is now a command of its own in Help ▸ Keyboard Shortcuts, so it can have a key like Set Rating or
  Set Flag; before, only the red, yellow, green and blue labels (6–9) had keys and clearing a label was menu-only. It has
  no key until you give it one. Photo ▸ Set Color Label ▸ None runs it and shows its key. The key clears the label of
  the selected photos as one undo step, only the active photo in Compare and Survey, and moves on with Auto Advance,
  as the other label keys do.

### Samsung panoramas import
- Some panoramas from Samsung phones were refused as truncated ("the image data doesn't end with an end-of-image
  marker"), although other apps open them whole: their image data runs straight into the Samsung (SEF) block at the
  end of the file, with no end-of-image marker in between. When the file ends with a valid SEF block, the image now
  ends where that block begins, so these photos import, show and build previews. A JPEG that is really cut short is
  still refused.

### Photos waiting for a library wait for a running import too
- Photos given on the command line or dropped on the launch list are imported once a library opens. If another import
  or a Synchronize Folder is still running then, they now wait for it to end instead of being dropped, and a message
  says so (issue #2779).

### Export sizes keep the pixels typed
- Export ▸ Image Sizing kept pixel sizes only in steps of 16, so a typed 1080 became 1088 and 3000 became 3008. The
  long edge, short edge, width and height fields now keep any whole number of pixels.

### The control channel refuses pointer positions that aren't on any screen
- `ui.click`, `ui.move`, `ui.drag`, `ui.dragWidget`, `ui.pointer` and `ui.scroll` with a coordinate or delta far past
  any screen (such as `1e308`) used to queue an infinite or NaN position. A debug build panicked on it and a release
  build handed it to every widget. Numbers outside ±1,000,000 are now an error, and nothing is queued.

### The histogram stays SDR when HDR is switched off
- After an HDR edit was dragged and HDR was then switched off, dragging a slider showed the old HDR histogram (with its
  SDR / HDR colour bar) again until the drag ended. The histogram now follows the edit: an SDR edit shows the SDR
  histogram while you drag, too.

### At most 64 red eye corrections per photo
- Every pixel of a render goes through every red eye correction, so a sidecar, preset or agent with thousands of
  them could stall a preview or export for hours. A photo now renders its first 64, and Red Eye refuses a 65th.

### Reliability
- A damaged or crafted Olympus ORF, Samsung SRW, Sony ARW or Pentax PEF whose strip sizes add up past the largest
  64-bit number no longer crashes LightCraft (debug builds) when it is read: the file is refused, or read as far as
  its data go.

### A colour-range mask keeps at most 64 samples, however it arrives
- A colour-range mask compares every pixel with every colour it has sampled, so a mask with thousands of samples
  could stall a render or export, or lose the GPU for the rest of the session. `mask.add` and `mask.update` already
  kept the first 64; a sidecar's `lc:settings`, a preset, `develop.merge`, `render --settings` and the catalog and
  edit history now keep the first 64 too, in order, and the extra ones are skipped as they are read. Masks made in
  the app (up to 5 samples) are unchanged.

### A SAM 3 model in the wrong format says so
- Object and Describe need `model.safetensors` from `facebook/sam3` in the Hugging Face format. The
  `sam3.safetensors` on community mirrors is Meta's original checkpoint, with other tensor names; installed in its
  place, AI masks failed with "unsupported dtype C64". LightCraft now says that the file is Meta's original
  checkpoint and which file it needs instead.

### Reliability
- A photo with a mask id at the 32-bit limit (from a damaged or hand-edited sidecar, a paste or an agent) no longer
  breaks the masks added after it: new masks, duplicates and the masks of a preset each get an id no other mask has.
  Before, the next id wrapped around to 0, so a second new mask shared the first one's id (debug builds crashed).

## 2026-10-29

### A person's page opens in seconds in a large library
- Finding the unnamed faces that look like a person (a person's page, and again after each face confirmed there) now
  runs on every processor core, and each comparison of two faces uses the processor's vector instructions. In a
  library of 92 000 unnamed faces, a person with 4 100 named ones is estimated to have taken about 2 minutes 40 seconds
  on one core of an Apple M1 Pro, with the window frozen, and now about 3 seconds (both measured on 3 000 of those
  faces, 5.3 s and 0.1 s, and scaled up). Name suggestions and the order of the unnamed
  faces get the faster comparison too.

### Move Folder To… no longer freezes the window
- Move Folder To…, in the menu of a folder in Folders or Local, now opens its folder dialog the way Import and Move to
  Folder… do: the window keeps drawing and answering while you choose, and the folder moves when the dialog closes,
  with the same "Moved; N photo(s) relinked" message. Before, the dialog held up the whole window until it closed;
  on Linux with X11 the app could stop responding altogether. While the dialog is open, another library can't be
  opened (the move belongs to the library it was started in), and while Open Library… or New Library… waits for
  its folder, Move Folder To… is unavailable.

### A library on a disconnected drive is reported at launch, not replaced
- When the library LightCraft last opened is missing at launch, because its drive or network share isn't connected
  yet or it was moved, the window now says it couldn't open the library and offers Try Again, Choose Another
  Library…, Continue Without Saving and Quit. Before, LightCraft quietly made a new, empty library at that path,
  filled it with the demo photos and opened it, so it looked as if the photos and edits were gone. Nothing is
  written there now, and Try Again opens the library once its drive or share is back: on an empty folder left in
  its place, such as a share's mount point, it says so again instead of making a new library there. A library
  folder LightCraft isn't allowed to read is reported as unreadable, with the system's reason, not as gone. A
  library given with `--library` or `LIGHTCRAFT_LIBRARY`, and the default location on first launch, are still
  created as before.

### The histogram's clipping buttons read in light themes
- The two buttons at the histogram's top corners (shadow and highlight clipping) are no longer near-black tiles with a
  grey chevron. They take the theme's own raised fill, and the chevron stands out from it in Charcoal, Midnight, Silver
  and Paper alike.

### Object and Describe masks show their pins
- With Show Pins on, an Object mask now has a pin where you first clicked the object, and a Describe mask has one in
  the middle of what its description found. Before, these masks had no pin on the photo, so they couldn't be picked
  there. Clicking a pin selects its mask; dragging it changes nothing, as these masks follow what they selected.
  With the Object tool on, a click on another mask's pin selects that mask too, instead of adding a click to the
  selected Object mask, and a click on the selected mask's own pin adds nothing. Subject, Sky, Background and the
  range masks still have no pin.

## 2026-10-28

### The empty photo grid names Import Photos' shortcut as it is bound
- When a view is empty and the way in is File ▸ Import Photos…, the message under "No photos" used a fixed "Cmd+Shift+I" (or a translation's fixed copy of it) whatever the platform or the binding. It now shows the key as the menu bar writes it (on a Mac its glyphs, elsewhere Ctrl+Shift+I) on its own line under the sentence, follows a remapped key, and names no key when Import Photos has none. The sentence now wraps in a narrow window instead of running off its edge.

### The photo in Detail is named with its file for screen readers
- With VoiceOver, the photo in Detail view is now announced with its file name, rating, flag and colour label ("Photo:
  IMG_0042.JPG, 4 stars, picked") instead of only "Photo", as the grid names the same photo.

### The grid's thumbnail size slider is a slider to screen readers
- With VoiceOver, the Thumbnail size control at the right of the grid's bottom bar is now announced as a slider with its
  value and range, instead of a button with no value. Increment and Decrement (VoiceOver's Control-Option-Up and Down) and,
  while it has the keyboard focus, the Up and Down arrow keys change it by one step, as the `=` and `-` keys do.

### Local's standard places can't be renamed by mistake
- The Pictures, Desktop, Downloads and Home rows of Local no longer offer Rename Folder… or Move Folder To…. Before,
  the Rename prompt began with the row's translated label rather than the folder's name, so in German pressing OK
  on “Bilder” renamed your real Pictures folder to “Bilder”, and Home could be renamed the same way. Folders inside
  them, and folders you keep in Local with Browse Folder…, rename as before, starting from their real name. The
  `folder.rename` and `folder.move` commands now refuse the home folder and its Pictures, Desktop and Downloads too,
  so a script or an agent can't do it either.


## 2026-10-27

### The Edit menu groups Select Only Active Photo and Paste Settings from Previous with their kin
- Select Only Active Photo now sits under Deselect All, and Paste Settings from Previous under Paste Selected Settings…, instead of both after Settings…, near the end of the Edit menu. On macOS, where Settings… lives in the application menu, the Edit menu no longer shows two separator lines in a row.

### Screen readers name the sidebar's disclosure triangles
- Each triangle in the left sidebar (album folders, local folders, By Date, imported folders, keywords) and in the
  keyword list now says which row it opens or closes: "Expand Travel", "Collapse 2026". They were all named just
  "Expand" or "Collapse", so VoiceOver read a list of identical buttons.

### Screen readers and Voice Control name the grid's date headers as they read
- A date header in the photo grid is now named with the text it shows ("Sunday, 20 September 2026 · 1 photo"), where
  it was named with the bare date key ("2026-09-20"). VoiceOver reads what a sighted person reads, and Voice Control's
  "click Sunday" finds the header.

### Reading a photo from a network share asks the server less
- Checking a photo (before an import, and for Reload, Recheck, Relink and Convert) asked for the file's size, then
  opened it, then asked for its modification time as a question of its own. It now opens the file once and reads
  both from the open file, so each photo costs the server fewer round trips (about a quarter less time to read a
  set of photos from an SMB share, in one measurement on one share). What is imported and the messages for a file
  that can't be read are the same.

### A refused name prompt stays open and says why
- A one-line prompt (Rename Folder, New Folder on Disk, the preset and version names, New Library for a name it can't
  use, Auto-Tag from Tracklog) that the app refuses, for example a folder name that is taken, now stays open with what
  you typed and shows the reason, so you can change the name. Before, it closed without a word. A New Folder on Disk
  that works now says so: “Created folder “Selects” in “2026/Shoot””. Cancel and Esc still close the prompt.

### The macOS menu bar shows every row
- File ▸ Export… no longer reads “Custom…”, File ▸ Export with Preset keeps its Custom… row, and Photo ▸ Add to Album keeps New Album…. When two menu rows ran the same command, the menu bar left the second one out and gave the first one its label; now each row is its own item and a click on either one runs the command. The View menu now also lists its second Layout Image row (without a key), as the in-window menu does.

### Shortcuts read the same way everywhere on macOS
- Wherever LightCraft shows a shortcut (the menu bar's labels, a button's tooltip, the empty-library hint, the
  line under "No photos" in an empty album, the Keyboard Shortcuts dialog and its search) it lists the modifiers in
  the order macOS menus use, Control, Option, Shift, Command: `⇧⌘I` where it used to say `⌘⇧I`. The line under
  "No photos" also shows the shortcut as it is bound now, and as Ctrl+Shift+I on Windows and Linux, where it
  always said Cmd+Shift+I. Windows and Linux otherwise keep the order a shortcut is written in.

### Screen readers name a slider's value field "value"
- The number beside a slider is now named "Exposure value" (the slider's name and "value"), and "Click to type a value,
  drag to change it" is its description. It was named "Exposure: Click to type a value, drag to change it", so
  VoiceOver read the whole sentence each time it reached the field, once for every slider in the Edit panel.

## 2026-10-29

### A dialog no longer dims the macOS title bar
- On macOS the top bar is the window's title bar, and the window buttons drawn over it stayed bright while a dialog dimmed
  the bar under them. As under a macOS sheet, the dialog's backdrop now leaves the bar alone: its buttons are inactive, a
  click on it does not close the dialog, and dragging its empty space still moves the window. Windows, Linux and full
  screen are unchanged.

## 2026-10-28

### The window's parts are named for screen readers
- The top bar, the tool strip and the bottom bar are now announced as toolbars (Top bar, Tool strip, Bottom bar), and the
  sidebar (Library), the photos, the Tools panel (or Presets) and the filmstrip as landmarks, so a screen reader can list
  them and jump to one instead of walking every control. Nothing on screen changes. Not yet tried with a real VoiceOver session.

### Dialogs are exposed to screen readers as dialogs
- Export, Settings and the other dialogs, the library chooser and the prompts that stop the window (the quit question, a
  library that could not be opened, a settings file problem) are now exposed to screen readers as dialogs: modal ones, so the
  rest of the window is marked as blocked while one is open, and an alert dialog for the prompts. The answer to Check for
  Updates is a dialog too, but not a modal one. Before, each was a plain group that only carried the dialog's title.
- An alert dialog also carries what it says as its description: a screen reader can read the library problem's path, error and
  explanation, or a prompt's text, together with the dialog.

### Screen readers no longer read the shortcut with every button name
- Icon-only buttons (the tool strip, the view buttons and Before/After and Filmstrip in the bottom bar, the top bar's
  Import Photos…, the mask overlay and White Balance Selector buttons, the slideshow controls) are now announced by
  their label alone ("Edit", "Presets", "Photo Grid"), and the shortcut is the button's description ("E", "Shift+P",
  "G"), which VoiceOver reads after a pause or on request. Before, it read "Edit (E)" every time it landed on the
  button. This holds in every language, including Japanese and Traditional Chinese, which write the shortcut in
  full-width brackets. The tooltip you see on hover is unchanged.

## 2026-10-25

### A one-field white balance edit no longer shifts the other one
- Settings that name only a white balance temperature, or only a tint — `develop.merge`,
  `lightcraft-cli render --settings <partial JSON>`, an MCP `set_develop {settings: …}` call, or a hand-written
  preset naming one of the two — used to change the colour balance of photos they never mentioned. On a raw edited in
  Kelvin the field the partial left out was re-read on the other scale: `wb {temp: 5800}` on a photo at 5000 K /
  +12 tint came back at 5800 K with the +12 read as a relative shift, moving the green–magenta balance by a visible
  amount. The kept field now carries the look the photo had, and the result is stored on the scale that photo is
  edited on.
- The same partial on a photo still on As Shot — or on a named white balance preset, on a raw with a Kelvin scale —
  stored the number without any render ever reading it: those modes answer their own white, so the panel showed
  5800 K over a picture still at the file's white. That white is now written out as Custom values before the partial
  lands; a partial that names a mode itself still gets that mode.
- Presets made from a photo and the crs / XMP and Luminar importers name mode, temperature, tint and scale together,
  and partials naming both fields (or neither) behave exactly as before; a partial that says `"scale": "kelvin"` is
  still read in Kelvin.

### JPEG ICC chunks are validated before colour conversion
- JPEG ICC profiles are assembled only when every chunk is present exactly once, the sequence numbers and
  declared totals agree, and every APP2 segment is complete. Reordered valid chunks still work. An invalid
  sequence uses the existing unsupported-profile sRGB fallback, including when the JPEG is a raw preview.


## 2026-10-25

### `>=` and `<=` work in the search box
- Fielded search tokens now take inclusive comparisons: `rating:>=3`, `iso:<=400`, `f:>=2.8`, `focal:<=50`. Before,
  the `=` was read as part of the number, so such a search quietly showed no photos at all.

### Import rename numbering stops safely at its limit
- Copy and Move imports can use the last representable rename number once. If more renamed files remain, each is
  reported as failed with guidance to lower `renameStart` or leave `rename` blank; completed transfers stay
  accounted for and catalogued. A `renameStart` value outside this platform's range is rejected before setup.

### Incomplete timestamps no longer become plausible photo times
- An ISO date ending in `T`, a clock with extra components, or a malformed fractional second is no longer read as a
  different valid time. XMP keeps the original property available and can use the next valid capture-time field;
  GPX treats an invalid point time as untimed, so it cannot tag a photo at a fabricated time.

### Invalid EXIF capture clocks no longer become plausible dates
- An EXIF capture time with extra clock components is ignored as a whole. LightCraft can use a valid DateTimeDigitized,
  IFD0 DateTime or embedded XMP value instead, while All Metadata continues to show the original tag. This also keeps
  malformed timestamps from creating false matches when geotagging from a track log.

### Drags reach the screen a frame sooner
- The window queues one frame ahead of the display again instead of two, as eframe does by default (since #136 it
  used egui-wgpu's high-throughput setting). Sliders, crop handles, brushes and every other drag show up one display
  frame sooner: 8.3 ms at 120 Hz, 16.7 ms at 60 Hz. On a MacBook Pro at 120 Hz, a heavy develop drag went from
  about 18 ms to about 10 ms between a frame being handed to macOS and macOS compositing it, with the same 120 frames
  a second. On a Mac the window also holds one drawable less: its width × height × 4 bytes, 15–22 MB for a
  1200–1500-point-wide window on a Retina display.


## 2026-10-24

### AI Denoise follows the photo's real noise
- The built-in denoiser used to read a photo's fine texture as noise, so it always believed a photo was at least as noisy as
  about ISO 12800 and over-smoothed quiet photos while under-cleaning noisy ones. It now measures each colour channel
  separately, ignores texture, and sets its strength from the noise it actually finds. On clean photos from a Canon EOS R and
  an R6 Mark III with noise added at real ISO levels, the result at ISO 25600 is 2.2 to 2.7 dB closer to the original, and at
  ISO 51200 about 5.5 dB closer; at ISO 6400 it is 0.6 to 1.0 dB closer, and quiet photos are no longer smoothed
  needlessly. Photos denoised before are denoised again when they are next needed. See [AI Denoise](denoise.md).

### Better built-in AI Denoise
- The built-in denoiser now cleans with block-matching collaborative filtering: similar blocks of the picture are
  grouped, transformed together and thresholded, so more fine detail survives at the same level of cleaning. On a
  public raw with added noise it scores 0.25 to 0.6 dB higher than before at every noise level and keeps more
  texture at medium to extreme noise (medium: 44 % of it against 42 %; extreme: 12 % against 9 %), a little less at
  light noise (58 % against 62 %), with no colour blotches or isolated specks. It is as fast as the old filter on the processor, and when the computer has a graphics card that is faster
  it runs there: on an RTX 4080 SUPER a photo's tiles go about 6 to 10 times faster, with the same picture (Settings ▸ AI
  Denoise ▸ Run on chooses; Automatic times both). Photos
  denoised before are denoised again the next time they are needed. See [AI Denoise](denoise.md).

### A right-click on a text field while you carry photos opens no menu
- While you drag photos from the grid, a right-click on a text field, such as the search box, opens no Cut, Copy and
  Paste menu. Before, the menu opened over the field the photos were on. Right-clicking a field without a drag works
  as before.

### Typed GPS coordinates are validated before editing metadata

- Entered coordinates must contain a complete decimal or degrees/minutes/seconds value. Unknown text, exponent notation, wrong-axis or conflicting hemispheres, and minutes or seconds outside [0,60) are rejected instead of being silently ignored and reinterpreted. A rejected GPS value also leaves the other fields in the same metadata edit unchanged.

### The Slideshow settings page stays put while the preview renders

- In Settings ▸ Slideshow, the line under the slide preview ("The selected photo as a slide…") now shows from the first
  frame instead of only once the preview has finished rendering. Before, it appeared with the render, pushing the page
  below it down by a line, so a click made in that moment could land beside the control you aimed at.

### A refused move no longer switches the Move Photos question off
- Ticking Don't ask again in the question that appears when you drop photos on a Folders row used to be remembered even when the move was then refused (for example because a file of the same name is already in that folder). The choice is now kept only once the move has started, so a refused move leaves the question on.

### Importing a folder from a network share waits less
- After the import review, the import used to check that each photo was still as the review had seen it by asking the
  disk about one photo after another, so on a network share every photo waited for a round trip to the server. It now
  asks about up to 16 photos at a time, the same wait for every 16 instead of for each one. Reading the photos in the review
  is unchanged.
- Looking for each photo's `.xmp` sidecar, which asks the disk several names for every photo, also goes up to 16
  photos at a time.

### Original export checks embedded place names when removing location
- Original copies are refused when “Remove Location Info” is enabled and the source contains GPS or a nonblank sublocation, city, state or country. Use JPEG or TIFF for rendered output; DNG is available for raw photos.

### Auto straighten finds the horizon and plumb lines
- Auto straighten (Auto in the crop panel, or a double-click with the Straighten tool) now finds a sea or
  landscape horizon among waves and clouds, and levels buildings by where their verticals converge, so a photo shot
  upward is no longer turned by the lean of its walls. Measured on 113 photos straightened by hand in Lightroom
  Classic, it straightens 50 of them instead of 29, without making more of them worse.
- It turns a photo by at most 5° (before: 10°); only two of those photos needed more.

### Sidecar keywords stay searchable in other applications
- Saving an XMP sidecar wrote each keyword's path into `dc:subject`, so after any change, even a star rating, a
  photo keyworded `Places|Italy|Rome` listed `Places|Italy|Rome` there instead of `Places`, `Italy` and `Rome`,
  and a search for "Rome" in Lightroom, darktable, digiKam or another DAM no longer found it. Sidecars now get the
  names in `dc:subject` and the paths in `lr:hierarchicalSubject`, as Lightroom Classic writes them, and LightCraft
  reads them back as the same keywords. A sidecar written by an earlier version is put right the next time it is
  saved.


### Relative-date filters and smart albums follow the clock
- A current view or source count using “in the last” rules now refreshes when the session clock moves into a new
  minute, even when no catalog edit occurred. This fixes stale photos remaining in the grid and stale counts for
  direct or nested smart albums. Held views keep photos that expired until Refresh View while admitting photos that
  newly match; expiry itself does not create an undo step.

### The histogram's tonal zones adjust the Basic tones
- The histogram is a control surface again, as in Lightroom Classic: hovering one of its five zones (Blacks, Shadows,
  Exposure, Highlights, Whites) tints it and shows its name and value under the plot ("Exposure: +0.45 EV"). Dragging one
  sideways moves that control — the zone you pressed stays locked while the pointer crosses the others, Shift drags a
  tenth as far, and the whole drag is one undo step. Double-clicking a zone puts its control back to its default.

### Drag the parametric Tone Curve
- In the Tone Curve's parametric mode the graph is now drawn from the renderer's own curve, so it always matches the
  photo, and the line can be dragged: pressing it picks the region under the pointer (Highlights, Lights, Darks or
  Shadows) and moving up or down changes that slider, as one undo step. Hovering the line names the region first.
## 2026-10-24

### Virtual-copy numbering stops cleanly at its limit
- Creating a numbered virtual copy now reports when the `Copy N` sequence is exhausted, before any copies in a
  batch are created. An explicit name remains available even when the numeric sequence is full.

### A right-click while you carry photos opens no menu
- While you drag photos from the grid, a right-click opens no context menu, on an album, folder or keyword row in
  the sidebar or on any other row or thumbnail that has one. Before, the menu opened on the row the photos were over.
  The menu of a text field (Cut, Copy, Paste) is not held back.

### Adding photos that are already in an album no longer costs an undo step
- Add to Album and dragging photos to an album now leave the album, the undo history and Redo alone when every
  photo was in the album already. Before, they still recorded an undo step, so Redo was lost and Undo had a step
  that changed nothing. Photos the library no longer has (a selection kept after an Undo took them away) are also
  left out of a new album made from the selection, out of Add to Album, and out of the target album (B), which does
  nothing and says nothing when none of them is left, and counts only the photos it changed.

## 2026-10-23

### AI Denoise works out of the box
- Turning on AI Denoise no longer asks for an ONNX model and a `denoise-model.json`: LightCraft's own built-in
  denoiser runs automatically. Installing a model of your own is optional (Settings ▸ AI Denoise ▸ Your own model);
  Stop using returns to the built-in one. The set-up prompt and its "resume after installing" flow are gone. See
  [AI Denoise](denoise.md).
- The built-in denoiser measures the photo's noise once, cleans the four colour planes together on variance-stabilised
  data (so colour blotches go too), and cleans harder, with a second pass, the noisier the photo is. Colour noise gets
  a wider pass of its own, and in heavy noise isolated specks are smoothed away. On a public raw with added noise it
  scores 3.7 to 10.4 dB higher than the noisy picture, the most at heavy noise, and keeps more fine texture there
  than before (heavy noise: 22 % of it against 20 %; extreme: 9 % against 7 %).

### Export subfolders stay inside the export folder
- An export subfolder (typed in the Export dialog, saved in a preset or passed to `export`) can no longer lead
  outside the chosen folder (issue #2571). `..` and `.` parts and a leading `/` are dropped, and `\` and `:` become
  `_`, so a shared preset with `../..` in its subfolder writes inside your folder instead of elsewhere on disk.
  Nested subfolders such as `Client/Jobs` still work.

### Place names always download into the settings folder
- Download Place Names (for Fill Location from GPS) always saves the GeoNames files in the settings folder (issue
  #2575). An agent or script driving LightCraft could name another folder and another server, and have that
  server's files written there; it is now refused. Each file is saved complete or not at all, and replaces whatever
  is in its place instead of writing through a link to somewhere else.

### Videos in the grid are no longer sent to the image decoders
- A video in the library (an MP4 or MOV brought in from a Lightroom catalog) no longer asks the image decoders for a
  grid thumbnail (issue #2682). They can't read one, so each video failed with "unrecognised image format", showed a
  "!" and was tried again whenever the library changed, filling the log. Its cell now keeps the plain placeholder until
  LightCraft can show video frames. The background face search skips videos too, instead of failing on each one at
  every launch. A video is recognised by its type, by the format `VIDEO` that Lightroom catalogs
  give videos, or by its extension (`.mp4`, `.mov`, `.m4v`, `.avi`, `.mts`, `.m2ts`, `.3gp`, `.mkv`, `.webm`, `.wmv`,
  `.mpg`, `.mpeg`).
### Right-button drags no longer pick up photos in the grid
- Dragging with the right mouse button over a photo in the grid used to pick up the photos and, on a photo that wasn't
  selected, replace the selection with it. Now only the left button drags photos, and a right-button release while you
  are dragging photos no longer drops them on an album, folder or keyword. Right-click still opens the photo menu.

### A damaged GIF or BMP can no longer crash an import
- A GIF or BMP whose header claims an enormous picture (a file of a few dozen bytes can claim a billion pixels) is
  now refused with an error when importing or opening it (issue #2553). Before, LightCraft set aside memory for the
  whole picture, several gigabytes, and could close without warning, losing unsaved work. At import, a GIF or BMP
  picture larger than 4 megapixels now has its size read from the header instead of being decoded; smaller ones are
  still decoded, so a damaged file is caught then. A picture that is mostly blank (a run-length-encoded BMP that ends
  early, or a GIF whose first frame is smaller than the picture) still opens up to 4 megapixels; a larger one is
  refused, because opening it would still set aside memory for the whole picture.

### Merge previews after a failed one
- The HDR, Panorama and HDR Panorama dialogs make a new preview each time they open (issue #2708). After a preview
  had failed once, for example because a photo's file was missing, opening the dialog again with the same options
  showed neither a preview nor an error until an option was changed.

### Edits read from a sidecar are kept in range
- LightCraft edits read back from an XMP sidecar (Read Metadata from File, or at import) are now checked like presets
  and pasted settings: sliders and curves are kept in their ranges and the straighten angle within ±45° (issue
  #2555). A turned crop that reaches far past the photo is trimmed to the photo when it is shown or exported. Before,
  a damaged or hand-edited sidecar could set a crop that made a full-size export ask for trillions of pixels and
  close the app. Crops made with the crop tool are unchanged.

### Text boxes in Settings
- Esc in a text box of Settings (Copyright, Creator, the Auto Import album, the two External Editing boxes) gives
  the text back and stays in Settings; in the first three it used to save what was typed, and in all of them it
  closed the window. Those three now have the Cut / Copy / Paste menu too, and what is typed in Copyright or
  Creator is saved when another tab is clicked, where it used to be dropped.

### The histogram stays in view
- The histogram stays at the top of the Edit panel while its sliders scroll under it (issue #2679). It used to scroll
  away with them, so it was out of sight while using the sliders further down the panel.

### No maze pattern in noisy raws
- Flat areas of high-ISO raws, such as a dim wall or a blurred background, could show short horizontal and vertical
  strokes instead of grain, often orange and blue under warm light. Noise reduction made them stand out more, because
  it smooths the grain around them. The demosaic chose between a horizontal and a vertical guess at each pixel, and
  in a flat area the noise alone decided which. Where neither guess is clearly better it now blends the two, so the
  noise looks like grain. Edges are as sharp as before. Thumbnails and previews that LightCraft rendered from a raw
  are redrawn; the camera's own embedded preview, shown for an unedited raw until its render is ready, is unchanged.


### Tab leaves the name field in New Album, New Smart Album and the name prompts
- In New Album, New Smart Album, Save Metadata Preset and the other dialogs with one name field, the field has the
  keyboard focus when the dialog opens, and Tab now moves on to the check box, Cancel and OK (Shift+Tab goes back).
  Before, the field took the focus back every frame, so only Return and Esc got you out of it. Return still
  confirms and Esc still cancels.

### A thumbnail that can't be read is tried again only when its photo changes
- A file the grid can't draw (a damaged JPEG, a file the decoders can't read) was decoded again after every change to
  the library, even a change to another photo: face recognition and metadata reads at launch made each such file
  fail 20 to 40 times in a few minutes, one log warning each time (issue #2682). It is now tried again when that photo
  changes (Locate Missing File, Reload, an edit) or when the library can't say which photos changed. A photo whose
  file is missing, such as one on a drive that is unplugged, is still tried again at every change, so it comes back
  once the drive does.

### Foreign XMP metadata survives namespace-prefix collisions
- Saving a sidecar keeps another application's properties even when its namespace uses prefixes such as `lc`
  or `xmp`. Those foreign properties no longer become LightCraft settings or ratings when imported.

### The log can show every thumbnail and preview LightCraft asks for
- With `RUST_LOG=warn,lightcraft*=info,lightcraft_ui_egui::render=debug` (or `LIGHTCRAFT_LOG=debug`), the log has a
  line for each picture LightCraft asks for, with its file, and one for how it ended: drawn, or dropped and why.
  Before, only failures were logged, so the log couldn't tell a file asked for 37 times from one asked for once and
  failing inside (issue #2682).
  The usual log is unchanged. See [Get help with a problem](user/troubleshooting/get-help.md).

### Find Missing Photos finds the files of a library made on Windows
- A library made on Windows records its originals with Windows paths. Opened on macOS or Linux, its photos are
  missing until relinked, and Find Missing Photos found them only by their content, so it missed every photo it had
  no fingerprint for (such as photos brought in from a Lightroom catalog) or that had been edited since, even in a
  folder holding every file under the same name. It now matches them by name as it does any other photo: the name
  after the last `/` or `\`, in any letter case, checked against the size when the library has one. Several files
  of that name are still reported rather than guessed, and undo puts back the paths as they were.

### Auto straighten leaves straight photos alone
- Auto straighten (Auto in the crop panel, or a double-click with the Straighten tool) no longer turns a photo by
  more than 10°, and acts only when at least three lines agree. Before, it often turned photos that were straight:
  the edges of wooden planks, a branch or a bird's wing were taken for the horizon, sometimes for a 25° turn.
- When it finds no horizon or vertical lines, it says so instead of doing nothing without a word.

### PSD and PSB metadata stays available
- LightCraft now imports PSD and PSB files with EXIF and XMP image resources. Capture dates, GPS and descriptive fields
  stay available in All Metadata; PSD edit copies keep their capture date. Original exports with Remove Location Info enabled
  are refused when a PSD or PSB embeds GPS, since LightCraft cannot strip metadata from a byte-for-byte original copy.

### Quitting on macOS leaves no lock note
- Quitting LightCraft on macOS no longer leaves `catalog.lock.owner` in the library folder. The note, which names the
  program that has the library open, stayed behind after Quit and the window's close button, pointing at a process
  that no longer existed. The library's lock is now released as soon as everything is saved, as it already was on
  the other systems.

### Asking for the missing photos no longer freezes the window
- `library.missing` (the control channel and `mcp --connect`) looked at every photo's file on the thread that draws
  the window, so on a library of hundreds of thousands of photos on a network share the app stopped answering for
  minutes. The files are now checked in the background: a "Missing Photos" row in the activity stack counts them and
  its ✕ stops the check, and the request is answered with the same list when the check ends. A second request meanwhile
  shares the running check. A caller that stops waiting (the control server gives a request 60 s) gets the answer of
  that check when it asks again within ten minutes, instead of another walk of the share, as long as the photos have
  not changed since; otherwise the files are checked again. If photos change while the files are being checked (an
  import adds some, others are removed, moved to Recently Deleted or relinked), the answer leaves out the photos that
  no longer apply, a photo added meanwhile is not part of that check, and the answer is not kept for a later request.


### Choose where a Lightroom catalog goes
- File ▸ Import Lightroom Catalog… now asks first: merge the catalog's photos, collections and edits into the
  library open now, or create a new library for them, named after the catalog and placed next to the open library
  (both can be changed). LightCraft opens the new library, then imports the catalog into it. Cancel does nothing.
  Open Library… on a Lightroom catalog goes through the same choice.
- To import as before without the question, choose Settings ▸ General ▸ When importing a Lightroom catalog ▸
  Always merge into the open library, or tick Don't ask again in the window. Ask where to import it brings the
  question back.

### Choose which library opens at launch
- Settings ▸ General ▸ **At launch**: *Open the last library* (as before) or *Ask me*. With *Ask me*, LightCraft
  starts with a window listing your recent libraries: each one's name, the folder it is in and when you last opened
  it. One that is gone (moved, renamed, on a disconnected drive), can't be read or no longer holds a library is
  greyed out with the reason. Each one is checked in the background, showing *Checking…* until it answers (and that
  the drive isn't answering after 5 s), so the list never waits on a network share that is offline or a disk that
  has to wake up; File ▸ Open Recent Library uses the same answers (it greys out only libraries that are gone). Making the list only looks: it never locks or writes anything in your libraries, so one
  that another program has open is reported, with who has it, when you press **Open**, and the window stays. The
  last library is selected; **Open** (or Return, or a double-click) opens it,
  **Open Another…** asks for a folder, **New Library…** makes one, **Quit** (or Esc) quits. ↑ and ↓ move through
  the list.
- Holding Alt (Option on a Mac) as the window appears shows the same list once, over the library that opened: open
  another one from it, or press Esc to keep working where you were. On a Mac, an Option key held from before
  LightCraft starts is only noticed once it changes, so press it as the window appears.
- `--library` and `LIGHTCRAFT_LIBRARY` still decide the library when given, and `lightcraft-cli` never asks.
- File ▸ Open Recent Library ▸ **Remove from List** forgets one library, and the launch window's **Remove from
  List** does the same. Only the entry goes: the library and its photos stay where they are. Agents use
  `app.recentLibraries {remove: path}`; its list now also says when each library was last opened (`lastOpened`).
- Photos dropped on the window while the list (or the window of a library that couldn't open) is up wait for the
  library you open and are imported into it, as files given on the command line are; presets dropped then are
  refused with a note, since they belong to a library.

### Editing a smart album made from an album keeps the album
- A smart album saved from the view of an album stays limited to that album when you edit its rules and press OK. It used to match the whole library afterwards, because the rules dialog has no album field and replaced the album along with the rules.
### Drag photos from the filmstrip
- You can now drag a photo from the filmstrip onto the sidebar, as you can from the grid: onto an
  album, a folder or a keyword. Dragging a selected photo takes the whole selection; dragging one that isn't selected
  takes just that photo and selects it. In the Compare view, a photo that isn't selected is dragged alone and the
  selection, the active photo and the two photos on screen stay as they were. Clicking a filmstrip photo works as
  before: outside Compare it selects the photo, and in Compare a plain click puts the photo into the active side.
### Dropping photos on a folder or album that shares its name
- When photos are dragged onto a Folders row and another folder has the same name (two folders called `Home` on
  different disks), the label at the pointer, the Move Photos question and the message afterwards name the folder
  it is in, for example `Home (/Volumes/Archive/Photos)`, so you can tell which one you are about to move them
  into. Long parent paths keep their last names behind `…/`, unless that would read like another folder's path of the
  same name (a mirror on another disk): then the whole path is shown. A name that nothing else uses stays as it was, except that a network share's row is now
  named as the Folders list shows it (`photos on tokyo`) instead of by the last part of its mount path.
- Photos dropped on an album that shares its name with an album in another folder say so too (`Garden (Travel
  2026)`). Two albums with the same name in the same place have nothing to tell them apart by and keep the plain
  name.

### Panel layout messages, face recognition prompts and the second window's title in your language
- The messages shown when a panel layout cannot be restored or a panel move is not applied, the face recognition
  prompts (the banner and the name box's offer, with their buttons), the "Face recognition is on" and "is installed"
  messages, and the title of the second window are now in the interface language in all 17 languages. They stayed in
  English before. So did "The download failed", shown under a denoise or face recognition model's download when the
  download gives no reason.


## 2026-10-22

### Exports don't create a photo's sidecar
- Saving an export, a preset or any other file as `IMG_1.xmp` or `IMG_1.jpg.xmp` in the folder of the library photo
  `IMG_1.jpg` is now refused when no sidecar is there yet, as writing over an existing one already was. The library
  would have read the new file back as that photo's edits.

### Dodge and burn with the brush on the photo itself
- With the mask overlay hidden (O, View ▸ Show Mask Overlay or the eye beside Masks, or after a mask slider hid it),
  the brush stroke you are painting is no longer drawn in the overlay colour while you hold the button, so the photo
  stays visible as you dodge and burn (issue #2549). The stroke's effect still appears when you let go. With the
  overlay shown, the stroke is tinted as before; a colour overlay at opacity 0 leaves it untinted too.


### Reliability
- Opening a library no longer writes through a link left at the lock's owner note (`catalog.lock.owner`): the link
  is removed and a new note made in its place, so the file it pointed to is never changed (as for the catalog's
  temporary files, issue #1502).

### Keys reach text fields and the shortcut editor on macOS
- While a text field has focus, the native menu now lets go of the keys the field needs (`G`, the digits `0`–`9`,
  `P`, `U` and the like), and while Keyboard Shortcuts is recording a key it lets go of all of them (⌘, and ⌘Q in
  the LightCraft menu too), except the system's own items (Hide LightCraft ⌘H, Hide Others ⌥⌘H, Close Window ⌘W
  and Minimize ⌘M) and the two keys of the Help menu (⌘/ and F1). Before, the menu item kept its key on macOS, so the
  key could still run the menu command instead of reaching the field or the editor. The keys come back on the menu
  items when the field loses focus or recording ends; the menu's order, checkmarks and enabled state are unchanged.

### Find the grid shortcut in View
- Photo Grid and Square Grid now show the grid-cycle shortcut (G by default) in the View menu. The hints follow
  shortcut changes; clicking a menu row still selects that grid (issue #2500).

### GPX track logs read escaped values correctly
- Coordinates, elevations and times in a GPX track can contain XML character references such as
  `&#53;`. LightCraft now decodes those characters before parsing values, reports malformed or
  undeclared references as an error, and keeps CDATA literal. Auto-Tag places photos at the track
  position represented by their capture time.

## 2026-10-21

### Settings window
- Settings is one size on every tab and stays where it is as you go through them; before, each tab made its own
  size and the tabs moved under the pointer. The tabs stay in place while a long page scrolls (in a very short
  LightCraft window, where the page has too little room to scroll by itself, the tabs and the page scroll together), a
  page that goes on below shows a scroll handle without the pointer on it, and each page remembers where it was
  scrolled to, also when Settings is closed and opened again.
- Nothing in Settings runs off the side any more. A row with more choices than fit (the languages on General) wraps
  onto further lines and a long label wraps in its column, where both used to widen the window: in a small
  LightCraft window the labels and the Close button of General ended up off screen.
- The Slideshow tab is one column like the other tabs, with the slide preview first, instead of a column beside the
  preview that made Settings much wider on that tab. The preview stays above the page while it scrolls, so pointing
  at a template further down still shows it there (in a very short window it scrolls with the page instead). Pointing
  at a template no longer pushes its rows out from under the pointer, and the Export buttons wrap.
- A long folder (General ▸ Library ▸ Location, the folder Auto Import watches, the smart previews folder, a monitor
  profile's file) is shortened to its last folders ("…/Pictures/Library") on one line; rest the pointer on it for
  the whole path.
### One star rating control
- The stars in the bottom bar, the Info panel and the filter bar are now one shared control, and the badges on grid
  cells, filmstrip thumbnails and the Compare caption use the same drawing. Hovering a star in the filter bar now
  previews the rating like the others do, and with a star focused (Tab), ← and → lower and raise the rating.
### Progress looks the same everywhere
- Download, scan, denoise and activity progress now use one bar that fills the width it is given, with its text on
  it, and a gentle sweep while the amount of work isn't known yet. Waits that only said "Selecting…", "Waiting to
  start…" or "Installing…" now show a small turning arc beside the text, and a thumbnail still being made shows a
  quiet placeholder that fits the theme in the grid, the filmstrip and Compare. Screen readers hear a progress
  indicator with its percentage.
### Menu names that say what they are
- Switches in the menus are named for what they show, with the check mark saying whether it is on: Filmstrip ✓,
  Focus Point ✓, Clipping ✓, Photo Counts ✓, Mask Overlay ✓ instead of Show Focus Point ✓. No two commands share a
  name any more: View ▸ Detail is now Detail View (the Edit section Detail keeps its name), Window ▸ Edit is Edit
  Panel, and Compare Before and After is Before/After Side by Side, next to Split and Top/Bottom.
- The preview cache command has one name, Discard Preview Cache, in File ▸ Previews and in the toast that confirms
  it. Keyboard shortcuts are unchanged.
### Dropping files shows what will happen
- While files are dragged over the window, it dims and a card says what dropping will do: review the photos and
  folders for import, import the presets, or open the face model. If a dialog is open, the card asks you to close it
  first, and a drop is refused with a message instead of replacing the dialog. The same goes while an import is
  running, and in the web version.
- The import review of a finished scan (a drop, or File ▸ Import) no longer replaces a dialog you opened in the
  meantime; it opens as soon as that dialog is closed.
- Photos and folders dropped on the window now open the same import review as File ▸ Import, so you pick add, copy or
  move, the destination and a preset before anything is imported. Settings ▸ Import ▸ "Import dropped files without
  review" brings back the old behaviour of importing at once.
- In the web version, a dropped file the browser can't read, or that can't be imported, now shows a message instead of
  failing silently.
- For scripts: `ui.dropFiles` hovers files over the window or drops them, as the operating system does.
### Counting many smart albums is faster
- Counting every smart album at once (the numbers in the Albums list, and the `albums.list` command) now works out an
  album that other smart albums test (“album is …”) once for the whole library, instead of again for every album
  that tests it. On a synthetic library of 312,000 photos with 77 smart albums, 51 of them testing one shared album,
  the full count took about 14 s instead of about 23 s on a heavily loaded machine; the rest is the text rules of
  the albums themselves. The counts are the same as before, including for albums that loop through each other.

### Selection rings you can see in the light themes
- The active photo's white outline vanished on Silver and Paper's light grid, and told the active photo from the other
  selected ones by lightness alone. Now the active photo has a heavier ring (3 pt, with a thin line against the photo so
  it also shows on a white one) and the other selected photos a lighter one (1.5 pt), in colours each theme picks to
  stand out from its grid. The filmstrip rings its photos the same way, and so do Compare, Reference and Survey (where
  the ring is the only sign of the active photo) and the People view's chosen person.
### Smart albums open and count faster on large libraries
- A smart album that tests another smart album and also a file path or other text (for example "in Rejects, and the
  path contains …") now checks the quick rules first and the slow ones last, so most photos are ruled out before the
  other album is asked about them. The photos in the album and their order are the same, and the rules still show in
  the order you wrote them. (A library whose smart albums test each other in a circle, which LightCraft doesn't let
  you make, or test each other 32 deep, keeps testing the rules as written.) On a 312,000-photo test library, opening
  51 such albums in turn went from about 19 s to about 5.8 s (about 377 ms to 113 ms each; a busy machine,
  so rough numbers).
### A count never covers a name in the sidebar
- In the left panel, the count of a deeply indented row (a folder, keyword or album) no longer prints over its icon
  and name when the panel is narrow. It follows the name, wholly past the panel's edge, and scrolling sideways
  reaches it; so does the colour dot of a labelled folder. A row with room for its name keeps its count at the edge, as before.

### Dialog buttons say what they do
- The confirm button in dialogs now names its action instead of "OK": Create for a new album, smart album, folder or
  preset, Rename for photos, albums, keywords, people and presets, Create Stacks for Auto-Stack, Change Time for Capture Time,
  Copy and Paste for settings, and Export 12 Photos in the Export dialog. Rename Photos now explains what its Tags
  button lists and what happens on disk, instead of pointing at the button. Contact Sheet, Auto-Stack and
  Slideshow Export no longer say "1 photos" or "1 stacks". The Import review's Move button counts photos that are
  in Recently Deleted apart, since they are restored and their files stay where they are: "Move 2 Photos · Restore 1
  Photo".


### Sharper resampling in HDR and panorama merges
- Merged photos (HDR and panorama) are resampled with a sharper filter (Lanczos-3 instead of bilinear). Moving a
  photo by half a pixel used to lose a third of its finest detail; now it keeps most of it, without halos around
  hard edges such as clipped highlights.

### Scrolling past editing sliders
- Leaving a selected slider disarms its vertical mouse-wheel adjustment (issue #1355), even when the pointer leaves
  and comes back between two screen updates. Scrolling back over it now scrolls the panel instead of unexpectedly
  changing the photo. Click or drag the slider again to adjust it with the vertical wheel; keyboard arrow
  adjustments and an ongoing drag still work. A slider reached with Tab takes the arrow keys but not the wheel:
  click it to scroll-adjust it. Sideways scrolling still adjusts the slider under the pointer without selecting it
  first.

### A pinch no longer re-renders the photo at every zoom step
- Pinching the loupe now keeps the size the fit view is rendered at until the gesture is quiet, as it already did for
  the render sizes and windows. Before, each frame of a pinch asked for a new full-frame render at a new size.

### Fujifilm lens corrections that match the camera
- X-T4 raws now read the lens's own distortion and vignetting tables the way the camera does. Many photos from zoom
  lenses got no distortion correction at all (more than half of those checked from the XF16-55 and XF70-300), and
  corners came out up to 6.5 % brighter than in the camera's JPEG. The corrected photo now lines up with the camera's
  framing. Other recent Fujifilm bodies whose tables have nine knots are read the same way, but only the X-T4 was
  measured. Thumbnails are redrawn when the library next shows them.

### Reference view reliability
- Escape now leaves Reference for Detail without changing the active photo or its edits; the next
  Escape returns to the grid. A dialog, tool or Lights Out still handles Escape first.
- Opening or creating a different library clears the pinned reference. Photo IDs belong to one
  catalog; a pin no longer silently becomes another library's photo with the same numeric ID.
- Missing or Recently Deleted photos cannot be pinned. Deleting the pinned reference stops drawing
  it; re-entering Reference chooses a live photo, and restoring the deleted photo makes it pinnable again.
  In Recently Deleted, Set as Reference Photo is greyed out, and a refusal says to restore the photo.
- Switching library from Reference view goes to the grid instead of an empty Reference canvas. A refused
  or failed switch keeps both the reference and the Compare pair.
- `photo.setReference` refuses an `id` that is not a photo id (a string, a negative or fractional
  number) instead of pinning the active photo; only an absent or null `id` means the active photo.

### Edit Capture Time leaves a capture time it can't read as it was
- A capture time that is cut short, has seconds that aren't digits or has text after it (a Lightroom catalog's capture
  time is copied as it is) is no longer shifted into a clean-looking wrong time: Edit Capture Time leaves the photo as
  it was and says so, and its preview says the photo's time can't be read instead of previewing a wrong one.
  `photo.setCaptureTime` counts those photos as `unreadable`. Set date & time still repairs such a photo selected on
  its own; with other photos selected it says why it can't move them by the same amount. When the dialog can't make a
  change (a time that isn't a date, a shift past year 9999), it now says why and stays open, instead of closing.

### Month names in folder and file name templates
- Date templates take the month by name: `%B` gives January, `%b` gives Jan. Import ▸ Folders ▸ Custom template…
  `{date:%Y}/{date:%B}` files photos into 2026/January, 2026/February…, the layout many existing libraries already
  use. The names are English in every interface language, so folders don't change when the language does. Rename
  Photos and export file names take them too.

## 2026-10-20

### Windows: photos follow the monitor's colour profile
- On Windows, photos are now shown through the colour profile Windows assigns to the monitor the window is on
  (Settings ▸ System ▸ Display ▸ Color profile, Color Management, or a calibrator's profile), while no monitor profile
  is chosen in Settings ▸ Display. Before, they were shown as sRGB, so a wide-gamut monitor showed them oversaturated
  and a calibration was ignored unless you picked the profile by hand. Moving the window to another monitor, or
  changing the profile in Windows, is followed within about a second.
- While Windows manages colour itself (HDR on, or Auto Color Management on Windows 11), photos stay sRGB: Windows
  converts them for the monitor then. Settings ▸ Display ▸ **Use Screen Profile** goes back to this after choosing a
  profile by hand. See [display profiles](display-profiles.md).

### Smart albums check File Path rules faster
- A smart album rule on File Path (contains, doesn't contain, is, isn't, starts with, ends with, is empty, isn't
  empty) no longer builds new text for every photo. On a synthetic library of 312,000 photos, a File Path "contains"
  or "doesn't contain" rule took about 17 ms per pass instead of about 62 ms, around 3.5 times faster, with the same
  photos matched; about 6 ms of that is only reading the paths. Paths and rule text with letters beyond A–Z are
  compared as before.
## 2026-10-19

### AI Denoise on DirectX 12 with NVIDIA GTX 10-series cards
- AI Denoise on the GPU under DirectX 12 (the Windows default) gives the same result as on the CPU again on an NVIDIA
  GeForce GTX 1070. The driver ran the denoise network's output stores wrong, so every second pixel's colour
  channels came out shifted and the denoised photo was wrong without any error. Vulkan was not affected.

### The Before label stays in view when zoomed in
- Zoomed in, the "Before" label (\ key or the Before view) now sits at the canvas' corner instead of leaving the
  screen with the photo's corner (issue #2267).

### Synchronize Folder on a folder that is up to date
- When the scan finds nothing to import, relink, remove or read, the Synchronize Folder dialog says the library is up
  to date with the folder and has one Close button. It used to show four greyed choices, two of them still ticked,
  and a Synchronize button that started a job with nothing to do. Files it skipped (already in the library elsewhere,
  or unreadable) are still listed.

### Hidden Local locations come back one at a time
- With several locations removed from Local, "Show N hidden locations" now lists them, so each comes back on its own
  (Restore All is there too). The row used to bring every hidden location back at once (issue #2114).

### Rating in a filtered view keeps your place
- When a filter shows only one-star photos and you take the star off a photo, it leaves the view and the photo after
  it is now selected, so you carry on from where you were (issue #2312). The same goes for flags and colour labels,
  in Picks and in smart albums. Before, the photo stayed selected although the grid no longer showed it, and → or
  Auto Advance jumped back to the first photo of the view. With Auto Advance on, or with Shift+0–5, you land on the
  photo after it: no photo is skipped.

## 2026-10-18

### Thumbnails of Lightroom HDR and panorama merges in a second
- Grid and filmstrip thumbnails of DNGs that Lightroom merged (HDR, Panorama, HDR Panorama) now develop from the
  small copies of the raw image Lightroom keeps inside the file (256 to 2048 px), not from the whole merge. A 23836 ×
  7078 drone panorama's 512 px thumbnail took 38 s and now takes about a second, with the same look; your edits still
  apply. Loupe previews larger than the biggest copy, 1:1 views and exports still read the full image.

### Fewer false "not imported" warnings for Lightroom raws
- Importing raws with Lightroom XMP sidecars no longer lists settings Lightroom writes only for itself as adjustments
  that weren't imported: `CompatibleVersion`, the camera and lens profile checksums, the lens profile's name, file and
  setup, the crop's "constrain to image" option, and the lens profile's strength when it is at its default of 100 %.
  A raw sidecar used to name all of them for every photo. Settings LightCraft really doesn't apply, such as `Look`,
  are still reported.

### Fill Location from GPS: no more neighbouring town in City
- The offline place names list towns of 15,000 people or more, so a photo taken in a smaller one got the nearest larger
  town in City, sometimes another municipality (issue #2242). The Fill Location tooltip and the command help now say so,
  and `metadata.reverseGeocode` takes `maxDistance` (metres): past it, City stays empty (State and Country still fill)
  and the reply names the town it found as `nearestCity`.

### A preset that couldn't be saved is no longer lost silently
- When presets.json can't be written (a full disk, or the browser's storage quota), the new or changed preset now
  counts as unsaved, like other changes: the top bar warns, saving is retried, and quitting asks first (issue #2266).
  Before, the failure was only logged, and the preset could be lost without any warning.

### Portrait crop ratios stay portrait
- A crop ratio no longer flips to the photo's orientation (issue #2253): after X turns the crop portrait, picking
  another ratio (4 × 5, 2 × 3…) keeps it portrait, as in Lightroom, and a Custom ratio taller than wide gives a
  portrait crop.

## 2026-10-17

### Short Epson and Mamiya raws are refused
- An Epson ERF or Mamiya MEF whose raw strip runs past the end of the file is now an error on import (issue #2119).
  Before, the header probe trusted the declared strip length and sized its sample buffer from it, so a few KB of file
  could make it allocate gigabytes.

## 2026-10-16

### Synchronize Folder shows that it is listing the folder
- A folder with tens of thousands of photos, above all on a network share, can take minutes to list before
  Synchronize Folder checks a single photo. Its row in the activity stack used to sit at "0 of 64085" with an empty
  bar all that time, as if it had stopped. It now says how many folders and files it has listed so far, with a bar
  that moves on its own, and then counts the photos it checks. The dialog says the same.
- ✕ on the row stops the listing too, not only the checks after it.

### The photo grid keeps up with imports into a large library
- While photos are imported into a large library, the grid takes in each batch as it arrives instead of reading
  every photo of the view again: at about 180,000 photos a batch cost the window a tenth of a second, over and over
  for as long as the import ran. Changing one photo's capture time or crop is quicker too.

## 2026-10-15

### Photo Merge takes each photo once
- A merge that names the same photo more than once (through `ids` on the control channel, MCP or the CLI) now merges
  it once (issue #1947). Naming one photo three times no longer "merges" it with itself, and a long repeated list no
  longer decodes the photo again for every entry.

### Constrain Crop tightens the crop you see
- Geometry ▸ Constrain Crop now changes the crop itself, as in Lightroom (issue #1927). Ticking it, or moving a
  Geometry slider, Upright or a lens correction while it is ticked, shrinks the crop about its centre until the blank
  corners left by the correction are gone, and the Crop tool shows the tighter crop. Before, it only applied when the
  photo was shown outside the Crop tool, so in the Crop panel, where the checkbox is, nothing seemed to change. A crop
  you draw by hand is kept as drawn, and unticking keeps the tighter crop.

### Web: no backup that misses unsaved changes
- In the browser version, File ▸ Back Up Library… no longer reports success while a save is failing (storage full or
  cleared): the backup copies the stored library, so it left out the changes that couldn't be saved (issue #1508). It
  now refuses to start and says how many changes are unsaved and why. LightCraft retries the save with every change;
  back up once nothing is left unsaved.

### Web: an unreadable library is not opened
- In the browser version, a library file that exists but can't be read at start-up is no longer taken for a missing
  one (issue #1507). LightCraft used to open the library without it, showing an incomplete catalog that the next save
  would write over the stored copy. It now leaves the library unopened, saves nothing and says which file couldn't be
  read; reload the page to try again.
## 2026-10-14

### Model downloads time out on a name that doesn't resolve
- Model downloads (AI denoise, faces, SAM) give up on a server name that doesn't resolve within the connect timeout,
  and Cancel stops a download that is still looking the name up (issue #1532). The lookup used to block with no time
  limit, and each of a server's addresses then got the whole connect timeout of its own.

### Sony A7CR distortion correction

- Full-frame 3:2 Bayer A7CR raws with distortion Auto or forced Auto now use their embedded distortion table. The corrected render is closer to the camera JPEG geometry. Off files keep their original geometry. APS-C, aspect crops and lossless M/S remain excluded.
- The warp also supplies the geometry needed by the lens-aligned camera-look fit in PR #1586. The pairing code is a separate change. Details and numeric validation are in `docs/sony-lens-corrections.md`.

### Lightroom radial masks on the right side
- Radial masks brought over from Lightroom Classic (catalog import, XMP sidecars, edits inside DNGs and presets) now
  apply their effect inside the circle, as Lightroom does, instead of outside it (issue #1815). A mask inverted in
  Lightroom stays inverted, and older Radial Filters keep their Effect Outside / Invert Mask setting. Photos imported
  before this keep the masks they got: Photo ▸ Read Metadata from File reads a sidecar's again.

### Delete in an album removes from the album only
- In an album, the Delete key now takes the selected photos out of the album, as in Lightroom. It used to move them
  to Recently Deleted, so they also left their folder's view (issue #1805). They stay in the library and in their
  folder; Edit ▸ Undo puts them back in the album. Photo ▸ Remove from Album does the same and now shows the key.
  To delete photos from the library from an album, use Photo ▸ Delete Photo or right-click ▸ Delete Photo. In a smart
  album, Delete changes nothing: its rules decide what it shows.

### Local folders select a photo
- Showing a Local folder now makes its first photo the active one, as albums and the other sources do (issue
  #1447), or keeps the photo you had when the folder holds it. Before, the photo from the previous view stayed active:
  Photo Grid marked nothing, and Detail, the histogram and the photo info showed a photo that isn't in the folder.

### Nikon Z 6II, Z 8 and D5300 colour profiles
- Raws from the Nikon Z 6II, Z 8 and D5300 now start from a colour profile built from a few hundred photos from each camera, the same way the
  Sony and Fujifilm profiles were made, instead of fitting each photo's colour on its own. Colours that one photo
  shows only a little of come out closer to the camera's JPEG.

### Closer camera colour for lens-corrected raws
- Raws from cameras that correct their own JPEG for lens distortion (Panasonic and Fujifilm with that setting on, the
  Sony A7R IVA) now start from colours closer to the camera's. The starting look was being fitted against a JPEG that
  didn't line up with the raw near the edges. Photos you already imported pick this up when their previews are rebuilt.

### HDR merges of moving lights, and many brackets at once
- A star, the moon or a light that moves between the photos of a bracket (a night bracket on a tripod) no longer
  leaves a black hole where the middle photo had it clipped. With Deghost, the brightest photo's copy of it in a
  dark sky is now removed too.
- Photo ▸ Photo Merge ▸ HDR for Each Bracket Set merges every bracket among the selected photos with the last HDR
  settings, one after the other: each stack, and photos taken less than a second apart at different exposures.
  From the command line: `lightcraft-cli merge hdr-batch FOLDER`.
- Merging uses about a quarter less memory: three 67 MP raws need 3.8 GB instead of 4.8 GB.

### XMP sidecars catch up after a network share comes back
- With automatic XMP writing on, an edit to a photo whose folder can't be written (a network share that dropped
  out, a folder that went read-only) used to be written only to the library, with a line in the log, so the sidecar
  stayed behind the edit. LightCraft now says once that the sidecars are waiting, tries again every few seconds once
  the files are reachable, and says when they're written. `library.retrySidecars` does it on request.

### Reliability
- A damaged or crafted raw can no longer make LightCraft set aside gigabytes of memory before finding out the file is
  bad (issue #1512): a compressed PEF whose data is too short for the size it declares, an uncompressed TIFF-based
  raw whose tile claims far more samples than it holds, and an ARW6 with a broken tile are refused first. All 74
  files in the test corpus still decode.

### Reliability
- Thumbnails and previews are written to the cache on disk by one background writer instead of a new thread for
  each image (issue #1522). When renders come in faster than the disk takes them, at most 128 MB of images wait;
  past that a write is skipped (the image stays in memory and is written by a later render), so a slow disk can't
  pile up threads and memory.

### Reliability
- Saving the catalog and resuming a model download no longer write through a link left at the temporary file's
  name (`catalog.snap.tmp`, `model.onnx.part`): the link is removed and a new file made in its place, so the file it
  pointed to is never changed (issue #1502).

### Reliability
- A photo whose file was replaced (Reload Changed Files after an outside edit) no longer keeps the faces recognition
  made from the old pixels: they are made again from the new file, and a scan still busy with the old file when it
  changed doesn't put its faces on the new one (issue #1524).

### Settings stays put when you change tabs
- The Settings window keeps its top edge and width when you switch tabs, so the tab row no longer jumps around
  under the pointer. It used to be centred, so every tab of a different height moved it.

### Layout image over the loupe
- View ▸ Loupe Overlay ▸ Choose Layout Image… draws a PNG with transparency over the photo in the loupe, such as
  a magazine cover's masthead and cover lines, so you can frame a shot for the page it is made for. It fits inside
  the photo, follows zoom and pan, and ⌥⌘O shows or hides it. The same menu sets its opacity and a matte that
  darkens the photo outside it. The layout stays chosen across launches.

### Smaller JPEG exports
- JPEG exports are 12–14 % smaller at the same quality, with exactly the same pixels (issue #1373): each file now
  carries Huffman tables made for its own picture instead of the standard example tables. Every viewer opens them
  as before.

### Reliability
- A photo whose Defringe amount was stored far past the sliders' 20 (in a damaged catalog or sidecar) no longer
  crashes LightCraft each time it renders: the amount acts as 20.

### Lens names for Nikon DSLR raws
- Photos from Nikon DSLRs like the D700, D750, D80 or D5300 now show which lens they were taken with, for example
  "24.0-70.0 mm f/2.8". Those cameras don't write a lens name, so the lens field was empty and you couldn't filter or
  search by lens. LightCraft now reads the lens range Nikon keeps in its own maker note, the way Lightroom shows a
  lens it doesn't have a name for. Newly imported photos show it.

### Import when a card is connected
- Settings → Import → Cameras and Cards has an option to open the import review as soon as a camera or memory card
  is connected, the same review File → Import from Device opens. It's off by default. A card that's already in when
  LightCraft starts doesn't ask, and if a dialog is open or an import is running it waits until that's done.

### Local folders
- Photos deleted (or moved) by another app no longer stay in a Local folder, failing to load and logging an error
  each time you look (issue #1392): the next time you browse the folder, in a later session, they're left out. A photo you had rated, flagged or
  edited stays, marked missing, so its changes aren't lost; photos in My Photos are unchanged.

### Full-resolution panoramas
- Panorama and HDR Panorama merges keep the resolution of their photos instead of stopping at 40 megapixels
  (issue #1455), up to 512 MP and 65,000 pixels on the long edge, as in Lightroom. When the full size needs more
  memory than the computer has, the panorama is made as large as fits (five 51 MP photos: up to 166 MP with
  16 GB, 355 MP with 32 GB) and the message says so. The merge also frees each photo's pixels once they are
  projected, so it needs less memory than before at any size (an 81 MP stitch: 4.4 GB instead of 6.2 GB).

### Nikon D600, D610 and D5000
- Raws from the Nikon D600, D610 and D5000 no longer show a bright line down the right edge (issue #1627). Their
  sensor data runs a few columns of filler past the picture, which stayed in the image; these bodies now develop
  at the camera JPEG's size (6016 × 4016 for the D600 and D610, 4288 × 2848 for the D5000, as the D90 already
  did). On the D600 and D610 the white level is also measured inside the picture, so the filler can't pass for
  clipped highlights.

### More Canon EOS R6 Mark II C-RAW files decode
- Some EOS R6 Mark II C-RAW files (about 1 in 50 in one photographer's library) opened as their preview with
  "Canon CRX adaptive QP value 172" or "175". They now decode from the sensor data. Photos imported before this
  switch over with Photo ▸ Reload from Disk.

### Nikon raws without coloured edge bars
- Raws from older Nikon bodies no longer show a thin coloured or dark bar along one edge (the right edge, or the top
  of a portrait photo): masked sensor columns were being developed as part of the picture (issues #976, #914,
  #1236). The D7000, D5100, D800, D800E, Df, D4, D3, D3S, D700, D300, D300S, D600, D610, D810, D4S, D5600,
  Coolpix A, D3000, D40X, D60, D80, D2Xs, D200, D3200, D3100, D5000, D40, D50, D70, D70s and Nikon 1 V1 now show
  the area the camera itself frames, a few pixels smaller than before.
### An empty library leads to Import
- A library with no photos now opens on the Photo Grid with the sidebar (Square Grid stays as it is), instead of a
  Detail view that pointed at an empty grid. The grid and Detail view offer Import Photos… and, in the desktop app,
  Import Lightroom Catalog… as buttons, and say that photos (and, in the desktop app, folders) can be dropped into
  the window. The shortcut they show is the one Import Photos… has now.
- The top bar has an Import button, beside Export, in every library.

### Reliability

### One kind of button
- Dialogs' buttons and About's Discord link are drawn by one button component: primary (filled with the accent),
  secondary, danger (filled red) and ghost, in three sizes, with an icon or without, and an icon alone (named for
  screen readers). Each darkens a little when hovered and more when pressed, shows a ring where the keyboard is, and
  its text reads at 4.5:1 or more on its fill in every theme. Dialog buttons are 30 points tall, like the welcome's.

### A dialog's confirm button says it's the next step
- In dialogs, the confirm button is on the right with Cancel to its left, and filled with the accent when it makes or
  brings in something (Import N Photos, Merge, Save, Create, Download…). Delete and Remove stay plain, so a destructive
  choice is never the eye-catching default.
### Straight brush strokes with ⇧-click
- With the brush, click once, then ⇧-click somewhere else: the brush paints a straight line between the two, and
  further ⇧-clicks carry on from the last point, as in Lightroom. Change the brush size, feather or flow before the
  ⇧-click and the line tapers from the old values to the new ones. The whole line is one undo step.
  `mask.brushStroke {connect: true}` does the same for agents.

### Local folders you browse are kept
- Browsing a folder under Local now records it as browsed, so the photos in folders you use aren't forgotten after
  the number of days set in Settings → Performance. Only the `library.browse` command recorded it before, so a
  folder browsed every day from the sidebar was forgotten anyway and read again from scratch on the next visit,
  which is slow on a network share.

### Hard-edged Heal and Clone spots
- A spot with Feather at 0 no longer draws black pixels exactly on its radius. This also fixes spots at 0% Opacity
  changing pixels along their edge. The hard edge now leaves pixels on the radius unchanged.

### Search finds people
- Typing a person's name in Search Photos finds the photos with that named face (issue #1449).
- Quotes keep several words together: `person:"Jane Doe"` finds exactly that person, `"jane doe"` those words side
  by side, and `keyword:"new york"` a keyword of two words.

### Where the camera focused
- **View ▸ Show Focus Point** (⌥F) draws the autofocus frame the camera recorded over the photo, in Detail, Compare
  and Survey, following rotation and crops; the Info panel says how it focused (`AF-C · Wide · Animal Eye Tracking`).
  Lightroom has no such view.
- **Z zooms to it**: Toggle Zoom centres on the focus point, so checking that the subject is sharp is one key
  (View ▸ Zoom to Focus Point turns this off). Photos without a point zoom as before.
- Sony α1 (ILCE-1) and A7S III (ILCE-7SM3) raws for now; manual-focus shots and Wide shots that only report the frame
  centre show nothing. More cameras will follow. Details: [focus-points.md](focus-points.md).
- **Photos already in your library get theirs in the background**: LightCraft reads just the few kilobytes of each
  file that hold the focus point (about 100 photos a second from a spinning disk), with a *Reading focus points* row
  in the activity stack you can stop. Photo ▸ Read Focus Points runs it by hand.

### Cleaner HDR merges
- Bright windows in an HDR made with Photo ▸ Photo Merge ▸ HDR no longer take a magenta tinge from the brighter
  photos along the areas they clipped: right next to a clipped area, a photo's colour isn't used. It showed most when
  the highlights were darkened or saturated.
- The same photos give the same HDR whatever order they are selected in; before, a different order could change a
  few pixels.

### HDR merges keep the white balance
- An HDR merged from raws that LightCraft colours from the camera's JPEG (CR2, NEF, ARW…) now opens at the same
  As Shot temperature and tint as its photos, and changing the white balance no longer turns it green. Sony ARW
  merges used to open at about 7000 K / +115 where the photos show 4000–5300 K / +8. HDRs merged before this keep
  their old white balance; merge them again.

## 2026-10-13

### Edits keep up in a large library
- A small edit (a keyword, a rating, a flag, a slider) no longer has the grid's list worked out again from every
  photo: only the photos the edit touched are asked about again and put at their place in the order. In a library of
  67,000 photos that step went from 37 ms to under 1 ms. Still worked out whole: an edit to more than 256 photos, a
  change to an album, a stack or the keyword list, an album shown in its own order, Recently Added, Missing Photos,
  and a view that asks about "the last N days".
- With no filter set, the count of the source is taken from the list instead of from a second pass over the library.
- The side panels keep a tally an edit cannot have changed: a rating or a slider changes none of the folders,
  keywords, dates, people and counts; a keyword leaves the folders, the dates and the counts as they are. With both
  side panels open, the frame after a keyword went from about 156 ms to about 54 ms in that library (2 ms with the
  panels closed, from 71 ms).
- The count beside each smart album in the left panel follows an edit the same way: the album asks only the photos
  the edit touched, where it asked every photo of the library after each edit (about 25 ms an album in that library).
  With that, the frame after a keyword is at about 29 ms there. New rules, a change to an album or the keyword list,
  an edit to more than 256 photos, and each new minute for an album that asks about "the last N days" are still
  counted from every photo.
- Still tallied from every photo on each edit: the filter bar's lists of cameras, lenses and keywords.

### A view can hold its photos until you refresh it
- View ▸ Hold View Until Refreshed (off unless you turn it on; kept with the library). With it on, an edit that makes
  a photo stop matching the view keeps it in the view: a keyword given in a smart album of photos without keywords,
  a flag taken off in Picks, a rating that falls under the filter bar's. With it off, as before, the photo leaves the
  grid at once and every photo after it moves up. View ▸ Refresh View (also in the sort menu while the hold is on),
  Reshuffle, another source, filter or sort, or new rules for the smart album shows what matches now.
- Photos that start matching still appear at once, and deleting a photo or taking it out of the album or folder
  shown still takes it out of the view.

### The window opens at the size you left it
- LightCraft's window opens at the size it had when you last used it, and zoomed (macOS) / maximized if it was: it
  used to open at 1600 × 1000 every time. Double-click the top bar to zoom the window, quit, and it is zoomed when
  you come back; un-zoom it and it goes back to the size it had before.
- Full screen is not remembered (the app opens as a window), and neither is where on the desktop the window was:
  the system places it.
- On macOS the system does not tell LightCraft when a window is zoomed from the Window menu or resized after a
  zoom, so it goes by the window's size, and when in doubt takes the window for a window of the size it has: it
  comes back as large as you left it, but not as a zoomed window. That is so after a zoom from the Window menu
  with the Dock at a side of the screen, or of a window already as wide as the screen. Zoomed by a double-click
  on the top bar it comes back zoomed. The other way round, a window you drag out until it is as wide as the
  screen and nearly as tall counts as zoomed.
- For scripts: a size set with `ui.resize` is remembered like any other, so a script that needs a size sets it.

### Keyword List: a keyword shows that one inside it is applied
- In the Keyword List (right panel ▸ Keywords), a keyword whose box is empty now carries a dot in it when the
  selected photos have a keyword inside it, at any depth: with `Places › Italy › Rome` on a photo, `Places` and
  `Italy` show the dot and `Rome` the tick. A folded keyword no longer hides that something in it is applied.
  Resting on the box names what is inside.
- The tick and the dash are as they were, and are about the keyword itself: all of the selected photos have it, or
  some do. A keyword the photos have themselves shows that, not the dot. A click on the box still gives the
  keyword itself to the selected photos.

### HDR merges keep the lens correction
- An HDR made with Photo ▸ Photo Merge ▸ HDR now keeps the lens correction the photos carry (Sony's, Panasonic's
  and Leica's, and the one some DNGs carry, e.g. from drones): Optics ▸ Enable Lens Corrections straightens it as it
  does the photos it was made from. HDRs merged before this don't have it; merge them again.

### Portable mode
- A file named `portable.txt` next to the LightCraft program makes it keep its settings, logs and downloaded
  models in a `LightCraftData` folder beside it instead of the per-user settings folder (issue #1200), as
  PhotoCraft does. If that folder can't be written, the usual folder is used and the log says why.

### Reliability
- Importing a preset zip that unpacks to far more than any preset (a "zip bomb") is refused with a message naming
  the file, instead of asking for gigabytes of memory, which could end LightCraft.

### Reliability
- A crop of no size (from an agent or a sidecar) no longer gives the Detail view a photo of no size to lay out
  (its width over its height was 0 / 0, which stopped debug builds); it shows as a single pixel's colour.

### Reliability
- A crop a hair wide (or of no size), set by an agent or read from a sidecar, no longer crashes or hangs the
  render with Clarity, Dehaze, Texture or Highlights / Shadows on: their blurs are held to the picture's size.
- A crop of no size with healing spots or masks no longer crashes the render: samples outside the picture read its
  edge.

### Reliability
- A library whose catalog file was damaged (or edited by hand) in a way that still reads no longer crashes
  LightCraft when it shows a stack, or hangs it when a folder is moved: on opening, records are put back under
  their own ids, stacks keep only photos that exist (at least two), folders inside themselves move to the top,
  and new photos never reuse an id in use. What was repaired is written to the log.

### Nikon lens corrections
- Raws from the Nikon Z 6II, Z 8 and Z f now open with the camera's own distortion correction, as Lightroom applies a
  built-in profile: straight lines near the edges stay straight and the framing matches the camera's JPEG. It works
  with Z lenses and with F-mount lenses on the FTZ adapter, follows the in-body stabilisation's shift of the optical
  centre, and can be turned off under Lens Corrections. Vignetting correction from the file comes later.

### Thumbnail cache stays in its folder
- The thumbnail cache no longer follows a shard folder (`thumbs/ab/`) that is a symlink or junction to somewhere
  else (issue #1504): trimming or clearing the cache can't delete files outside it, and thumbnails are never read
  from or written through such a link.

### Synchronize Folder asks a network share less
- A Synchronize Folder scan no longer asks the disk about every photo of the library in the folder (is its file
  there, has it a sidecar): it has just listed the folder, and reads the answers off the listing. On a network share
  each of those questions is a round trip. Measured on a hard disk over SMB, for folders of 1,600 to 2,400 photos
  that had not changed: 4.6 to 7.1 s of questions after the listing (about 3 ms a photo), now under 0.05 s. The
  listing itself took 4 to 5 s and is as it was.
- The disk is still asked wherever a listing can't be sure what a file system would say: about a name that is
  there in another case, a link, a name Windows reads another way (`aux.jpg`, a `~`, a `:`, a dot or a space at the
  end), a folder the scan did not walk to its end, and, in a folder that holds any name with a character outside
  ASCII (an umlaut, a curly apostrophe, a dash), about every name the folder does not list. Names it does list are
  answered there too. In such a folder the scan saves less, and its checks now start after the listing where they
  ran beside it.
- A photo the folder lists is there for the scan, also where its file could not be opened by its path (a folder
  that may be listed but not searched, a path longer than the system takes): such photos used to be listed as
  missing. And a share that drops after the scan has listed a folder no longer turns that folder's photos into
  missing ones.

### Large libraries: Synchronize Folder, import and sidecars start sooner
- Opening Synchronize Folder, an import review or an import no longer makes a table of every photo in the library
  before it looks at the first file. In a library of 3.5 million photos that held the window for about 6 s before
  a Synchronize scan began (now under 1 s), 2.6 s before an import review (now 0.05 s) and 1.7 s before an import
  (now 0.05 s). The scan looks up only the files it finds and the contents it reads, on its own thread.
- Writing one photo's XMP sidecar (Save Metadata to File, or automatically after an edit) no longer works out who
  owns every shared sidecar name in the library: 2.4 s a write in that library, now under 1 s.
- Nothing else changes: the same files are found new, already there, moved or missing, and sidecars get the same
  names.

### Faces load faster in the People view
- Faces on unedited raws are cut from the camera's preview inside the file instead of developing the raw (issue
  #1440): a Sony α1 face reads 8 MB instead of 70 MB, an A7S III face about 1 MB instead of 24 MB, several times
  faster on an external hard disk. A face whose preview is too small, and any edited photo, is still developed from
  the raw, so it keeps its look.
- Raws whose preview is in Adobe RGB (Sony α1 among others) no longer read the whole file for their grid thumbnail:
  only the header and the preview are read, as for other raws.
- Faces are kept in the thumbnail cache at a few fixed sizes, so moving the thumbnail slider no longer renders them
  all again.

### Reliability
- A raw file whose black level is stored as "not a number" (or infinity), or a floating-point DNG with such
  samples, no longer crashes LightCraft at 1:1 or on export: the bad values read as black.

### Reliability
- Exporting a photo more than 65,535 times as long as it is wide (a one-pixel strip), or opening the Export dialog's
  watermark preview for it, no longer crashes LightCraft; the export is capped at the encoders' 65,535 pixels.

### Reliability
- Dragging a Tone Curve point that sits very close to its neighbours (two clicks side by side, or a curve from a
  sidecar, a preset or an agent) no longer crashes LightCraft; the point stays between them. Curve points sent to
  `develop.curve` are kept in order, and a sidecar curve with a "NaN" or "inf" point is ignored.

### Reliability
- A demo photo whose stored size was damaged (a width of 4294967295) is rendered at its own size instead of
  one that overflows.

### DNG camera identity
- DNG export and Convert to DNG no longer duplicate the camera make when the model already
  contains it: `Canon EOS R6` stays `Canon EOS R6`, rather than `Canon Canon EOS R6` (the camera
  identity subissue in #840). The separate Make and Model tags keep their original values.
  This does not change colour matrices, camera calibration or RAW noise rendering.
### Building from source
- The English and Simplified Chinese build guides now describe command-local job and release
  profile overrides for memory-constrained builders (issue #1196), including why packaging
  recipes can override inherited settings. Default release profiles and application behavior
  are unchanged; the guidance does not guarantee a build will fit every 16 GB machine.
### Lightroom catalog import
- A Lightroom Classic catalog whose changes are all still in its `.lrcat-wal` file (a new catalog Lightroom has
  open and hasn't written back yet) imports instead of failing with "unsupported SQLite schema format".

### Reliability
- An export enlarged far past the photo's size (20000 typed for 2000 with Don't Enlarge off) no longer ends
  LightCraft: an enlargement stops at 512 megapixels, as in Lightroom, and one that needs more memory than the
  computer has is refused with the size it would take, before it starts.
### Confirm matching faces together
- On a person's page, the ↑ control on a suggested face confirms it and all shown suggestions with equal or higher
  similarity as one undo step. Hover or focus that face and press A for the same action; hidden suggestions stay hidden.

### Noise reduction looks the same at every size
- Luminance and colour noise reduction no longer look much stronger in grid thumbnails and the filmstrip than in
  the loupe (issue #960). Their radii are now set in the original photo's pixels, so a thumbnail, the fitted loupe
  and a smaller export look like the full-size result scaled down; 1:1 views and full-size exports are unchanged.
  Like in Lightroom, judge noise reduction at 1:1: at small sizes the noise it removes is mostly averaged away.
  Cached thumbnails are redrawn.
- When the graphics driver resets the GPU while LightCraft is exporting or rendering (issue #1337, seen on Windows
  with NVIDIA), LightCraft still stops, because its window can't keep drawing on the lost device, but it now says
  "LightCraft stopped unexpectedly" and why, instead of "could not open its window", and the next start renders
  on the CPU and explains how to turn the GPU back on.

### Resetting a slider
- Resting the pointer on a slider's name or its thumb now says that a double-click resets it ("Exposure — double-click
  to reset it"), in every language (issue #1436).
- A double-click on the thumb is a single step in History and Undo. Its first click no longer nudges the value to
  where the pointer landed, and a single click on the thumb leaves the value alone.

