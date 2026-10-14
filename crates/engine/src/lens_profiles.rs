//! Lens profile corrections from a lens database ([`lightcraft_lensdb`]).
//!
//! When a raw carries no lens correction of its own (DNG opcodes, or a camera correction a raw reader converts,
//! [`crate::files::embedded_lens`]), the lens named in its EXIF is looked up in a lensfun-format database. The
//! profile is converted to the same [`lightcraft_develop::EmbeddedLens`] form, so "Enable Profile Corrections", its
//! amount sliders, the GPU path and DNG export treat both alike.
//!
//! The database (the lensfun project's `data/db`, CC BY-SA 3.0) is never bundled with the app. It is read from the
//! directory in `LIGHTCRAFT_LENSFUN_DIR`, else from the folder the host names with [`use_dir`]: the apps name
//! `<settings>/lensfun` ([`use_default_dir`]), which is also where `optics.downloadLensProfiles`
//! ([`crate::cmd::lens_profiles`]) puts the files it fetches. A library that names no folder reads the variable
//! only, so tests stay hermetic. Without a database nothing changes.
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use lightcraft_develop::EmbeddedLens;
use lightcraft_lensdb::{LensDb, Shot};

/// The environment variable naming the lens database directory.
pub const ENV: &str = "LIGHTCRAFT_LENSFUN_DIR";
/// The credit the lensfun database asks for, shown wherever its download is offered.
pub const ATTRIBUTION: &str = "Lens profiles: the lensfun database (lensfun.github.io), CC BY-SA 3.0";

/// How a lens-profile download ended.
#[derive(Clone, Debug, PartialEq)]
pub enum DownloadOutcome {
    /// Done: this many lens profiles in this many files.
    Done {
        lenses: usize,
        files: usize,
    },
    /// Failed, in words the user can act on.
    Failed(String),
    Cancelled,
}

struct Store {
    /// The folder the host named ([`use_dir`]); read when [`ENV`] is unset, and where downloads go.
    dir: Option<PathBuf>,
    /// The database as last read (`None` inside: none configured, or it holds no lenses), until [`reload`].
    loaded: Option<Option<Arc<LensDb>>>,
    /// Counts every change of what [`database`] may return, so callers can drop what they derived from it.
    generation: u64,
    /// Where downloads come from instead of the lensfun project's own address ([`use_source`]).
    source: Option<String>,
}

static STORE: Mutex<Store> = Mutex::new(Store { dir: None, loaded: None, generation: 0, source: None });

/// Lens data a background lookup found for photos of the library with generation `library`.
pub(crate) struct Found {
    pub library: u64,
    /// (photo, the file it was read from, its lens data)
    pub lenses: Vec<(lightcraft_catalog::PhotoId, String, EmbeddedLens)>,
}

#[derive(Default)]
struct JobState {
    downloading: bool,
    last: Option<DownloadOutcome>,
    /// The last outcome, until the UI has shown it ([`Jobs::take_outcome`]).
    unreported: Option<DownloadOutcome>,
    /// A background lookup of the library's photos is running.
    scanning: bool,
    /// What the last lookup found, until the session takes it ([`crate::Session::finish_lens_scan`]).
    found: Option<Found>,
}

/// A session's lens-profile work in the background ([`crate::cmd::lens_profiles`]): the database download and the
/// lookup of the library's photos. The database itself is the process's; these belong to the session that started
/// them.
#[derive(Clone, Default)]
pub struct Jobs(Arc<Mutex<JobState>>);

impl Jobs {
    fn state(&self) -> MutexGuard<'_, JobState> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether a download is running.
    pub fn downloading(&self) -> bool {
        self.state().downloading
    }

    /// How the last download ended (kept until the next one ends).
    pub fn last_outcome(&self) -> Option<DownloadOutcome> {
        self.state().last.clone()
    }

    /// A download that ended since the last call (the UI shows it once).
    pub fn take_outcome(&self) -> Option<DownloadOutcome> {
        self.state().unreported.take()
    }

    /// Whether a background lookup of the library's photos is running.
    pub fn scanning(&self) -> bool {
        self.state().scanning
    }

    /// Mark a download as running; `false` when one already is.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn begin_download(&self) -> bool {
        !std::mem::replace(&mut self.state().downloading, true)
    }

    /// Record how the running download ended (`None`: it could not start).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn end_download(&self, outcome: Option<DownloadOutcome>) {
        let mut st = self.state();
        st.downloading = false;
        if let Some(outcome) = outcome {
            st.last = Some(outcome.clone());
            st.unreported = Some(outcome);
        }
    }

    /// Mark a lookup as running; `false` when one already is.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn begin_scan(&self) -> bool {
        !std::mem::replace(&mut self.state().scanning, true)
    }

    /// The running lookup ended with `found` (`None`: it could not start).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn end_scan(&self, found: Option<Found>) {
        let mut st = self.state();
        st.scanning = false;
        if found.is_some() {
            st.found = found;
        }
    }
}

