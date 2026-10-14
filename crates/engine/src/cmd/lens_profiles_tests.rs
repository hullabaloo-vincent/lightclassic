use std::path::{Path, PathBuf};
use std::sync::Arc;

use lightcraft_catalog::{MediaKind, Op, Photo, PhotoId, Source};
use lightcraft_develop::{EmbeddedLens, EmbeddedWarp};
use lightcraft_pipeline::SourceInfo;

use super::*;

/// A synthetic database file in the lensfun format (made-up coefficients, not lensfun data).
fn file(lens: &str) -> String {
    format!(
        r#"<lensdatabase version="2">
          <camera><maker>Testcam</maker><model>TC-1</model><mount>T</mount><cropfactor>1.5</cropfactor></camera>
          <lens><maker>Testcam</maker><model>Testcam {lens} 18-55mm f/3.5-5.6</model><mount>T</mount><cropfactor>1.5</cropfactor>
            <calibration><distortion model="poly3" focal="18" k1="-0.03" /></calibration>
          </lens></lensdatabase>"#
    )
}

fn temp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lightcraft-lens-profiles-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn lens() -> EmbeddedLens {
    EmbeddedLens { warp: Some(EmbeddedWarp { planes: [[1.0, -0.1, 0.03, 0.0, 0.0, 0.0]; 3], ..Default::default() }), ..Default::default() }
}

/// A raw as an older session catalogued it: no lens data, correction off.
fn add_raw(s: &mut Session, name: &str) -> PhotoId {
    let id = s.catalog.alloc_photo_id();
    let mut p = Photo::new(id, Source::File { path: format!("/photos/{name}") }, name, "CR2", 64, 64, "2026-01-01T00:00:00");
    p.kind = MediaKind::Raw;
    p.develop = Arc::new(p.camera_defaults());
    s.catalog.apply(Op::AddPhoto { photo: Box::new(p) }).unwrap();
    id
}

