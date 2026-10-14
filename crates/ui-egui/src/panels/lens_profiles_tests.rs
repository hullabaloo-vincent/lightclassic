//! Lens profiles in the window, headless: the note under Enable Lens Corrections in both interface styles, the
//! download it offers (from a local server), and photos already in the library gaining their profile.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use lightcraft_catalog::{MediaKind, Op, Photo, PhotoId, Source};
use lightcraft_develop::{EmbeddedLens, EmbeddedWarp};
use lightcraft_engine::cmd::lens_profiles::FILES;
use lightcraft_engine::lens_profiles as store;
use serde_json::json;

use crate::headless::Headless;
use crate::state::{RightPanel, ViewMode};
use crate::{LightcraftApp, Services};

const T: Duration = Duration::from_secs(20);
const SETTLE: Duration = Duration::from_secs(120);
const SIZE: [f32; 2] = [1600.0, 1000.0];
const LENS: &str = "Testcam A 18-55mm f/3.5-5.6";

fn temp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lightcraft-ui-lens-profiles-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// A synthetic database file in the lensfun format (made-up coefficients, not lensfun data).
fn database() -> String {
    format!(
        r#"<lensdatabase version="2">
          <lens><maker>Testcam</maker><model>{LENS}</model><mount>T</mount><cropfactor>1.5</cropfactor>
            <calibration><distortion model="poly3" focal="18" k1="-0.03" /></calibration>
          </lens></lensdatabase>"#
    )
}

fn lens() -> EmbeddedLens {
    EmbeddedLens { warp: Some(EmbeddedWarp { planes: [[1.0, -0.1, 0.03, 0.0, 0.0, 0.0]; 3], ..Default::default() }), ..Default::default() }
}

/// A raw as an older session catalogued it (no lens data, correction off), with the lens name its file gave.
fn add_raw(app: &mut LightcraftApp, name: &str, lens_name: &str) -> PhotoId {
    let s = &mut app.session;
    let id = s.catalog.alloc_photo_id();
    let mut p = Photo::new(id, Source::File { path: format!("/photos/{name}") }, name, "CR2", 64, 64, "2026-01-01T00:00:00");
    p.kind = MediaKind::Raw;
    p.meta.lens = lens_name.into();
    p.develop = Arc::new(p.camera_defaults());
    s.catalog.apply(Op::AddPhoto { photo: Box::new(p) }).unwrap();
    id
}

/// The window on photo `id` with the lens panel open: Develop's Lens Corrections (Classic) or the Edit panel's
/// Optics section (Modern).
fn window(mut app: LightcraftApp, classic: bool, id: PhotoId) -> Headless {
    app.ui.settings.gpu = false;
    app.ui.view = ViewMode::PhotoGrid;
    app.ui.right = RightPanel::None;
    if classic {
        app.run("view.interface", json!({"style": "classic"})).expect("the style is set");
    }
    app.run("library.select", json!({"ids": [id.0], "active": id.0})).expect("the photo is selected");
    app.run(if classic { "module.develop" } else { "panel.edit" }, json!({})).expect("the edit tool opens");
    app.ui.open_sections = vec!["optics".to_string()];
    let mut h = Headless::new(app, SIZE, 1.0);
    assert!(h.settle(SETTLE), "the window never comes to rest");
    h
}

fn app() -> LightcraftApp {
    LightcraftApp::new(lightcraft_engine::Session::with_demo(), Services { png: None, ..Default::default() })
}

fn says(h: &Headless, part: &str) -> bool {
    h.painted_text().iter().any(|t| t.contains(part))
}

fn widget(h: &Headless, id: &str) -> Option<egui::Rect> {
    h.app.widgets.iter().find(|(w, _)| w == id).map(|(_, r)| *r)
}