fn store() -> MutexGuard<'static, Store> {
    STORE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The directory in [`ENV`], when it names one.
pub fn env_dir() -> Option<PathBuf> {
    std::env::var_os(ENV).filter(|v| !v.is_empty()).map(PathBuf::from)
}

/// `<settings>/lensfun`: where the apps keep the lens database.
pub fn default_dir() -> Option<PathBuf> {
    crate::config::config_dir().map(|d| d.join("lensfun"))
}

/// Read lens profiles from `dir` when [`ENV`] is unset, and download them there. `None`: the variable only.
pub fn use_dir(dir: Option<PathBuf>) {
    let mut st = store();
    if st.dir != dir {
        st.dir = dir;
        st.loaded = None;
        st.generation = st.generation.wrapping_add(1);
    }
}

/// [`use_dir`] with [`default_dir`]: what the desktop app and the CLI call at start.
pub fn use_default_dir() {
    use_dir(default_dir());
}

/// Download the database from `base` (a mirror of lensfun's `data/db`, or a test server) instead of the project's
/// own address; `None`: the project's.
pub fn use_source(base: Option<String>) {
    store().source = base;
}

/// The address set with [`use_source`].
pub fn source() -> Option<String> {
    store().source.clone()
}

/// Whether the folder profiles are read from exists (a cheap check that reads no database).
pub fn folder_exists() -> bool {
    dir().is_some_and(|d| d.is_dir())
}

/// The folder the host named ([`use_dir`]): where a download goes.
pub fn host_dir() -> Option<PathBuf> {
    store().dir.clone()
}

/// The directory profiles are read from: [`ENV`]'s, else the host's.
pub fn dir() -> Option<PathBuf> {
    env_dir().or_else(host_dir)
}

/// Read the database again on its next use (its files changed).
pub fn reload() {
    let mut st = store();
    st.loaded = None;
    st.generation = st.generation.wrapping_add(1);
}

/// A number that changes whenever [`database`] may return something else.
pub fn generation() -> u64 {
    store().generation
}

/// The configured database, read on first use and kept until [`reload`]; `None` when none is configured or it
/// holds no lenses.
pub fn database() -> Option<Arc<LensDb>> {
    let env = env_dir();
    let mut st = store();
    if let Some(db) = &st.loaded {
        return db.clone();
    }
    let db = env.or_else(|| st.dir.clone()).and_then(|dir| match LensDb::load_dir(&dir) {
        Ok((db, errors)) => {
            for (file, e) in errors {
                log::warn!("lens database: skipped {file}: {e}");
            }
            (!db.lenses.is_empty()).then(|| Arc::new(db))
        }
        // (a settings folder without a `lensfun` folder is the usual case: nothing was downloaded)
        Err(e) => {
            log::debug!("{e}");
            None
        }
    });
    st.loaded = Some(db.clone());
    db
}

/// A private lens folder for a while: the host folder is `dir` until the guard is dropped, and other holders wait.
/// For tests and tools, which share this process-wide setting.
pub fn scoped_dir(dir: Option<PathBuf>) -> ScopedDir {
    static TURN: Mutex<()> = Mutex::new(());
    let turn = TURN.lock().unwrap_or_else(PoisonError::into_inner);
    let before = host_dir();
    use_dir(dir);
    ScopedDir { before, _turn: turn }
}

/// See [`scoped_dir`].
pub struct ScopedDir {
    before: Option<PathBuf>,
    _turn: MutexGuard<'static, ()>,
}

impl Drop for ScopedDir {
    fn drop(&mut self) {
        use_dir(self.before.take());
        store().source = None;
    }
}

/// What a photo's lens data corrects, for the Lens Corrections panel.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Corrects {
    pub distortion: bool,
    /// Lateral chromatic aberration: the colour planes are warped differently.
    pub fringing: bool,
    pub vignetting: bool,
}

/// What `lens` corrects.
pub fn corrects(lens: &EmbeddedLens) -> Corrects {
    let planes_differ = lens.warp.is_some_and(|w| w.planes[0] != w.planes[1] || w.planes[2] != w.planes[1])
        || lens.stages.iter().flatten().any(|s| s.planes[0] != s.planes[1] || s.planes[2] != s.planes[1]);
    Corrects { distortion: lens.warp.is_some() || lens.has_stages(), fringing: planes_differ, vignetting: lens.vignette.is_some() }
}

