//! A canvas the exported photo is placed on (the Print module's page): this fork's addition to the export options.

use lightcraft_raster::Rgba8;
use serde::{Deserialize, Serialize};

use super::ExportFormat;
use super::params::unknown_key;

/// A canvas the exported photo is placed on, as a print layout places a photo on a page: the file is the canvas,
/// with the photo scaled to a share of it and set where the layout says, on a background colour.
///
/// The layout is in fractions of the canvas, so it means the same picture at any canvas size.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Canvas {
    /// The canvas in pixels ([`CANVAS_EDGE_RANGE`] each way, [`CANVAS_MAX_PIXELS`] in all).
    pub width: u32,
    pub height: u32,
    /// The colour around the photo (sRGB).
    pub background: [u8; 3],
    /// How large the photo is: it is fitted inside this fraction of the canvas, so 1 reaches the canvas edges on
    /// the tighter side ([`CANVAS_SIZE_RANGE`]).
    pub size: f32,
    /// Where the photo's centre sits, as a fraction of the canvas across and down (0.5, 0.5 is the middle). What
    /// reaches past the canvas is cut off.
    pub x: f32,
    pub y: f32,
}

/// A canvas edge in pixels.
pub const CANVAS_EDGE_RANGE: (u32, u32) = (16, 20_000);
/// The most pixels a canvas may have (its 8-bit image is four bytes a pixel).
pub const CANVAS_MAX_PIXELS: u64 = 120_000_000;
/// The photo's share of the canvas.
pub const CANVAS_SIZE_RANGE: (f32, f32) = (0.02, 1.0);

impl Default for Canvas {
    fn default() -> Self {
        Self { width: 3000, height: 3000, background: [255, 255, 255], size: 0.85, x: 0.5, y: 0.5 }
    }
}

impl Canvas {
    /// The keys of a `canvas` object.
    pub const KEYS: &'static [&'static str] = &["width", "height", "background", "size", "x", "y"];

    /// Whether a canvas can be written as `format`: the formats rendered in 8 bits per channel.
    pub fn takes(format: ExportFormat) -> bool {
        format.is_rendered() && format != ExportFormat::RenderedDng
    }

    /// This canvas when it can be made, else what is wrong with it.
    pub fn checked(self) -> Result<Self, String> {
        let (lo, hi) = CANVAS_EDGE_RANGE;
        if !(lo..=hi).contains(&self.width) || !(lo..=hi).contains(&self.height) {
            return Err(format!("the canvas is {} × {} px: each side must be {lo}..{hi} px", self.width, self.height));
        }
        if self.width as u64 * self.height as u64 > CANVAS_MAX_PIXELS {
            return Err(format!("the canvas is {} × {} px: at most {} megapixels", self.width, self.height, CANVAS_MAX_PIXELS / 1_000_000));
        }
        let (lo, hi) = CANVAS_SIZE_RANGE;
        if !(lo..=hi).contains(&self.size) {
            return Err(format!("`size` must be {lo}..{hi} (the photo's share of the canvas)"));
        }
        if !(0.0..=1.0).contains(&self.x) || !(0.0..=1.0).contains(&self.y) {
            return Err("`x` and `y` must be 0..1 (the photo's centre across and down the canvas)".to_string());
        }
        Ok(self)
    }

    /// Read a `canvas` object strictly: an unknown key, a value of the wrong kind or one out of range is an error.
    pub fn from_json(v: &serde_json::Value) -> Result<Self, String> {
        let Some(o) = v.as_object() else { return Err("must be an object {width, height, background?, size?, x?, y?}".to_string()) };
        if let Some(k) = o.keys().find(|k| !Self::KEYS.contains(&k.as_str())) {
            return Err(unknown_key("key", k, Self::KEYS));
        }
        serde_json::from_value::<Self>(v.clone()).map_err(|e| e.to_string())?.checked()
    }

    /// The photo's size on the canvas in pixels, for a `w × h` (cropped) photo: its shape fitted inside
    /// [`Self::size`] of the canvas.
    pub fn photo_size(&self, w: f64, h: f64) -> (usize, usize) {
        let (w, h) = (w.max(1.0), h.max(1.0));
        let (lo, hi) = CANVAS_SIZE_RANGE;
        let size = if self.size.is_finite() { self.size.clamp(lo, hi) as f64 } else { 1.0 };
        let k = (self.width.max(1) as f64 * size / w).min(self.height.max(1) as f64 * size / h);
        (((w * k).round() as usize).max(1), ((h * k).round() as usize).max(1))
    }

    /// Where a photo of `w × h` pixels starts on the canvas (its left and top; negative when it reaches past).
    pub fn photo_origin(&self, w: usize, h: usize) -> (i64, i64) {
        let at = |centre: f32, canvas: u32, photo: usize| {
            let centre = if centre.is_finite() { centre.clamp(0.0, 1.0) as f64 } else { 0.5 };
            (centre * canvas as f64 - photo as f64 / 2.0).round() as i64
        };
        (at(self.x, self.width, w), at(self.y, self.height, h))
    }

    /// The canvas with `photo` on it, `background` (already in the image's colour space) around it.
    pub fn place(&self, photo: &Rgba8, background: [u8; 3]) -> Rgba8 {
        // a canvas that was not `checked` is still never larger than the limits
        let (cw, ch) = (self.width.clamp(1, CANVAS_EDGE_RANGE.1) as usize, self.height.clamp(1, CANVAS_EDGE_RANGE.1) as usize);
        let mut out = Rgba8 { width: cw, height: ch, data: vec![[background[0], background[1], background[2], 255]; cw * ch] };
        let (left, top) = self.photo_origin(photo.width, photo.height);
        // the photo's columns that land on the canvas
        let from = (-left).clamp(0, photo.width as i64) as usize;
        let to = (cw as i64 - left).clamp(0, photo.width as i64) as usize;
        if from >= to {
            return out;
        }
        for row in 0..photo.height {
            let y = top + row as i64;
            if y < 0 || y >= ch as i64 {
                continue;
            }
            let start = y as usize * cw + (left + from as i64) as usize;
            let (Some(src), Some(dst)) =
                (photo.data.get(row * photo.width + from..row * photo.width + to), out.data.get_mut(start..start + (to - from)))
            else {
                continue;
            };
            dst.copy_from_slice(src);
        }
        out
    }
}