/// Every file of the database, served from this machine.
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
            let name = line.split(' ').nth(1).unwrap_or("").trim_start_matches('/').to_string();
            let (status, body) = if FILES.contains(&name.as_str()) { ("200 OK", database()) } else { ("404 Not Found", "nope".to_string()) };
            let _ = c.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes());
            let _ = c.write_all(body.as_bytes());
        }
    });
    format!("http://127.0.0.1:{port}")
}

/// A raw without lens data and no lens database: both styles say where profiles come from and offer the download;
/// a host with nowhere to keep them (the library's own default) only says it. Before, both said "No lens data
/// embedded in this file (DNG lens opcodes only)", and only once the switch was on.
#[test]
fn a_raw_without_lens_data_is_offered_the_download_in_both_styles() {
    for classic in [true, false] {
        let dir = temp(if classic { "offer-classic" } else { "offer-modern" });
        let scope = store::scoped_dir(Some(dir.clone()));
        let mut a = app();
        let id = add_raw(&mut a, "a.CR2", LENS);
        let h = window(a, classic, id);
        assert!(says(&h, "Lens profiles are a separate download"), "classic {classic}: {:?}", h.painted_text());
        assert!(widget(&h, "button:lensProfilesDownload").is_some(), "classic {classic}");
        assert!(!says(&h, "DNG lens opcodes"));
        // nothing was started by looking
        assert!(!h.app.session.lens_profiles.downloading() && !dir.exists());
        drop(scope);

        let _none = store::scoped_dir(None);
        let mut a = app();
        let id = add_raw(&mut a, "a.CR2", LENS);
        let h = window(a, classic, id);
        assert!(says(&h, "Lens profiles are a separate download") && widget(&h, "button:lensProfilesDownload").is_none(), "classic {classic}");
    }
}

/// A rendered file says why it has no profile only when the switch is on; a photo with lens data names its lens and
/// what is corrected.
#[test]
fn the_note_names_the_lens_and_what_is_corrected() {
    let _scope = store::scoped_dir(None);
    let mut a = app();
    let id = add_raw(&mut a, "a.CR2", "EF28-135mm f/3.5-5.6 IS USM");
    a.session.catalog.apply(Op::SetEmbeddedLens { id, lens: Some(Box::new(lens())) }).unwrap();
    let mut h = window(a, true, id);
    assert!(says(&h, "EF28-135mm f/3.5-5.6 IS USM"), "{:?}", h.painted_text());
    assert!(says(&h, "Lens profile corrects: distortion"), "{:?}", h.painted_text());
    assert!(!says(&h, "vignetting,") && !says(&h, "separate download") && widget(&h, "button:lensProfilesDownload").is_none());
    // its amount sliders: the one with nothing to scale is not offered
    h.app.session.execute("develop.merge", &json!({"settings": {"optics": {"lens_profile": true}}})).unwrap();
    assert!(super::scales(&h.app, id, "optics.profileDistortion") && !super::scales(&h.app, id, "optics.profileVignetting"));
    assert!(!super::scales(&h.app, PhotoId(u64::MAX), "optics.profileDistortion"), "no such photo");

    // a demo photo is not a raw file
    let mut a = app();
    let jpg = a.session.visible_cloned()[0];
    let mut h = window(a, true, jpg);
    assert!(!says(&h, "Lens profiles apply to raw photos"), "quiet while the switch is off");
    let r = h.request("engine.execute", json!({"command": "develop.merge", "params": {"settings": {"optics": {"lens_profile": true}}}}), T);
    assert_eq!(r["ok"], true, "{r}");
    h.settle(SETTLE);
    assert!(says(&h, "Lens profiles apply to raw photos"), "{:?}", h.painted_text());
}

