//! The Print module's layout and its commands (`print.set`, `print.export`); the drawing is in
//! `panels/classic/print.rs`.
//!
//! Print lays a photo out on a page and writes the page as an image file, which is what Lightroom Classic's Print
//! module does with "Print to: JPEG File". The page is the export engine's [`Canvas`]: its size, its background, the
//! photo's share of it and where the photo's centre sits. Everything about the placement is a fraction of the page, so
//! the preview on screen and the file written are the same layout at different sizes.

use lightcraft_engine::export::{CANVAS_EDGE_RANGE, CANVAS_SIZE_RANGE, Canvas, ExportFormat};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::LightcraftApp;

/// The file a page is written as.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrintFormat {
    #[default]
    Jpeg,
    Png,
    Tiff,
}

impl PrintFormat {
    pub const ALL: [Self; 3] = [Self::Jpeg, Self::Png, Self::Tiff];

    pub fn id(self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::Tiff => "tiff",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Jpeg => "JPEG",
            Self::Png => "PNG",
            Self::Tiff => "TIFF",
        }
    }

    pub fn export(self) -> ExportFormat {
        match self {
            Self::Jpeg => ExportFormat::Jpeg,
            Self::Png => ExportFormat::Png,
            Self::Tiff => ExportFormat::Tiff,
        }
    }
}

/// The page shapes the Layout panel offers: (key, label, width : height).
pub const PAGE_SHAPES: [(&str, &str, u32, u32); 9] = [
    ("1x1", "1 : 1", 1, 1),
    ("4x5", "4 : 5", 4, 5),
    ("5x4", "5 : 4", 5, 4),
    ("2x3", "2 : 3", 2, 3),
    ("3x2", "3 : 2", 3, 2),
    ("9x16", "9 : 16", 9, 16),
    ("16x9", "16 : 9", 16, 9),
    ("a4", "A4", 210, 297),
    ("a4wide", "A4 wide", 297, 210),
];

/// The backgrounds the Page panel offers: (key, label, sRGB).
pub const PAGE_COLOURS: [(&str, &str, [u8; 3]); 4] =
    [("white", "White", [255, 255, 255]), ("light", "Light", [224, 224, 224]), ("grey", "Grey", [128, 128, 128]), ("black", "Black", [0, 0, 0])];

/// The Print module's page: where a photo goes on it, and the file it is written as.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PrintLayout {
    /// The page and the photo on it, as the export engine takes them.
    pub canvas: Canvas,
    pub format: PrintFormat,
    /// JPEG quality, 1–100.
    pub quality: u8,
    /// The folder the files are written to (empty: the last export's, else the default export folder).
    pub dir: String,
}

impl Default for PrintLayout {
    fn default() -> Self {
        Self { canvas: Canvas::default(), format: PrintFormat::Jpeg, quality: 92, dir: String::new() }
    }
}

/// A `long`-edge page of shape `w : h`, within the engine's limits.
pub fn shaped(long: u32, w: u32, h: u32) -> (u32, u32) {
    let (lo, hi) = CANVAS_EDGE_RANGE;
    let long = long.clamp(lo, hi) as f64;
    let (w, h) = (w.max(1) as f64, h.max(1) as f64);
    let short = |ratio: f64| ((long * ratio).round() as u32).clamp(lo, hi);
    if w >= h { (long as u32, short(h / w)) } else { (short(w / h), long as u32) }
}

impl PrintLayout {
    /// The page's longer side in pixels.
    pub fn long_edge(&self) -> u32 {
        self.canvas.width.max(self.canvas.height)
    }

    /// The key of the [`PAGE_SHAPES`] entry the page has (to within a pixel), if it has one.
    pub fn shape(&self) -> Option<&'static str> {
        PAGE_SHAPES.iter().find(|(_, _, w, h)| shaped(self.long_edge(), *w, *h) == (self.canvas.width, self.canvas.height)).map(|(key, ..)| *key)
    }

    /// These settings as `app.export` options (without a destination).
    pub fn export_params(&self) -> Value {
        json!({"format": self.format.id(), "quality": self.quality, "canvas": self.canvas, "naming": "{name}-print"})
    }

    /// The x (or y) range that keeps a photo `photo` pixels wide (tall) on a page `page` pixels wide (tall): its
    /// centre can go from half its width in to half its width from the far edge; a photo as wide as the page stays
    /// in the middle.
    pub fn centre_range(photo: usize, page: u32) -> (f32, f32) {
        let half = (photo as f32 / 2.0 / page.max(1) as f32).min(0.5);
        (half, 1.0 - half)
    }
}

