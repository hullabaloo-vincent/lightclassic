//! The lens-profile database: where it is, and fetching it.
//!
//! Lens profiles ([`crate::lens_profiles`]) come from the lensfun project's database, which is licensed CC BY-SA 3.0
//! and is not shipped with the app. `optics.downloadLensProfiles` fetches its XML files (about 5 MB) into the
//! host's lens folder, in the background; nothing about the user's photos is sent. Only the data files are fetched,
//! never lensfun's code.
//!
//! Sources: the lensfun database's file layout (its `data/db` directory listing); own design.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use lightcraft_catalog::{PhotoId, Source};
use lightcraft_develop::EmbeddedLens;
use lightcraft_lensdb::LensDb;
use serde_json::{Value, json};

use super::{CommandSpec, always, bad, bool_or, cmd};
use crate::lens_profiles::{self as store, ATTRIBUTION, DownloadOutcome};
use crate::{Result, Session};

/// Where the lensfun project publishes its database files.
pub const SOURCE: &str = "https://raw.githubusercontent.com/lensfun/lensfun/master/data/db";

/// The database's files (lensfun `data/db`, as listed on 2026-10-13). A file the project has since dropped is
/// skipped; one it has added is not fetched until it is named here.
pub const FILES: &[&str] = &[
    "6x6.xml",
    "actioncams.xml",
    "compact-canon.xml",
    "compact-casio.xml",
    "compact-fujifilm.xml",
    "compact-kodak.xml",
    "compact-konica-minolta.xml",
    "compact-leica.xml",
    "compact-nikon.xml",
    "compact-olympus.xml",
    "compact-panasonic.xml",
    "compact-pentax.xml",
    "compact-ricoh.xml",
    "compact-samsung.xml",
    "compact-sigma.xml",
    "compact-sony.xml",
    "contax.xml",
    "generic.xml",
    "mil-canon.xml",
    "mil-fujifilm.xml",
    "mil-hasselblad.xml",
    "mil-leica.xml",
    "mil-nikon.xml",
    "mil-olympus.xml",
    "mil-panasonic.xml",
    "mil-pentax.xml",
    "mil-samsung.xml",
    "mil-samyang.xml",
    "mil-sigma.xml",
    "mil-sony.xml",
    "mil-tamron.xml",
    "mil-tokina.xml",
    "mil-zeiss.xml",
    "misc.xml",
    "om-system.xml",
    "rf-leica.xml",
    "slr-canon.xml",
    "slr-hasselblad.xml",
    "slr-konica-minolta.xml",
    "slr-leica.xml",
    "slr-nikon.xml",
    "slr-olympus.xml",
    "slr-panasonic.xml",
    "slr-pentax.xml",
    "slr-ricoh.xml",
    "slr-samsung.xml",
    "slr-samyang.xml",
    "slr-schneider.xml",
    "slr-sigma.xml",
    "slr-soligor.xml",
    "slr-sony.xml",
    "slr-tamron.xml",
    "slr-tokina.xml",
    "slr-ussr.xml",
    "slr-vivitar.xml",
    "slr-zeiss.xml",
];

fn status(s: &mut Session, _: &Value) -> Result<Value> {
    let db = store::database();
    let (last, error) = match s.lens_profiles.last_outcome() {
        None => (Value::Null, Value::Null),
        Some(DownloadOutcome::Done { lenses, files }) => (json!({"lenses": lenses, "files": files}), Value::Null),
        Some(DownloadOutcome::Failed(e)) => (json!("failed"), json!(e)),
        Some(DownloadOutcome::Cancelled) => (json!("cancelled"), Value::Null),
    };
    Ok(json!({
        "installed": db.is_some(),
        "lenses": db.as_ref().map_or(0, |d| d.lenses.len()),
        "cameras": db.as_ref().map_or(0, |d| d.cameras.len()),
        "path": store::dir(),
        "fromEnv": store::env_dir().is_some(),
        "downloadTo": store::host_dir(),
        "downloading": s.lens_profiles.downloading(),
        "looking": s.lens_profiles.scanning(),
        "lastDownload": last,
        "lastError": error,
        "attribution": ATTRIBUTION,
        "source": SOURCE,
    }))
}

/// A raw file without lens data: its path, the photos that show it (virtual copies share a file) and the lens name
/// the catalog has for it.
struct Candidate {
    path: String,
    ids: Vec<PhotoId>,
    lens: String,
}