#[test]
fn the_database_is_read_from_the_hosts_folder_and_reread_on_reload() {
    let dir = temp("store");
    let _scope = store::scoped_dir(Some(dir.clone()));
    let mut s = Session::new();
    let st = s.execute("optics.lensProfilesStatus", &json!({})).unwrap();
    assert_eq!((st["installed"].as_bool(), st["lenses"].as_u64()), (Some(false), Some(0)), "no folder yet: {st}");
    assert_eq!(st["downloadTo"].as_str(), dir.to_str());
    assert!(st["attribution"].as_str().is_some_and(|a| a.contains("lensfun") && a.contains("CC BY-SA")));

    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.xml"), file("A")).unwrap();
    assert!(store::database().is_none(), "kept until told the files changed");
    let before = store::generation();
    store::reload();
    assert_ne!(store::generation(), before);
    let st = s.execute("optics.lensProfilesStatus", &json!({})).unwrap();
    assert_eq!((st["installed"].as_bool(), st["lenses"].as_u64(), st["cameras"].as_u64()), (Some(true), Some(1), Some(1)), "{st}");
    assert_eq!(st["path"].as_str(), dir.to_str());

    // a photo whose EXIF names the lens gets the profile
    let m = lightcraft_meta::Metadata {
        make: Some("Testcam".into()),
        model: Some("TC-1".into()),
        lens_model: Some("A 18-55mm F3.5-5.6".into()),
        focal_length: Some(18.0),
        ..Default::default()
    };
    let db = store::database().unwrap();
    assert!(store::lookup(&db, &m, 6000.0, 4000.0, lightcraft_geom::Orientation::Normal).is_some_and(|l| l.warp.is_some()));

    // naming another folder (or none) drops what was read
    store::use_dir(None);
    assert!(store::database().is_none() && store::dir().is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_decode_teaches_the_catalog_the_lens_and_turns_the_correction_on_for_an_untouched_photo() {
    let mut s = Session::new();
    let id = add_raw(&mut s, "a.CR2");
    let undo = s.undo.len();
    let found = SourceInfo { raw: true, lens: Some(lens()), ..Default::default() };
    // a decode without lens data teaches nothing
    s.learn_lens(id, &SourceInfo { raw: true, ..Default::default() });
    assert!(s.catalog.photo(id).unwrap().embedded_lens.is_none());

    s.learn_lens(id, &found);
    let p = s.catalog.photo(id).unwrap();
    assert_eq!(p.embedded_lens, Some(lens()));
    assert!(p.develop.optics.lens_profile && !p.is_edited(), "as import would have left it");
    assert_eq!(s.undo.len(), undo, "bookkeeping, not an undo step");
    assert!(s.pending_log.iter().any(|op| matches!(op, Op::Batch { .. })), "journaled");
    assert_eq!(s.source_info(id).lens, Some(lens()), "the next render uses it");

    // once known it stays: another decode's data (a changed database) does not replace it
    let logged = s.pending_log.len();
    let other = EmbeddedLens { warp: Some(EmbeddedWarp { planes: [[1.0, 0.2, 0.0, 0.0, 0.0, 0.0]; 3], ..Default::default() }), ..Default::default() };
    s.learn_lens(id, &SourceInfo { raw: true, lens: Some(other), ..Default::default() });
    assert_eq!(s.catalog.photo(id).unwrap().embedded_lens, Some(lens()));
    assert_eq!(s.pending_log.len(), logged);
}

#[test]
fn an_edited_photo_gains_the_lens_data_but_keeps_its_settings() {
    let mut s = Session::new();
    let id = add_raw(&mut s, "b.CR2");
    s.selection.ids = vec![id];
    s.selection.active = Some(id);
    s.execute("develop.set", &json!({"values": {"light.exposure": 0.7}})).unwrap();
    let found = SourceInfo { raw: true, lens: Some(lens()), ..Default::default() };

    // not in the middle of a slider drag: the drag's own settings would be overwritten
    s.begin_interaction("Exposure").unwrap();
    s.learn_lens(id, &found);
    assert!(s.catalog.photo(id).unwrap().embedded_lens.is_none());
    s.execute("develop.cancel", &json!({})).ok();
    s.interaction = None;

    s.learn_lens(id, &found);
    let p = s.catalog.photo(id).unwrap();
    assert_eq!(p.embedded_lens, Some(lens()));
    assert!(!p.develop.optics.lens_profile, "the user's settings are theirs");
    assert_eq!(p.develop.light.exposure, 0.7);

    // a rendered file never gets lens data
    let jpg = s.catalog.alloc_photo_id();
    s.catalog
        .apply(Op::AddPhoto { photo: Box::new(Photo::new(jpg, Source::File { path: "/photos/c.jpg".into() }, "c.jpg", "JPEG", 64, 64, "")) })
        .unwrap();
    s.learn_lens(jpg, &found);
    assert!(s.catalog.photo(jpg).unwrap().embedded_lens.is_none());
}

/// Accepting a finished render whose decode found lens data records it (the path a photo opened in Develop takes).
#[test]
fn accepting_a_decode_records_its_lens_data() {
    use crate::media::SourceRef;
    let mut s = Session::new();
    let id = add_raw(&mut s, "d.CR2");
    s.selection.ids = vec![id];
    s.selection.active = Some(id);
    let decoded = SourceInfo { raw: true, lens: Some(lens()), ..Default::default() };
    let pixels = lightcraft_raster::Rgb32f::from_fn(64, 64, |x, _| [0.1 + x as f32 / 128.0; 3]);
    let mut job = s.render_job(id, 64, 64, false, true).unwrap();
    let (loaded_info, loaded_pixels) = (decoded.clone(), pixels.clone());
    job.source = SourceRef::File {
        path: "/photos/d.CR2".into(),
        max_edge: 64,
        loader: Some(Arc::new(move |_, _| Ok((loaded_pixels.clone(), loaded_info.clone())))),
        fallback: None,
        denoise: None,
    };
    let r = job.run();
    assert!(r.rendered.is_ok());
    s.accept(&r);
    let p = s.catalog.photo(id).unwrap();
    assert_eq!(p.embedded_lens, Some(lens()));
    assert!(p.develop.optics.lens_profile);
}

#[test]
fn corrects_names_what_the_lens_data_holds() {
    use lightcraft_develop::EmbeddedVignette;
    assert_eq!(store::corrects(&lens()), store::Corrects { distortion: true, fringing: false, vignetting: false });
    let mut l = lens();
    if let Some(w) = &mut l.warp {
        w.planes[0][0] = 1.0004;
    }
    l.vignette = Some(EmbeddedVignette::default());
    assert_eq!(store::corrects(&l), store::Corrects { distortion: true, fringing: true, vignetting: true });
    assert_eq!(store::corrects(&EmbeddedLens::default()), store::Corrects::default());
}

/// A local HTTP server; the first path segment picks the behaviour: `good` (every file a small database), `partial`
/// (three files are gone), `garbage` (HTML instead of XML), `empty` (databases without lenses), `sparse` (only the
/// first file exists), anything else 404.
#[cfg(not(target_arch = "wasm32"))]
fn serve() -> String {
    use std::io::{BufRead, BufReader, Write};
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for conn in l.incoming() {
            let Ok(mut c) = conn else { continue };
            let mut r = BufReader::new(c.try_clone().unwrap());
            let mut line = String::new();
            let _ = r.read_line(&mut line);
            loop {
                let mut h = String::new();
                if r.read_line(&mut h).unwrap_or(0) == 0 || h.trim().is_empty() {
                    break;
                }
            }
            let path = line.split(' ').nth(1).unwrap_or("").to_string();
            let mut parts = path.split('/').skip(1);
            let (mode, name) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
            let known = FILES.contains(&name);
            let gone = [FILES[3], FILES[20], FILES[41]].contains(&name);
            let body: Option<Vec<u8>> = match mode {
                "good" if known => Some(file(name).into_bytes()),
                "partial" if known && !gone => Some(file(name).into_bytes()),
                "garbage" if known => Some(b"<html>not a database</html>".to_vec()),
                "empty" if known => Some(b"<lensdatabase version=\"2\"></lensdatabase>".to_vec()),
                "sparse" if name == FILES[0] => Some(file(name).into_bytes()),
                _ => None,
            };
            let (status, body) = match body {
                Some(b) => ("200 OK", b),
                None => ("404 Not Found", b"nope".to_vec()),
            };
            let _ = c.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes());
            let _ = c.write_all(&body);
        }
    });
    format!("http://127.0.0.1:{port}")
}

