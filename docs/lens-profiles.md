# Lens profiles from a lens database

> **Last reviewed:** 2026-10-13 · **Last updated:** 2026-10-13 · **Change:** minor (the database is downloaded from inside the app and kept in the settings folder; photos already in the library gain their profile; corrected images fill the frame)

Raws that carry no lens correction of their own (DNG `OpcodeList3`, or a camera correction a raw reader converts,
see [sony-lens-corrections.md](sony-lens-corrections.md)) are corrected from the **lensfun project's lens database**.
The app does not ship it: the data is licensed CC BY-SA 3.0 and stays a separate download, which the app fetches
when you ask. Only the database's XML files are used, never lensfun's code. Adobe LCP profiles are never used.

## Getting the database

- **In the app:** open a raw photo, open **Lens Corrections** (Optics in the Modern style) and click **Download
  Lens Profiles**, or choose Photo ▸ Download Lens Profiles (lensfun). The download is about 5 MB (56 XML files from
  `github.com/lensfun/lensfun`, `data/db`), runs in the background, shows in the activity stack and can be
  cancelled. Nothing about your photos is sent. The files go to `<settings>/lensfun` (macOS:
  `~/Library/Application Support/LightCraft/lensfun`), with an `ABOUT.txt` naming their source and licence.
  Download again to update; a download that fails leaves the installed files alone.
- **Your own copy:** point `LIGHTCRAFT_LENSFUN_DIR` at a directory of lensfun XML files (a checkout's `data/db`, or
  what a Linux distribution installs under `/usr/share/lensfun/version_1/`). The variable wins over the settings
  folder, and the app then offers no download.
- **From a script:** the engine commands `optics.downloadLensProfiles` and `optics.lensProfilesStatus`, through the
  control channel's `engine.execute` or MCP. The CLI reads the same settings folder as the app.

Invalid files are skipped with a warning in the log. Without a database nothing changes.

## Photos already in the library

A photo gets its lens data when it is imported. For raws catalogued before the database was there:

- When a download finishes, and whenever a library is opened with the database installed, the files of raws without
  lens data **whose lens the database names** are read in the background (activity stack, cancellable) and gain
  their profile. Raws the database can't help are not read again at every start.
- A raw opened in Develop is looked up once more on its own, so the panel's "no profile for this lens" is said only
  after its file was read; and any decode that finds lens data records it.
- As at import, **Enable Lens Corrections is switched on** for photos still at their import settings. A photo you
  have edited only gains the data: its switch stays as you left it. This is bookkeeping, not an undo step; untick
  the switch (and sync it to other photos) to turn the correction off.
- Photo ▸ Reload from Disk does the same for the selected photos.

Under the switch the panel names the lens and what its data corrects (distortion, colour fringing, vignetting).
An amount slider with nothing to scale is greyed out: many zooms in the database have distortion data only.

## What is applied

The lens is identified from EXIF: camera make / model (for the mount and crop factor; the 35 mm equivalent focal
length is used when the camera isn't listed) and the lens model string. Lens names match on their words; every
number (focal lengths, apertures) in the database name must appear in the EXIF name, and the lens maker must agree
when the file names one.

| Correction | Models | Interpolated over |
|---|---|---|
| Distortion | `poly3`, `poly5`, `ptlens` | focal length |
| Lateral CA (TCA) | `linear`, `poly3` | focal length |
| Vignetting | `pa` | focal length, aperture (in stops), focus distance (nearest calibrated, by 1/d; infinity when unknown) |

Fisheye and other non-rectilinear lenses get no distortion correction. The profile is converted by least squares
into the DNG `WarpRectilinear` / `FixVignetteRadial` polynomials, so it goes through the same path as embedded
corrections: **Enable Lens Corrections** and its Distortion / Vignetting amounts, the CPU and GPU geometry
stage (one resample), orientation handling and DNG export. A conversion whose radial error exceeds 0.001 of the
half diagonal, or that would fold the image, is not applied.

Normalisation follows lensfun's documentation: distortion and TCA radii are 1 at half the shorter side of the
calibration frame (its `cropfactor` and `aspect-ratio`), vignetting radii 1 at its corner; the optical centre is
the image centre (lensfun's `<center>` offsets are not read).

**The corrected image fills the frame.** The database's distortion models keep the scale at their own reference
radius, not at the edge of the frame the photo was taken on, so a correction applied as it stands leaves blank
canvas along the edges it pulls inward (a full-frame zoom's wide end on an APS-C body: a border about 20 px wide at
18 MP) or crops pixels the lens recorded. The corrected image is therefore zoomed about its centre until an edge
point just reaches the recorded frame: in for the first case, out for the second. The zoom is part of the warp, so
the Distortion amount scales it with the correction (above 100 % an edge can show blank canvas again).

## Limits

- Only raws: rendered images (JPEG, HEIF, TIFF) are often already corrected in camera.
- The focus distance is not read from maker notes yet: vignetting uses the farthest calibrated distance.
- No manual make / model picker yet; matching is automatic.
- The database's file list is fixed in the app (`crates/engine/src/cmd/lens_profiles.rs`, as of 2026-10-13): a file
  the lensfun project adds later is not fetched until it is named there; one it drops is skipped.
- Checked on real photos only this far: 20 Canon EOS Rebel T5 frames with the EF 28-135mm f/3.5-5.6 IS USM (the
  database has distortion data only for it) are matched, corrected and fill the frame at every focal length shot.
  The size and direction of the correction have **not** been compared with another renderer or a test chart; the
  tests use synthetic profiles and grids.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-13 | minor | LENS.3: the Lens Corrections note names the lens and what is corrected, and offers the download; a library opened with the database installed is caught up in the background |
| 2026-10-13 | minor | LENS.2: the database is read from `<settings>/lensfun`; `optics.downloadLensProfiles`, `optics.lensProfilesStatus`, `optics.findLensProfiles`; a decode records lens data the catalog lacks |
| 2026-10-13 | patch | LENS.1: corrected images are zoomed to fill the frame |