/// The whole path: the button starts the download (shown in the activity stack), the database is installed in the
/// host's folder, and the photos of the library whose lens it names are read and gain their profile, the one on
/// screen included; the correction is on for them because they were still at their import settings.
#[test]
fn the_download_installs_the_database_and_the_librarys_photos_gain_their_profiles() {
    let dir = temp("download");
    let _scope = store::scoped_dir(Some(dir.clone()));
    store::use_source(Some(serve()));
    let mut a = app();
    let shown = add_raw(&mut a, "shown.CR2", LENS);
    let other = add_raw(&mut a, "other.CR2", LENS);
    let unlisted = add_raw(&mut a, "unlisted.CR2", "Otherco 70-200mm f/4");
    let read = Arc::new(std::sync::Mutex::new(Vec::new()));
    let log = read.clone();
    a.session.media.file_probe = Some(Arc::new(move |path: &str| {
        log.lock().unwrap().push(path.to_string());
        let found = (!path.contains("unlisted")).then(lens);
        Ok(lightcraft_engine::media::ProbeInfo { embedded_lens: found, ..Default::default() })
    }));
    let mut h = window(a, true, shown);
    assert!(read.lock().unwrap().is_empty(), "no database: no file is read");

    let r = h.request("ui.clickWidget", json!({"id": "button:lensProfilesDownload"}), T);
    assert_eq!(r["ok"], true, "{r}");
    let has_lens = |h: &Headless, id: PhotoId| h.app.session.catalog.photo(id).is_some_and(|p| p.embedded_lens.is_some());
    assert!(h.step_until(SETTLE, |h| has_lens(h, shown) && has_lens(h, other)), "the photos never gained their profile");
    h.settle(SETTLE);

    assert_eq!(std::fs::read_dir(&dir).unwrap().flatten().filter(|e| e.path().extension().is_some_and(|x| x == "xml")).count(), FILES.len());
    for id in [shown, other] {
        let p = h.app.session.catalog.photo(id).unwrap();
        assert!(p.develop.optics.lens_profile && !p.is_edited(), "{id:?}: on, as import would have left it");
    }
    assert!(!has_lens(&h, unlisted));
    assert!(!read.lock().unwrap().iter().any(|p| p.contains("unlisted")), "a file whose lens the database does not name is not read in bulk");
    assert!(says(&h, "Lens profile corrects: distortion"), "{:?}", h.painted_text());
    assert!(widget(&h, "button:lensProfilesDownload").is_none());
    assert!(h.app.session.activity.list().is_empty(), "{:?}", h.app.session.activity.list());

    // the photo the database can't help says so, after its own file was read once
    let r = h.request("engine.execute", json!({"command": "library.select", "params": {"ids": [unlisted.0], "active": unlisted.0}}), T);
    assert_eq!(r["ok"], true, "{r}");
    h.settle(SETTLE);
    assert!(says(&h, "The lens database has no profile for Otherco 70-200mm f/4"), "{:?}", h.painted_text());
    let reads = |read: &Arc<std::sync::Mutex<Vec<String>>>| read.lock().unwrap().iter().filter(|p| p.contains("unlisted")).count();
    assert_eq!(reads(&read), 1);
    for _ in 0..5 {
        h.step();
    }
    assert_eq!(reads(&read), 1, "read once, not every frame");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A window opened on a library whose photos predate the installed database catches them up by itself.
#[test]
fn a_library_opened_with_the_database_installed_is_caught_up_in_the_background() {
    let dir = temp("catch-up");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.xml"), database()).unwrap();
    let _scope = store::scoped_dir(Some(dir.clone()));
    let mut a = app();
    let old = add_raw(&mut a, "old.CR2", LENS);
    a.session.media.file_probe =
        Some(Arc::new(|_: &str| Ok(lightcraft_engine::media::ProbeInfo { embedded_lens: Some(lens()), ..Default::default() })));
    // the grid, nothing selected: no panel asks for it
    a.ui.settings.gpu = false;
    let mut h = Headless::new(a, SIZE, 1.0);
    assert!(h.step_until(SETTLE, |h| h.app.session.catalog.photo(old).is_some_and(|p| p.embedded_lens.is_some())), "never caught up");
    assert!(h.app.session.catalog.photo(old).unwrap().develop.optics.lens_profile);
    let _ = std::fs::remove_dir_all(&dir);
}