/// The raws among `ids` (`None`: the whole library) that are developed from their sensor data, have a file and
/// no lens data yet.
fn candidates(s: &Session, ids: Option<&[PhotoId]>) -> Vec<Candidate> {
    let mut by_path: std::collections::BTreeMap<String, Candidate> = std::collections::BTreeMap::new();
    let mut add = |p: &lightcraft_catalog::Photo| {
        let Source::File { path } = &p.source else { return };
        if p.deleted || p.embedded_lens.is_some() || !p.develops_raw() {
            return;
        }
        let c = by_path.entry(path.clone()).or_insert_with(|| Candidate { path: path.clone(), ids: Vec::new(), lens: String::new() });
        c.ids.push(p.id);
        if c.lens.is_empty() {
            c.lens = p.meta.lens.trim().to_string();
        }
    };
    match ids {
        Some(ids) => ids.iter().filter_map(|id| s.catalog.photo(*id)).for_each(|p| add(p)),
        None => s.catalog.photos().for_each(|p| add(p)),
    }
    by_path.into_values().collect()
}

/// The candidates whose lens the database names, going by the catalog's lens name without opening a file: what a
/// lookup over a whole library reads, so photos the database can't help are not read again every time.
fn named(todo: Vec<Candidate>, db: &LensDb) -> Vec<Candidate> {
    let mut known: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    todo.into_iter()
        .filter(|c| !c.lens.is_empty() && *known.entry(c.lens.clone()).or_insert_with(|| db.find_lens(None, &c.lens, None).is_some()))
        .collect()
}

/// Read each candidate's file and keep the lens data found, for every photo that shows it (`only_named`: of
/// [`named`] candidates only).
fn look_up(
    todo: Vec<Candidate>,
    db: &LensDb,
    probe: &crate::media::FileProbe,
    only_named: bool,
    mut keep_going: impl FnMut(usize, usize) -> bool,
) -> Vec<(PhotoId, String, EmbeddedLens)> {
    let todo = if only_named { named(todo, db) } else { todo };
    let mut out = Vec::new();
    for (n, c) in todo.iter().enumerate() {
        if !keep_going(n, todo.len()) {
            break;
        }
        if let Ok(info) = probe(&c.path)
            && let Some(lens) = info.embedded_lens
        {
            out.extend(c.ids.iter().map(|id| (*id, c.path.clone(), lens)));
        }
    }
    out
}

/// `optics.findLensProfiles`: give raws catalogued without lens data the profile the database has for them.
fn find(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "optics.findLensProfiles";
    let all = bool_or(p, "all", false);
    let background = bool_or(p, "background", false);
    let ids = (!all).then(|| s.targets(p));
    let todo = candidates(s, ids.as_deref());
    let nothing = |why: &str| Ok(json!({"candidates": todo.len(), "found": [], "started": false, "note": why}));
    let Some(probe) = s.media.file_probe.clone() else { return nothing("this session reads no files") };
    if todo.is_empty() {
        return nothing("every raw here has lens data, or none is a raw with a file");
    }
    let count = todo.len();
    if background {
        // (the database is read on the lookup's own thread: a first read takes a moment)
        if !store::folder_exists() {
            return nothing("no lens database is installed");
        }
        return scan::start(s, todo, probe).map(|()| json!({"candidates": count, "started": true})).map_err(|e| bad(C, e));
    }
    let Some(db) = store::database() else { return nothing("no lens database is installed") };
    let found = look_up(todo, &db, &probe, all, |_, _| true);
    let gained = s.record_lenses(&found);
    let ids: Vec<u64> = found.iter().map(|(id, _, _)| id.0).collect();
    Ok(json!({"candidates": count, "found": ids, "changed": gained, "started": false}))
}

/// The background lookup: one thread reading files, shown in the activity stack, cancellable.
mod scan {
    #[cfg(not(target_arch = "wasm32"))]
    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

    use super::*;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::activity::Cancel;

