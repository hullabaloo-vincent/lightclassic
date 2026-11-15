//! White-balance presets of Canon CR2 bodies whose `ColorData` layout `craft-raw` doesn't read (this fork).
//!
//! `craft-raw` reads the planckian presets of `ColorData` versions -4 and 13–15 (`vendor::cr2`). The EOS Rebel T5
//! (EOS 1200D, Kiss X70) writes version 10 with 1275 words: the layout the ExifTool Canon tag-name documentation calls
//! `ColorData6` (versions 10, 1273 or 1275 words). Version 10 is also `ColorData7` (1312 words and more, other bodies),
//! where the presets sit elsewhere, so the length decides. Its 5-word records (R, G, G, B levels, then the temperature):
//! Daylight at word 103 (temperature at 107), Shade 108 (112), Cloudy 113 (117), Tungsten 118 (122), Fluorescent 123,
//! Kelvin 128, Flash 133 (137). Fluorescent (a green light off the locus) and the Kelvin row (the dial's last setting:
//! Daylight's levels at 5213 K on these files) are left out, as `craft-raw` does.
//!
//! Checked on 20 CR2s of one Rebel T5 (an owner's photos, never committed; numbers only): every file has the same
//! presets (Daylight 5200 K, Shade 7000 K, Cloudy 6000 K, Tungsten 3200 K, Flash 6363 K), and each file's as-shot
//! levels, read on the curve through them ([`KelvinPresets::temp_tint`]), land within 16 mired of the camera's own
//! `ColorTempAsShot` (word 67): −1.6 to +3.3 % on the 15 frames in daylight, +3.4 to +5.2 % on four tungsten interiors
//! (the camera says 3200 K for all four) and −8.8 % on one at 6454 K. Lightroom Classic's own as-shot readings of 11 of
//! the same files (from the owner's catalog; compared, not fitted to) differ from these by −0.5 to +1.0 % in daylight,
//! −3.6 to −4.7 % on the interiors and −5.9 % on the 6454 K frame, with tints 1–7 lower.
use craft_tiff::{ByteOrder, Tiff, Value, makernote, tags as t};

use crate::RawFormat;
use crate::RawImage;
use crate::kelvin_presets::KelvinPresets;

/// Canon's `ColorData` maker-note tag.
const COLOR_DATA: u16 = 0x4001;

/// `ColorData6` as (version, lengths, preset rows: the word of each preset's `R G G B` levels and of its temperature).
const LAYOUTS: &[(i16, &[usize], [(usize, usize); 5])] = &[(10, &[1273, 1275], [(103, 107), (108, 112), (113, 117), (118, 122), (133, 137)])];

/// Gives a CR2 whose `ColorData` `craft-raw` doesn't read its presets ([`kelvin_presets`]).
pub(crate) fn fill(bytes: &[u8], img: &mut RawImage) {
    if img.format == RawFormat::Cr2 && img.color.kelvin_presets.is_none() {
        img.color.kelvin_presets = kelvin_presets(bytes);
    }
}

/// The presets of a CR2's `ColorData` of a [`LAYOUTS`] version, as a Kelvin curve; `None` otherwise.
pub(crate) fn kelvin_presets(bytes: &[u8]) -> Option<KelvinPresets> {
    let tiff = Tiff::parse(bytes).ok()?;
    let make = tiff.ifds.first().and_then(|i| i.string(t::MAKE)).unwrap_or_default();
    let entry = tiff.exif()?.get(t::MAKER_NOTE)?;
    let note = makernote::parse_makernote(bytes, entry.offset, entry.count() as u64, tiff.order, &make)?;
    from_color_data(&words(note.ifd.value(COLOR_DATA)?, note.order))
}

/// The 16-bit words of `ColorData` (SHORT, or bytes paired up in the maker note's order).
fn words(v: &Value, order: ByteOrder) -> Vec<u64> {
    match v {
        Value::Byte(b) | Value::Undefined(b) => b.as_chunks::<2>().0.iter().map(|c| u64::from(order.u16(*c))).collect(),
        _ => v.to_u64_vec(),
    }
}

/// [`kelvin_presets`] from the `ColorData` words.
pub(crate) fn from_color_data(v: &[u64]) -> Option<KelvinPresets> {
    let version = *v.first()? as u16 as i16;
    let (_, _, rows) = LAYOUTS.iter().find(|(ver, lengths, _)| *ver == version && lengths.contains(&v.len()))?;
    let points = rows.iter().filter_map(|&(at, temp_at)| {
        let q = v.get(at..at.checked_add(4)?)?;
        let temp = *v.get(temp_at)? as f32;
        let wb = multipliers(q[0] as f32, q[1] as f32, q[2] as f32, q[3] as f32)?;
        (1000.0..=20000.0).contains(&temp).then_some((temp, wb))
    });
    KelvinPresets::from_unordered(points.collect::<Vec<_>>())
}