const SET_KEYS: &[&str] = &["shape", "longEdge", "width", "height", "background", "size", "x", "y", "format", "quality", "dir"];

/// `print.set {shape?, longEdge?, width?, height?, background?, size?, x?, y?, format?, quality?, dir?}`: change the
/// page. Nothing changes unless all of it can be applied.
pub(super) fn set(app: &mut LightcraftApp, p: &Value) -> Result<Value, String> {
    let bad = |msg: String| format!("print.set: {msg}");
    let Some(o) = p.as_object() else { return Err(bad("params must be an object".into())) };
    if let Some(k) = o.keys().find(|k| !SET_KEYS.contains(&k.as_str())) {
        return Err(bad(format!("unknown `{k}` ({})", SET_KEYS.join(", "))));
    }
    let given = |k: &str| o.get(k).filter(|v| !v.is_null());
    let number = |k: &str| -> Result<Option<f64>, String> {
        given(k).map(|v| v.as_f64().filter(|f| f.is_finite()).ok_or_else(|| bad(format!("`{k}` must be a number")))).transpose()
    };
    let pixels = |k: &str| -> Result<Option<u32>, String> {
        let (lo, hi) = CANVAS_EDGE_RANGE;
        match number(k)? {
            Some(v) if v.fract() == 0.0 && (lo as f64..=hi as f64).contains(&v) => Ok(Some(v as u32)),
            Some(_) => Err(bad(format!("`{k}` must be a whole number of pixels, {lo}..{hi}"))),
            None => Ok(None),
        }
    };
    let mut layout = app.ui.classic.print.clone();
    let c = &mut layout.canvas;
    if let Some(v) = pixels("width")? {
        c.width = v;
    }
    if let Some(v) = pixels("height")? {
        c.height = v;
    }
    // a shape keeps the long edge; a long edge keeps the shape
    let long = pixels("longEdge")?;
    match given("shape") {
        Some(v) => {
            let keys = || PAGE_SHAPES.map(|(key, ..)| key).join("|");
            let key = v.as_str().ok_or_else(|| bad(format!("`shape` is one of {}", keys())))?;
            let (_, _, w, h) = PAGE_SHAPES.iter().find(|(k, ..)| *k == key).ok_or_else(|| bad(format!("unknown shape `{key}` ({})", keys())))?;
            (c.width, c.height) = shaped(long.unwrap_or(c.width.max(c.height)), *w, *h);
        }
        None => {
            if let Some(long) = long {
                (c.width, c.height) = shaped(long, c.width, c.height);
            }
        }
    }
    if let Some(v) = given("background") {
        c.background = match v {
            Value::String(name) => PAGE_COLOURS
                .iter()
                .find(|(key, ..)| key == name)
                .map(|(_, _, rgb)| *rgb)
                .ok_or_else(|| bad(format!("unknown background `{name}` ({}, or [r, g, b])", PAGE_COLOURS.map(|(key, ..)| key).join("|"))))?,
            other => serde_json::from_value::<[u8; 3]>(other.clone())
                .map_err(|_| bad("`background` is [r, g, b], each 0..255, or a colour's name".into()))?,
        };
    }
    for (k, field) in [("size", &mut c.size), ("x", &mut c.x), ("y", &mut c.y)] {
        if let Some(v) = number(k)? {
            *field = v as f32;
        }
    }
    layout.canvas = layout.canvas.checked().map_err(bad)?;
    if let Some(v) = given("format") {
        let name = v.as_str().unwrap_or_default();
        layout.format = PrintFormat::ALL
            .into_iter()
            .find(|f| f.id() == name)
            .ok_or_else(|| bad(format!("`format` is one of {}", PrintFormat::ALL.map(PrintFormat::id).join("|"))))?;
    }
    if let Some(v) = number("quality")? {
        if v.fract() != 0.0 || !(1.0..=100.0).contains(&v) {
            return Err(bad("`quality` must be a whole number 1..100".into()));
        }
        layout.quality = v as u8;
    }
    if let Some(v) = given("dir") {
        layout.dir = v.as_str().ok_or_else(|| bad("`dir` must be a string".into()))?.trim().to_string();
    }
    app.ui.classic.print = layout;
    Ok(json!(app.ui.classic.print))
}

