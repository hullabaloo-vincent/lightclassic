//! Dark frames that keep their colour (this fork, `LR-PROF-CAMERACOLOR`; first landed as `25e0419a` before upstream
//! split `camera_preview.rs`, and kept in its own file so upstream's changes merge around it).
//!
//! Two gates of the look fit read a dim, colourful frame (an EOS Rebel T5 church interior at ISO 3200) as grey:
//! a pixel counted as coloured only by its channel spread, and a partial look had to cut the linear error that a
//! few lamp pixels hold. Both are relaxed here for such frames only.

/// Linear channel spread above which a camera JPEG pixel counts as coloured when a colour matrix is learnt from the
/// photo ([`Coloured::Learnable`]).
pub(super) const COLOUR_SPREAD: f32 = 0.05;

/// ...or, for a pixel too dark to reach that spread, this share of its brightest channel. A spread measures
/// brightness times saturation: 0.05 is a tenth of a pixel whose brightest channel is 0.5, 0.28 of one at 0.18, and
/// more than a pixel below 0.05 has. A dim interior whose every pixel is a saturated gold or red therefore counted
/// as colourless (an EOS Rebel T5 church ceiling at ISO 3200: one pair in 635 had the spread, all 635 a saturation
/// above 0.4), no colour was learnt from it and it rendered grey-brown. 0.3 is what the spread asks at 0.17, just
/// below mid-grey: a darker pixel needs that saturation, a brighter one the spread as before.
pub(super) const DARK_COLOUR_SATURATION: f32 = 0.3;

/// Share of the neutral fallback's held-out squared error, in linear display values, that a `fit_partial` look may
/// keep on its second pass: it must not be further from the camera JPEG, while in gamma-encoded values (closer to
/// what is seen) it must still be clearly closer. In a dark frame the linear error belongs to the few bright pixels:
/// in an ISO 3200 church interior (EOS Rebel T5) a lamp covers two of 640 pairs, and the one that is held out has
/// 63% of the look's linear error (4% of the fallback's). The camera clips the lamp to white inside a proxy pixel
/// that is dark otherwise, which no curve applied to the pixel's mean reproduces, and the dark default curve happens
/// to land on it. Every look then left the linear error where it was (0.96 of the fallback's; 0.31 without those two
/// pixels) while cutting the gamma-encoded one to 0.54, and the photo kept the grey fallback.
pub(super) const PARTIAL_LINEAR_LIMIT: f64 = 1.0;

/// What makes a camera JPEG pixel count as coloured in `collect_pairs`, which needs one in twenty.
#[derive(Clone, Copy, Debug)]
pub(super) enum Coloured {
    /// Its channels are further apart than this (linear).
    Spread(f32),
    /// It shows enough colour to learn a colour matrix from: [`COLOUR_SPREAD`], or [`DARK_COLOUR_SATURATION`].
    Learnable,
}

impl Coloured {
    pub(super) fn is(self, pixel: [f32; 3]) -> bool {
        let min = pixel.iter().copied().fold(f32::INFINITY, f32::min);
        let max = pixel.iter().copied().fold(0.0, f32::max);
        match self {
            Coloured::Spread(spread) => max - min > spread,
            Coloured::Learnable => max - min > COLOUR_SPREAD || max - min > DARK_COLOUR_SATURATION * max,
        }
    }

    /// [`Coloured::is`] for a training pair's JPEG side.
    pub(super) fn is_f64(self, pixel: &[f64; 3]) -> bool {
        self.is(pixel.map(|v| v as f32))
    }
}

/// Upstream's call sites pass a spread.
impl From<f32> for Coloured {
    fn from(spread: f32) -> Self {
        Coloured::Spread(spread)
    }
}

/// Share of the held-out pairs, by the look's own linear error (at least one), that [`linear_errors`] leaves out of
/// both sides of the second pass's linear test. Since upstream fits the look on the vignetting-corrected sensor, the
/// lamp of the same ISO 3200 interior is one held-out pair of 248 and holds 90% of the partial look's linear error:
/// the look renders it a bright blue-white where the camera clipped a proxy pixel that is dark otherwise. Without
/// that pair the look keeps 0.37 of the fallback's linear error; with it, 3.0 times it, and the photo was grey.
pub(super) const PARTIAL_TRIM: f64 = 0.01;

/// Summed (fallback, look) squared linear errors of the held-out pairs, the pairs where the look is furthest from the
/// camera JPEG left out of both sums when `trim` ([`PARTIAL_TRIM`]).
pub(super) fn linear_errors(mut pairs: Vec<(f64, f64)>, trim: bool) -> (f64, f64) {
    let skip = if trim {
        pairs.sort_by(|a, b| b.1.total_cmp(&a.1));
        ((pairs.len() as f64 * PARTIAL_TRIM).ceil() as usize).min(pairs.len().saturating_sub(1))
    } else {
        0
    };
    pairs.iter().skip(skip).fold((0.0, 0.0), |(b, a), (before, after)| (b + before, a + after))
}

#[cfg(test)]
mod tests;