/// White-balance multipliers (green = 1) from one set of `R G G B` levels, when they are plausible.
fn multipliers(r: f32, g1: f32, g2: f32, b: f32) -> Option<[f32; 3]> {
    if g1 < 256.0 || g2 < 256.0 || r < 64.0 || b < 64.0 || g1 > 16384.0 || (g1 - g2).abs() > 0.05 * g1 {
        return None;
    }
    let g = (g1 + g2) / 2.0;
    let (r, b) = (r / g, b / g);
    ((0.25..6.0).contains(&r) && (0.25..6.0).contains(&b)).then_some([r, 1.0, b])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ColorData` of `len` words, version `version`, with the Rebel T5's presets in the `ColorData6` rows.
    fn rebel_t5(version: i16, len: usize) -> Vec<u64> {
        let mut v = vec![0u64; len];
        v[0] = u64::from(version as u16);
        // Daylight, Shade, Cloudy, Tungsten, Fluorescent, Kelvin, Flash: as the camera writes them
        let rows = [
            (103, [2013, 1024, 1024, 1572, 5200]),
            (108, [2320, 1024, 1024, 1341, 7000]),
            (113, [2166, 1024, 1024, 1448, 6000]),
            (118, [1448, 1024, 1024, 2351, 3200]),
            (123, [1774, 1024, 1024, 2226, 3729]),
            (128, [2013, 1024, 1024, 1572, 5213]),
            (133, [2250, 1024, 1024, 1429, 6363]),
        ];
        for (at, words) in rows {
            if len >= at + 5 {
                v[at..at + 5].copy_from_slice(&words);
            }
        }
        v
    }

    #[test]
    fn the_rebel_t5_presets_make_a_kelvin_curve() {
        let p = from_color_data(&rebel_t5(10, 1275)).expect("presets");
        assert!(from_color_data(&rebel_t5(10, 1273)).is_some(), "the other ColorData6 length");
        // each planckian preset's own levels read at its temperature, on the curve
        for (r, b, kelvin) in
            [(1448.0, 2351.0, 3200.0), (2013.0, 1572.0, 5200.0), (2166.0, 1448.0, 6000.0), (2250.0, 1429.0, 6363.0), (2320.0, 1341.0, 7000.0)]
        {
            let (temp, tint) = p.temp_tint([r / 1024.0, 1.0, b / 1024.0]).expect("usable");
            assert!((temp - kelvin).abs() < 1.0 && tint.abs() < 0.5, "{kelvin}: {temp} {tint}");
        }
        // Fluorescent is not a point: its green-shifted levels read off the curve
        let (_, tint) = p.temp_tint([1774.0 / 1024.0, 1.0, 2226.0 / 1024.0]).expect("usable");
        assert!(tint.abs() > 5.0, "{tint}");
    }

    /// The as-shot levels (R, B over G = 1024) of the owner's 20 frames (17 different ones) and the camera's own
    /// `ColorTempAsShot` for each, read on the curve within 16 mired of it.
    #[test]
    fn the_rebel_t5_as_shot_levels_read_near_the_cameras_temperature() {
        let p = from_color_data(&rebel_t5(10, 1275)).expect("presets");
        let frames = [
            (1556, 2608, 3200),
            (1533, 2551, 3200),
            (1547, 2583, 3200),
            (1579, 2661, 3200),
            (1920, 1667, 4812),
            (1928, 1675, 4801),
            (2185, 1515, 5800),
            (1945, 1691, 4801),
            (1903, 1657, 4801),
            (1990, 1725, 4801),
            (1956, 1697, 4801),
            (2040, 1608, 5169),
            (1953, 1694, 4801),
            (1942, 1686, 4801),
            (1935, 1680, 4801),
            (2040, 1553, 5346),
            (2175, 1376, 6454),
        ];
        for (r, b, camera) in frames {
            let (temp, tint) = p.temp_tint([r as f32 / 1024.0, 1.0, b as f32 / 1024.0]).expect("usable");
            let mired = 1e6 / temp - 1e6 / f64::from(camera);
            assert!(mired.abs() < 16.0 && tint.abs() < 40.0, "{r} {b}: {temp} K (tint {tint}) against the camera's {camera} K");
        }
    }

    /// Version 10 of another length is `ColorData7`, whose presets sit elsewhere; other versions are `craft-raw`'s.
    #[test]
    fn other_layouts_give_no_presets() {
        assert!(from_color_data(&rebel_t5(10, 1312)).is_none(), "ColorData7");
        assert!(from_color_data(&rebel_t5(9, 1275)).is_none());
        assert!(from_color_data(&rebel_t5(13, 1275)).is_none());
    }

    /// Short, empty or garbled arrays and files that aren't CR2s give nothing and never panic.
    #[test]
    fn hostile_color_data_gives_no_presets() {
        assert!(from_color_data(&[]).is_none());
        assert!(from_color_data(&[10]).is_none());
        let mut flat = rebel_t5(10, 1275);
        for w in &mut flat[103..138] {
            *w = 1024;
        }
        assert!(from_color_data(&flat).is_none(), "no curve");
        let mut huge = rebel_t5(10, 1275);
        huge[103..107].copy_from_slice(&[u64::MAX, 1024, 1024, u64::MAX]);
        huge[107] = u64::MAX;
        assert!(from_color_data(&huge).is_some(), "an unusable row is dropped, the others still make a curve");
        assert!(kelvin_presets(b"").is_none());
        assert!(kelvin_presets(b"II*\0\x08\0\0\0\0\0").is_none());
    }
}