/// The photo's size range as percent, for the panel's slider.
pub fn size_percent_range() -> (f64, f64) {
    ((CANVAS_SIZE_RANGE.0 * 100.0).ceil() as f64, (CANVAS_SIZE_RANGE.1 * 100.0) as f64)
}

/// `print.export {dir?, path?, ids?, background?}`: write the page with each selected photo on it, one file a photo
/// (`ids`: default the selection, else the active photo). `dir` is remembered as the Print folder; `path` names the
/// file of a single photo instead. In the background where the host can (`background: false` waits for it).
pub(super) fn export(app: &mut LightcraftApp, p: &Value) -> Result<Value, String> {
    use lightcraft_engine::export::{Destination, ExportOptions, export_batch, prepare_batch};
    const KEYS: &[&str] = &["dir", "path", "ids", "background"];
    let bad = |msg: String| format!("print.export: {msg}");
    let Some(o) = p.as_object() else { return Err(bad("params must be an object".into())) };
    if let Some(k) = o.keys().find(|k| !KEYS.contains(&k.as_str())) {
        return Err(bad(format!("unknown `{k}` ({})", KEYS.join(", "))));
    }
    let text = |k: &str| o.get(k).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let opts = ExportOptions::from_params(&app.ui.classic.print.export_params()).map_err(|e| e.to_string())?;
    let ids: Vec<lightcraft_catalog::PhotoId> = match o.get("ids").and_then(Value::as_array) {
        Some(a) => a.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect(),
        None if !app.session.selection.ids.is_empty() => {
            // in the grid's order, as an export
            let selected: std::collections::HashSet<_> = app.session.selection.ids.iter().copied().collect();
            let mut ids: Vec<_> = app.session.visible().iter().copied().filter(|id| selected.contains(id)).collect();
            let shown: std::collections::HashSet<_> = ids.iter().copied().collect();
            ids.extend(app.session.selection.ids.iter().filter(|id| !shown.contains(id)));
            ids
        }
        None => app.session.active().into_iter().collect(),
    };
    if ids.is_empty() {
        return Err("no photo selected".into());
    }
    let exact = text("path");
    if let Some(dir) = text("dir") {
        app.ui.classic.print.dir = dir;
    }
    let dir = folder(app);
    if dir.is_empty() && exact.is_none() {
        return Err("Choose a folder to print to.".into());
    }
    let to = Destination { dir: dir.clone(), exact };
    let after = app.session.after_export(&opts)?;
    let background = o.get("background").and_then(Value::as_bool).unwrap_or(true) && app.services.write_shared.is_some();
    if background {
        let items = prepare_batch(&mut app.session, &ids, &opts)?;
        return crate::export_task::start(app, items, opts, to, after);
    }
    let write = app.services.write.as_mut().ok_or("no writer")?;
    let files = export_batch(&mut app.session, &ids, &opts, &to, &mut |path, bytes| write(path, bytes), &|path| std::path::Path::new(path).exists())?;
    Ok(json!({"files": files, "dir": dir}))
}

/// The folder Print writes to: the one chosen, else the last export's, else the default export folder.
pub fn folder(app: &LightcraftApp) -> String {
    let chosen = app.ui.classic.print.dir.trim();
    if !chosen.is_empty() {
        return chosen.to_string();
    }
    let last = app.session.last_export.as_ref().and_then(|l| l.get("dir")).and_then(Value::as_str).map(str::trim).filter(|d| !d.is_empty());
    last.map(str::to_string).unwrap_or_else(crate::control::default_export_dir)
}