    #[cfg(target_arch = "wasm32")]
    pub(super) fn start(_: &mut Session, _: Vec<Candidate>, _: crate::media::FileProbe) -> std::result::Result<(), String> {
        Err("a background lookup needs threads: call without `background`".into())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn start(s: &mut Session, todo: Vec<Candidate>, probe: crate::media::FileProbe) -> std::result::Result<(), String> {
        let jobs = s.lens_profiles.clone();
        if !jobs.begin_scan() {
            return Err("lens profiles are already being looked up".into());
        }
        let library = s.library_generation();
        let activity = s.activity.clone();
        let thread_jobs = jobs.clone();
        let spawned = std::thread::Builder::new().name("lens-profile-lookup".into()).spawn(move || {
            let found = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let Some(db) = store::database() else { return Vec::new() };
                let todo = named(todo, &db);
                if todo.is_empty() {
                    return Vec::new();
                }
                // only now is there something that takes time: reading the files (the row goes when this ends)
                let cancel = Arc::new(AtomicBool::new(false));
                let task = activity.start("lensProfiles", "Looking up lens profiles", Cancel::Flag(cancel.clone()));
                look_up(todo, &db, &probe, false, |n, of| {
                    task.progress(n as u64, of as u64);
                    !cancel.load(Relaxed)
                })
            }));
            // (a cancelled lookup keeps what it had found)
            thread_jobs.end_scan(Some(store::Found { library, lenses: found.unwrap_or_default() }));
        });
        if let Err(e) = spawned {
            jobs.end_scan(None);
            return Err(format!("could not start the lookup: {e}"));
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod download {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

    use super::*;
    use crate::activity::{Cancel, TaskHandle};
    use crate::cmd::str_param;

    /// What the files are and what they may be used for, written beside them.
    const ABOUT: &str = "These XML files are the lens database of the lensfun project (https://lensfun.github.io),\n\
    licensed under the Creative Commons Attribution-ShareAlike 3.0 licence\n\
    (https://creativecommons.org/licenses/by-sa/3.0/). LightCraft downloaded them unchanged, at your request, from\n\
    https://github.com/lensfun/lensfun (data/db) and reads lens corrections from them.\n";

    /// The folder a download goes to: `dir`, else the host's lens folder.
    fn target_dir(p: &Value) -> Option<PathBuf> {
        str_param(p, "dir").map(PathBuf::from).or_else(store::host_dir)
    }

    /// Largest file accepted (the database's biggest is under 1 MB).
    const MAX_FILE: u64 = 16 << 20;
    /// Files that may be missing or unreadable before the download counts as failed.
    const MAX_SKIPPED: usize = 8;

    /// A download base: https, or plain http to this machine's loopback address only (a test server), with nothing
    /// but a port or path after the host, so `http://127.0.0.1.example.com/` is refused.
    pub(super) fn base_allowed(base: &str) -> bool {
        base.starts_with("https://")
            || base.strip_prefix("http://127.0.0.1").is_some_and(|rest| rest.is_empty() || rest.starts_with(':') || rest.starts_with('/'))
    }

    /// Fetch one file into `staging` and read it as a lens database: its lens count.
    fn fetch(name: &str, base: &str, staging: &Path, cancel: &AtomicBool) -> std::result::Result<usize, String> {
        let file = lightcraft_fetch::FileSpec { name, size: None, sha256: None, max: MAX_FILE };
        let url = format!("{}/{name}", base.trim_end_matches('/'));
        lightcraft_fetch::download_url(&file, &url, staging, &lightcraft_fetch::Options::default(), cancel, &mut |_| {})
            .map_err(|e| e.to_string())?;
        let bytes = std::fs::read(staging.join(name)).map_err(|e| e.to_string())?;
        LensDb::parse(&String::from_utf8_lossy(&bytes)).map(|db| db.lenses.len()).map_err(|e| e.to_string())
    }

    /// Fetch the database into `dir`: (lens profiles, files). Every file is read as a lens database before any is
    /// put in place, so a failed or cancelled download leaves the folder as it was.
    pub(super) fn run(base: &str, dir: &Path, cancel: &AtomicBool, task: &TaskHandle) -> std::result::Result<(usize, usize), String> {
        let staging = dir.join(".download");
        let _ = std::fs::remove_dir_all(&staging);
        std::fs::create_dir_all(&staging).map_err(|e| format!("could not create {}: {e}", staging.display()))?;
        let fetched = fetch_all(base, &staging, cancel, task).and_then(|(lenses, good)| {
            for name in &good {
                let (from, to) = (staging.join(name), dir.join(name));
                std::fs::rename(&from, &to).or_else(|_| std::fs::copy(&from, &to).map(|_| ())).map_err(|e| format!("could not save {name}: {e}"))?;
            }
            let _ = std::fs::write(dir.join("ABOUT.txt"), ABOUT);
            Ok((lenses, good.len()))
        });
        let _ = std::fs::remove_dir_all(&staging);
        fetched
    }

    fn fetch_all(base: &str, staging: &Path, cancel: &AtomicBool, task: &TaskHandle) -> std::result::Result<(usize, Vec<&'static str>), String> {
        let (mut lenses, mut good, mut skipped) = (0usize, Vec::new(), 0usize);
        for (n, name) in FILES.iter().enumerate() {
            if cancel.load(Relaxed) {
                return Err("cancelled".into());
            }
            task.progress(n as u64, FILES.len() as u64);
            match fetch(name, base, staging, cancel) {
                Ok(count) => {
                    lenses = lenses.saturating_add(count);
                    good.push(*name);
                }
                // the first file failing is the connection or the address, not that file: no point in 55 more tries
                Err(e) if good.is_empty() || skipped >= MAX_SKIPPED => return Err(e),
                Err(e) => {
                    log::warn!("lens profiles: {name} skipped: {e}");
                    skipped += 1;
                }
            }
        }
        if lenses == 0 {
            return Err("the downloaded database holds no lens profiles".into());
        }
        Ok((lenses, good))
    }

    /// `optics.downloadLensProfiles`: starts a background download (shown in the activity stack, cancellable).
    pub(super) fn start(s: &mut Session, p: &Value) -> Result<Value> {
        const C: &str = "optics.downloadLensProfiles";
        let dir = target_dir(p).ok_or_else(|| bad(C, "no settings folder: give `dir`"))?;
        let base = str_param(p, "base").map(str::to_string).or_else(store::source).unwrap_or_else(|| SOURCE.to_string());
        if !base_allowed(&base) {
            return Err(bad(C, "`base` must be an https:// address"));
        }
        let jobs = s.lens_profiles.clone();
        if !jobs.begin_download() {
            return Err(bad(C, "the lens profiles are already downloading"));
        }
        let thread_jobs = jobs.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let task = s.activity.start("download", "Downloading lens profiles", Cancel::Flag(cancel.clone()));
        task.detail("lensfun");
        let handle = task.handle();
        let thread_dir = dir.clone();
        let spawned = std::thread::Builder::new().name("lens-profiles-download".into()).spawn(move || {
            // the activity row goes when the thread ends, however it ends
            let _task = task;
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&base, &thread_dir, &cancel, &handle)));
            let outcome = match outcome {
                Ok(Ok((lenses, files))) => DownloadOutcome::Done { lenses, files },
                _ if cancel.load(Relaxed) => DownloadOutcome::Cancelled,
                Ok(Err(e)) => DownloadOutcome::Failed(format!("Could not download the lens profiles: {e}. Check the connection and try again.")),
                Err(_) => DownloadOutcome::Failed("The lens profile download stopped unexpectedly. Try again.".into()),
            };
            match &outcome {
                DownloadOutcome::Done { .. } => store::reload(),
                DownloadOutcome::Failed(e) => log::warn!("{e}"),
                DownloadOutcome::Cancelled => {}
            }
            thread_jobs.end_download(Some(outcome));
        });
        if let Err(e) = spawned {
            jobs.end_download(None);
            return Err(bad(C, format!("could not start the download: {e}")));
        }
        Ok(json!({"started": true, "path": dir, "files": FILES.len(), "attribution": ATTRIBUTION}))
    }
}