impl crate::Session {
    /// Record lens data found for photos (`found`: photo, the file it was read from, the data), each as
    /// [`Self::learn_lens`] does: only for a raw that still shows that file and has no lens data, and not the photo
    /// a slider is being dragged on. Returns how many photos gained lens data.
    pub(crate) fn record_lenses(&mut self, found: &[(lightcraft_catalog::PhotoId, String, EmbeddedLens)]) -> usize {
        use lightcraft_catalog::Source;
        let dragging = self.interaction.as_ref().map(|i| i.photo);
        let (mut ops, mut gained) = (Vec::new(), 0);
        for (id, path, lens) in found {
            let Some(p) = self.catalog.photo(*id) else { continue };
            let same_file = matches!(&p.source, Source::File { path: now } if now == path);
            if p.deleted || p.embedded_lens.is_some() || !p.develops_raw() || !same_file || dragging == Some(*id) {
                continue;
            }
            let probe = crate::media::ProbeInfo { embedded_lens: Some(*lens), ..Default::default() };
            let mine = crate::cmd::convert::lens_ops(*id, p, &probe);
            if !mine.is_empty() {
                ops.extend(mine);
                gained += 1;
            }
        }
        if !ops.is_empty()
            && let Err(e) = self.apply_system(lightcraft_catalog::Op::Batch { ops })
        {
            log::warn!("lens data not recorded: {e}");
            return 0;
        }
        gained
    }

    /// Take what a background lookup (`optics.findLensProfiles` with `background`) found and record it: how many
    /// photos gained lens data, or `None` when no lookup has ended since the last call. The UI calls this every
    /// frame. A lookup started for another library is dropped.
    pub fn finish_lens_scan(&mut self) -> Option<usize> {
        let found = self.lens_profiles.state().found.take()?;
        Some(if found.library == self.library_generation() { self.record_lenses(&found.lenses) } else { 0 })
    }

    /// Record the lens data a decode of photo `id`'s file found (`info`) when the catalog has none: a raw
    /// catalogued before the lens database was installed. As at import, the correction is switched on when the
    /// photo's settings are still what import gave it; an edited photo only gains the data. Bookkeeping, not an
    /// undo step. Nothing is ever taken away here: a photo keeps its lens data when the database goes.
    pub(crate) fn learn_lens(&mut self, id: lightcraft_catalog::PhotoId, info: &lightcraft_pipeline::SourceInfo) {
        let Some(lens) = info.lens else { return };
        let Some(p) = self.catalog.photo(id) else { return };
        if p.embedded_lens.is_some() || !p.develops_raw() || self.interaction.is_some() {
            return;
        }
        let probe = crate::media::ProbeInfo { embedded_lens: Some(lens), ..Default::default() };
        let ops = crate::cmd::convert::lens_ops(id, p, &probe);
        if !ops.is_empty()
            && let Err(e) = self.apply_system(lightcraft_catalog::Op::Batch { ops })
        {
            log::warn!("photo {}: lens data not recorded: {e}", id.0);
        }
    }
}

/// The database profile for a photo with metadata `m`, for its `w × h` (unoriented, default-cropped) image,
/// re-expressed after `orientation`. `None` without a database, lens name, focal length or matching profile.
pub fn lookup(db: &LensDb, m: &lightcraft_meta::Metadata, w: f64, h: f64, orientation: lightcraft_geom::Orientation) -> Option<EmbeddedLens> {
    let focal = m.focal_length.filter(|f| f.is_finite() && *f > 0.0)?;
    let camera = db.find_camera(m.make.as_deref().unwrap_or_default(), m.model.as_deref().unwrap_or_default());
    let lens = db.find_lens(m.lens_make.as_deref(), m.lens_model.as_deref()?, camera)?;
    let crop = frame_crop_factor(camera.map(|c| c.crop_factor), focal, m.focal_length_35mm);
    let shot = Shot { focal, aperture: m.f_number, distance: None, crop_factor: crop, width: w, height: h };
    let corr = lens.correction(&shot)?;
    Some(lightcraft_pipeline::optics::reorient_lens(&corr, orientation, w, h))
}