#[cfg(not(target_arch = "wasm32"))]
fn download(s: &mut Session, base: &str) -> DownloadOutcome {
    let r = s.execute("optics.downloadLensProfiles", &json!({"base": base})).unwrap();
    assert_eq!(r["files"].as_u64(), Some(FILES.len() as u64));
    // the work shows in the activity stack while it runs
    assert!(s.lens_profiles.downloading() || s.lens_profiles.last_outcome().is_some());
    let started = std::time::Instant::now();
    loop {
        if let Some(o) = s.lens_profiles.take_outcome() {
            while !s.activity.list().is_empty() && started.elapsed().as_secs() < 30 {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert!(s.activity.list().is_empty(), "the activity row ends with the download");
            assert!(!s.lens_profiles.downloading());
            return o;
        }
        assert!(started.elapsed().as_secs() < 120, "the download never ended");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn xml_files(dir: &Path) -> usize {
    std::fs::read_dir(dir).map(|d| d.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "xml")).count()).unwrap_or(0)
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn download_end_to_end_and_its_failures() {
    let dir = temp("download");
    let _scope = store::scoped_dir(Some(dir.clone()));
    let mut s = Session::new();
    let base = serve();
    assert!(s.execute("optics.downloadLensProfiles", &json!({"base": "http://example.com/"})).is_err(), "https or loopback only");

    // nothing there, not a database, databases without lenses, most files missing: an error to act on, and nothing kept
    for bad in ["missing", "garbage", "empty", "sparse"] {
        let o = download(&mut s, &format!("{base}/{bad}"));
        let DownloadOutcome::Failed(e) = o else { panic!("{bad}: {o:?}") };
        assert!(e.contains("try again"), "{bad}: {e}");
        let st = s.execute("optics.lensProfilesStatus", &json!({})).unwrap();
        assert_eq!(
            (st["installed"].as_bool(), st["lastDownload"].as_str(), st["lastError"].as_str()),
            (Some(false), Some("failed"), Some(e.as_str()))
        );
        assert_eq!(xml_files(&dir), 0, "{bad}");
        assert!(!dir.join(".download").exists(), "{bad}: staging folder left behind");
        assert!(s.lens_profiles.take_outcome().is_none(), "reported once");
    }

    // a few files gone from the project: the rest is installed
    assert_eq!(download(&mut s, &format!("{base}/partial")), DownloadOutcome::Done { lenses: FILES.len() - 3, files: FILES.len() - 3 });
    assert_eq!(xml_files(&dir), FILES.len() - 3);

    let before = store::generation();
    assert_eq!(download(&mut s, &format!("{base}/good/")), DownloadOutcome::Done { lenses: FILES.len(), files: FILES.len() });
    assert_ne!(store::generation(), before, "the new files are read");
    let st = s.execute("optics.lensProfilesStatus", &json!({})).unwrap();
    assert_eq!((st["installed"].as_bool(), st["lenses"].as_u64(), st["lastError"].is_null()), (Some(true), Some(FILES.len() as u64), true), "{st}");
    assert_eq!(st["lastDownload"]["files"].as_u64(), Some(FILES.len() as u64));
    assert_eq!(xml_files(&dir), FILES.len());
    assert!(std::fs::read_to_string(dir.join("ABOUT.txt")).is_ok_and(|t| t.contains("lensfun") && t.contains("Attribution-ShareAlike")));
    assert!(!dir.join(".download").exists());

    // a later download that fails leaves the installed database alone
    assert!(matches!(download(&mut s, &format!("{base}/garbage")), DownloadOutcome::Failed(_)));
    assert_eq!(xml_files(&dir), FILES.len());
    assert!(store::database().is_some_and(|db| db.lenses.len() == FILES.len()));
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_download_needs_a_folder_and_one_at_a_time() {
    let _scope = store::scoped_dir(None);
    let mut s = Session::new();
    let e = s.execute("optics.downloadLensProfiles", &json!({})).unwrap_err().to_string();
    assert!(e.contains("no settings folder"), "{e}");
    assert!(s.lens_profiles.begin_download());
    let dir = temp("busy");
    let e = s.execute("optics.downloadLensProfiles", &json!({"dir": dir})).unwrap_err().to_string();
    assert!(e.contains("already downloading"), "{e}");
    // another session's download is its own
    assert!(Session::new().lens_profiles.begin_download());
    s.lens_profiles.end_download(None);
    assert!(!s.lens_profiles.downloading() && s.lens_profiles.take_outcome().is_none());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn download_base_is_https_or_loopback_only() {
    assert!(download::base_allowed(SOURCE));
    assert!(download::base_allowed("http://127.0.0.1:8080"));
    assert!(download::base_allowed("http://127.0.0.1/db"));
    assert!(!download::base_allowed("http://127.0.0.1.example.com/"));
    assert!(!download::base_allowed("http://127.0.0.1@example.com/"));
    assert!(!download::base_allowed("http://raw.githubusercontent.com/"));
}

#[test]
fn the_file_list_names_plain_xml_files_once_each() {
    let mut seen = std::collections::HashSet::new();
    for name in FILES {
        assert!(name.ends_with(".xml") && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')), "{name}");
        assert!(seen.insert(name), "{name} twice");
    }
    assert!(FILES.contains(&"slr-canon.xml"));
}

/// A lens folder with one profile ("Testcam A 18-55mm f/3.5-5.6"), a session whose files are "read" by a probe that
/// finds lens data in every file but those named `none*`, and the paths it was asked for.
fn finder(tag: &str) -> (store::ScopedDir, Session, Arc<std::sync::Mutex<Vec<String>>>, PathBuf) {
    let dir = temp(tag);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.xml"), file("A")).unwrap();
    let scope = store::scoped_dir(Some(dir.clone()));
    let mut s = Session::new();
    let asked = Arc::new(std::sync::Mutex::new(Vec::new()));
    let log = asked.clone();
    s.media.file_probe = Some(Arc::new(move |path: &str| {
        log.lock().unwrap().push(path.to_string());
        let lens = (!path.contains("none")).then(lens);
        Ok(crate::media::ProbeInfo { embedded_lens: lens, ..Default::default() })
    }));
    (scope, s, asked, dir)
}

fn set_lens_name(s: &mut Session, id: PhotoId, name: &str) {
    let mut meta = s.catalog.photo(id).unwrap().meta.clone();
    meta.lens = name.into();
    s.catalog.apply(Op::SetMeta { id, meta: Box::new(meta) }).unwrap();
}

#[test]
fn find_reads_the_files_of_raws_without_lens_data_and_records_what_it_finds() {
    let (_scope, mut s, asked, dir) = finder("find");
    let a = add_raw(&mut s, "a.CR2");
    let none = add_raw(&mut s, "none.CR2");
    // a virtual copy shows the same file: read once, both gain the data
    let copy = s.catalog.alloc_photo_id();
    let mut c = (**s.catalog.photo(a).unwrap()).clone();
    (c.id, c.copy_of) = (copy, Some(a));
    s.catalog.apply(Op::AddPhoto { photo: Box::new(c) }).unwrap();
    // a raw that has lens data, and a rendered file: not read
    let has = add_raw(&mut s, "has.CR2");
    s.catalog.apply(Op::SetEmbeddedLens { id: has, lens: Some(Box::new(lens())) }).unwrap();
    let jpg = s.catalog.alloc_photo_id();
    s.catalog
        .apply(Op::AddPhoto { photo: Box::new(Photo::new(jpg, Source::File { path: "/photos/c.jpg".into() }, "c.jpg", "JPEG", 64, 64, "")) })
        .unwrap();
    let undo = s.undo.len();

    let r = s.execute("optics.findLensProfiles", &json!({"ids": [a.0, none.0, copy.0, has.0, jpg.0]})).unwrap();
    assert_eq!((r["candidates"].as_u64(), r["changed"].as_u64(), r["started"].as_bool()), (Some(2), Some(2), Some(false)), "{r}");
    let mut read = asked.lock().unwrap().clone();
    read.sort();
    assert_eq!(read, ["/photos/a.CR2", "/photos/none.CR2"]);
    for id in [a, copy] {
        let p = s.catalog.photo(id).unwrap();
        assert!(p.embedded_lens == Some(lens()) && p.develop.optics.lens_profile && !p.is_edited(), "{id:?}");
    }
    assert!(s.catalog.photo(none).unwrap().embedded_lens.is_none());
    assert_eq!(s.undo.len(), undo, "bookkeeping, not an undo step");

    // nothing left to do for them; without a database or a file reader nothing is read at all
    asked.lock().unwrap().clear();
    let r = s.execute("optics.findLensProfiles", &json!({"ids": [a.0, copy.0, has.0]})).unwrap();
    assert_eq!((r["candidates"].as_u64(), asked.lock().unwrap().len()), (Some(0), 0), "{r}");
    store::use_dir(None);
    for background in [false, true] {
        let r = s.execute("optics.findLensProfiles", &json!({"ids": [none.0], "background": background})).unwrap();
        assert!(r["note"].as_str().is_some_and(|n| n.contains("no lens database")) && asked.lock().unwrap().is_empty(), "{r}");
        assert!(!s.lens_profiles.scanning() && s.activity.list().is_empty());
    }
    store::use_dir(Some(dir.clone()));
    s.media.file_probe = None;
    let r = s.execute("optics.findLensProfiles", &json!({"ids": [none.0]})).unwrap();
    assert!(r["note"].as_str().is_some_and(|n| n.contains("reads no files")), "{r}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Over a whole library only the files whose lens the database names are opened, so the photos it can't help are
/// not read again at every start.
#[test]
fn a_whole_library_lookup_reads_only_files_whose_lens_the_database_names() {
    let (_scope, mut s, asked, dir) = finder("all");
    let listed = add_raw(&mut s, "listed.CR2");
    set_lens_name(&mut s, listed, "A 18-55mm F3.5-5.6");
    let other = add_raw(&mut s, "other.CR2");
    set_lens_name(&mut s, other, "70-200mm F4");
    let unnamed = add_raw(&mut s, "unnamed.CR2");
    let r = s.execute("optics.findLensProfiles", &json!({"all": true})).unwrap();
    assert_eq!((r["candidates"].as_u64(), r["changed"].as_u64()), (Some(3), Some(1)), "{r}");
    assert_eq!(*asked.lock().unwrap(), ["/photos/listed.CR2"]);
    assert!(s.catalog.photo(listed).unwrap().embedded_lens.is_some());
    assert!(s.catalog.photo(other).unwrap().embedded_lens.is_none() && s.catalog.photo(unnamed).unwrap().embedded_lens.is_none());
    // asked for by id (the photo on screen), a file is read whatever the catalog calls its lens
    let r = s.execute("optics.findLensProfiles", &json!({"ids": [unnamed.0]})).unwrap();
    assert_eq!(r["changed"].as_u64(), Some(1), "{r}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_background_lookup_shows_in_the_activity_stack_and_is_recorded_when_it_ends() {
    let (_scope, mut s, asked, dir) = finder("background");
    let ids: Vec<PhotoId> = (0..5)
        .map(|i| {
            let id = add_raw(&mut s, &format!("p{i}.CR2"));
            set_lens_name(&mut s, id, "A 18-55mm F3.5-5.6");
            id
        })
        .collect();
    // the reader waits until the test has seen the row
    let gate = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (open, log) = (gate.clone(), asked.clone());
    s.media.file_probe = Some(Arc::new(move |path: &str| {
        while !open.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        log.lock().unwrap().push(path.to_string());
        Ok(crate::media::ProbeInfo { embedded_lens: Some(lens()), ..Default::default() })
    }));
    assert!(s.finish_lens_scan().is_none(), "nothing has run");
    let r = s.execute("optics.findLensProfiles", &json!({"all": true, "background": true})).unwrap();
    assert_eq!((r["started"].as_bool(), r["candidates"].as_u64()), (Some(true), Some(5)), "{r}");
    // the row appears once the lookup has files to read
    let started = std::time::Instant::now();
    while !s.activity.list().iter().any(|t| t.kind == "lensProfiles" && t.cancellable) {
        assert!(started.elapsed().as_secs() < 60, "no activity row: {:?}", s.activity.list());
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(s.lens_profiles.scanning());
    assert!(s.execute("optics.findLensProfiles", &json!({"all": true, "background": true})).is_err(), "one at a time");
    assert!(s.catalog.photo(ids[0]).unwrap().embedded_lens.is_none(), "nothing is recorded off the UI thread");
    gate.store(true, std::sync::atomic::Ordering::Relaxed);

    let gained = loop {
        if let Some(n) = s.finish_lens_scan() {
            break n;
        }
        assert!(started.elapsed().as_secs() < 60, "the lookup never ended");
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(gained, 5);
    assert!(ids.iter().all(|id| s.catalog.photo(*id).unwrap().embedded_lens == Some(lens())));
    assert!(!s.lens_profiles.scanning() && s.finish_lens_scan().is_none(), "taken once");
    while !s.activity.list().is_empty() && started.elapsed().as_secs() < 60 {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(s.activity.list().is_empty());

    // a lookup is its session's: another session never sees its result
    let more = add_raw(&mut s, "later.CR2");
    set_lens_name(&mut s, more, "A 18-55mm F3.5-5.6");
    s.execute("optics.findLensProfiles", &json!({"all": true, "background": true})).unwrap();
    while s.lens_profiles.scanning() {
        assert!(started.elapsed().as_secs() < 120);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(Session::new().finish_lens_scan().is_none());
    assert_eq!(s.finish_lens_scan(), Some(1));
    assert!(s.catalog.photo(more).unwrap().embedded_lens.is_some());
    let _ = std::fs::remove_dir_all(&dir);
}