pub fn specs() -> Vec<CommandSpec> {
    #[allow(unused_mut)]
    let mut v = vec![
        cmd!(query "optics.lensProfilesStatus", "Lens Profiles Status", [], None, "→ {installed, lenses, cameras, path, fromEnv (LIGHTCRAFT_LENSFUN_DIR names the folder), downloadTo, downloading, looking, lastDownload, lastError, attribution, source} — the lens-profile database Enable Lens Corrections reads for raws without lens data of their own", always, status),
        cmd!(
            "optics.findLensProfiles",
            "Find Lens Profiles",
            [],
            None,
            "{ids?, all?: bool (the whole library, reading only files whose lens the database names), background?: bool} → {candidates, found: [ids], changed, started} — reads the files of raws catalogued without lens data and gives them the profile the lens database has for their lens; the correction is switched on for photos still at their import settings, as import does. Bookkeeping, not an undo step. With `background` it runs in the activity stack and its result is recorded when it ends",
            always,
            find
        ),
    ];
    #[cfg(not(target_arch = "wasm32"))]
    v.push(cmd!(
        "optics.downloadLensProfiles",
        "Download Lens Profiles (lensfun)",
        ["Photo"],
        None,
        "{dir?: folder} → {started, path, files, attribution} — downloads the lensfun lens database (CC BY-SA 3.0, about 5 MB of XML, from the lensfun project on GitHub) in the background into the lens folder; raws without lens data of their own are then corrected from it when they are next opened. No photo data is sent",
        always,
        download::start
    ));
    v
}

#[cfg(test)]
#[path = "lens_profiles_tests.rs"]
mod tests;