/// The crop factor of the recorded frame, which scales the profile's radii: the body's (`camera`), else what the
/// 35 mm equivalent focal length implies. A body shooting a crop of its sensor (a full-frame camera in APS-C /
/// Super 35 mode, or switched there by an APS-C lens) records a larger 35 mm equivalent than its own crop factor
/// gives; the frame's corners are then nearer the optical axis, so that crop wins. Within 10 % of the body's, the
/// difference is the tag's whole-millimetre rounding and the body's value stands.
fn frame_crop_factor(camera: Option<f64>, focal: f64, focal_35mm: Option<f64>) -> Option<f64> {
    let frame = focal_35mm.map(|f35| f35 / focal).filter(|c| c.is_finite() && (0.1..=20.0).contains(c));
    match (camera, frame) {
        (Some(body), Some(frame)) if frame > body * 1.1 => Some(frame),
        (body, frame) => body.or(frame),
    }
}

/// [`crate::files::embedded_lens`], else the database profile ([`lookup`]) for a raw with metadata `meta`.
pub fn for_raw(raw: &lightcraft_raw::RawInfo, meta: impl FnOnce() -> lightcraft_meta::Metadata) -> Option<EmbeddedLens> {
    crate::files::embedded_lens(raw).or_else(|| {
        let db = database()?;
        let c = raw.crop.clipped(raw.active_area.width, raw.active_area.height);
        let (w, h) = if c.width == 0 || c.height == 0 { (raw.active_area.width, raw.active_area.height) } else { (c.width, c.height) };
        lookup(&db, &meta(), w as f64, h as f64, raw.orientation)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lightcraft_geom::Orientation;

    const DB: &str = r#"<lensdatabase>
      <camera><maker>Testcam</maker><model>TC-1</model><mount>T</mount><cropfactor>1.5</cropfactor></camera>
      <lens><maker>Testcam</maker><model>Testcam 18-55mm f/3.5-5.6</model><mount>T</mount><cropfactor>1.5</cropfactor>
        <calibration>
          <distortion model="poly3" focal="18" k1="-0.03" />
          <vignetting model="pa" focal="18" aperture="3.5" distance="1000" k1="-0.4" k2="0" k3="0" />
        </calibration>
      </lens></lensdatabase>"#;

    fn meta() -> lightcraft_meta::Metadata {
        lightcraft_meta::Metadata {
            make: Some("Testcam".into()),
            model: Some("TC-1".into()),
            lens_model: Some("18-55mm F3.5-5.6".into()),
            focal_length: Some(18.0),
            f_number: Some(3.5),
            ..Default::default()
        }
    }

    #[test]
    fn exif_lens_is_found_and_converted() {
        let db = LensDb::parse(DB).unwrap();
        let lens = lookup(&db, &meta(), 6000.0, 4000.0, Orientation::Normal).unwrap();
        assert!(lens.warp.is_some() && lens.vignette.is_some());
        let rotated = lookup(&db, &meta(), 6000.0, 4000.0, Orientation::Rotate90).unwrap();
        assert_eq!(rotated.warp.unwrap().center, lightcraft_geom::Point::new(0.5, 0.5));
        // no lens name, no focal length, an unknown lens: nothing
        assert!(lookup(&db, &lightcraft_meta::Metadata { lens_model: None, ..meta() }, 6000.0, 4000.0, Orientation::Normal).is_none());
        assert!(lookup(&db, &lightcraft_meta::Metadata { focal_length: None, ..meta() }, 6000.0, 4000.0, Orientation::Normal).is_none());
        let other = lightcraft_meta::Metadata { lens_model: Some("70-200mm F4".into()), ..meta() };
        assert!(lookup(&db, &other, 6000.0, 4000.0, Orientation::Normal).is_none());
    }

    /// A full-frame body in APS-C / Super 35 mode (an ILCE-1M2 at 70 mm records 105 mm equivalent) frames only the
    /// middle of the image circle: the profile is applied at the frame's crop factor, not the body's, so the crop's
    /// corners don't get the full frame's corner correction.
    #[test]
    fn in_camera_crop_mode_uses_the_frames_crop_factor() {
        let db = LensDb::parse(
            r#"<lensdatabase>
              <camera><maker>Sony</maker><model>ILCE-1M2</model><mount>Sony E</mount><cropfactor>1</cropfactor></camera>
              <lens><maker>Sigma</maker><model>24-70mm F2.8 DG DN | Art 019</model><mount>Sony E</mount><cropfactor>1.0</cropfactor>
                <calibration>
                  <distortion model="ptlens" focal="70" a="0.0019947" b="0.0035263" c="0.0005016" />
                  <vignetting model="pa" focal="70" aperture="2.8" distance="1000" k1="-0.4" k2="0" k3="0" />
                </calibration>
              </lens></lensdatabase>"#,
        )
        .unwrap();
        let m = |f35: Option<f64>| lightcraft_meta::Metadata {
            make: Some("SONY".into()),
            model: Some("ILCE-1M2".into()),
            lens_model: Some("24-70mm F2.8 DG DN | Art 019".into()),
            focal_length: Some(70.0),
            focal_length_35mm: f35,
            f_number: Some(2.8),
            ..Default::default()
        };
        let at = |crop: f64, w: f64, h: f64| {
            let shot = Shot { focal: 70.0, aperture: Some(2.8), distance: None, crop_factor: Some(crop), width: w, height: h };
            db.lenses[0].correction(&shot).unwrap()
        };
        let aps_c = lookup(&db, &m(Some(105.0)), 5616.0, 3744.0, Orientation::Normal).unwrap();
        assert_eq!(aps_c, at(1.5, 5616.0, 3744.0));
        assert_ne!(aps_c, at(1.0, 5616.0, 3744.0), "the crop is not corrected as a full frame");
        // full frame, a 35 mm equivalent rounded off by a millimetre, or none at all: the body's crop factor
        for f35 in [Some(70.0), Some(71.0), None] {
            assert_eq!(lookup(&db, &m(f35), 8640.0, 5760.0, Orientation::Normal).unwrap(), at(1.0, 8640.0, 5760.0), "{f35:?}");
        }
        assert_eq!(frame_crop_factor(None, 70.0, Some(105.0)), Some(1.5), "no body in the database: the 35 mm equivalent's");
        assert_eq!(frame_crop_factor(Some(1.0), 0.0, Some(105.0)), Some(1.0), "a hostile focal length is ignored");
    }

    /// A synthetic grid recorded through the profile's barrel distortion comes out of the pipeline's geometry
    /// stage with straight lines.
    #[test]
    fn pipeline_straightens_a_distorted_grid() {
        let db = LensDb::parse(DB).unwrap();
        let profile = &db.lenses[0];
        let (w, h) = (600usize, 400usize);
        let lens = lookup(&db, &meta(), w as f64, h as f64, Orientation::Normal).unwrap();
        // record the grid: each sensor pixel sees the scene point whose distorted position it is
        let n = h as f64 / 2.0;
        let undistort = |x: f64, y: f64| {
            let (dx, dy) = ((x - w as f64 / 2.0) / n, (y - h as f64 / 2.0) / n);
            let rd = dx.hypot(dy);
            let (mut lo, mut hi) = (0.0, 3.0);
            for _ in 0..60 {
                let mid = (lo + hi) / 2.0;
                if mid * profile.distortion_ratio(18.0, mid) < rd {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let s = if rd > 0.0 { lo / rd } else { 1.0 };
            (w as f64 / 2.0 + dx * s * n, h as f64 / 2.0 + dy * s * n)
        };
        // horizontal scene lines every 50 px (2 px wide, dark on light)
        let line = |v: f64| if v.rem_euclid(50.0) < 2.0 { 0.02f32 } else { 0.8 };
        let src = lightcraft_raster::Rgb32f::from_fn(w, h, |x, y| {
            let (_, sy) = undistort(x as f64 + 0.5, y as f64 + 0.5);
            [line(sy); 3]
        });
        // the dark row nearest y = 51 in each column, sub-pixel (centroid of darkness), as rendered
        let line_rows = |img: &lightcraft_raster::Rgba8| -> Vec<f64> {
            (20..w - 20)
                .step_by(10)
                .map(|x| {
                    let (mut sw, mut swy) = (0.0, 0.0);
                    for y in 35..70 {
                        let d = 255.0 - img.get(x, y)[1] as f64;
                        if d > 60.0 {
                            sw += d;
                            swy += d * (y as f64 + 0.5);
                        }
                    }
                    swy / sw.max(1e-9)
                })
                .collect()
        };
        let spread = |rows: &[f64]| rows.iter().cloned().fold(f64::MIN, f64::max) - rows.iter().cloned().fold(f64::MAX, f64::min);
        let req = lightcraft_pipeline::RenderRequest::fit(w, h);
        let mut s = lightcraft_develop::DevelopSettings::default();
        let info = lightcraft_pipeline::SourceInfo { lens: Some(lens), ..Default::default() };
        let before = spread(&line_rows(&lightcraft_pipeline::render(&src, &info, &s, &req).image));
        s.optics.lens_profile = true;
        let after = spread(&line_rows(&lightcraft_pipeline::render(&src, &info, &s, &req).image));
        assert!(before > 2.0, "the recorded line is curved: {before} px");
        // 8-bit output and a 2 px line: the centroid measurement itself is good to about a pixel
        assert!(after < 1.0 && after < before / 8.0, "corrected line bent by {after} px (uncorrected {before})");
    }
}
