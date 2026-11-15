use super::super::look_fit::fit_look;
use super::super::pairs::{EDGE_CONTRAST, MIN_IMPROVEMENT, collect_pairs, displayed, fit_pairs, fit_partial_on, luma};
use super::super::tone::{HighlightCheck, highlights};
use super::super::{CameraLook, HueSat, Mat3, Rgb32f, ToneMap, luminance_2020};
use super::{COLOUR_SPREAD, Coloured, PARTIAL_LINEAR_LIMIT, linear_errors};

/// A dim interior of saturated golds and reds (a church ceiling at ISO 3200), three stops under mid-grey, as a camera
/// with a strong colour matrix and a curve three times brighter than the scene renders it. `contrast` is the
/// brightness ratio of neighbouring pixels (fine detail everywhere when above [`EDGE_CONTRAST`]).
fn dim_interior(contrast: f32) -> (Rgb32f, Rgb32f) {
    let (w, h) = (96usize, 64usize);
    let mut sensor = Rgb32f::new(w, h);
    for (i, p) in sensor.data.iter_mut().enumerate() {
        let detail = if (i % w + i / w) % 2 == 0 { 1.0 } else { 1.0 / contrast };
        let ev = (0.0048 + (i % 23) as f32 * 0.00018) * detail;
        *p = [ev * (1.0 + (i % 11) as f32 * 0.03), ev, ev * (0.55 + (i % 13) as f32 * 0.03)];
    }
    let mut reference = sensor.clone();
    reference.map_in_place(dim_camera);
    (sensor, reference)
}

/// The camera of [`dim_interior`].
fn dim_camera(p: [f32; 3]) -> [f32; 3] {
    let q = Mat3([[2.2, -0.9, -0.3], [-0.35, 1.7, -0.35], [-0.1, -0.6, 1.7]]).apply_f32(p);
    let y = luminance_2020(q);
    if y <= 0.0 { [0.0; 3] } else { q.map(|v| (v * (1.0 - (-3.0 * y).exp()) / y).max(0.0)) }
}

/// Mean squared error (all channels) of `render` against `target` over `scene`.
fn mean_error(scene: &Rgb32f, target: impl Fn([f32; 3]) -> [f32; 3], render: impl Fn([f64; 3]) -> [f64; 3]) -> f64 {
    let error: f64 = scene
        .data
        .iter()
        .map(|x| {
            let (p, t) = (render(x.map(f64::from)), target(*x));
            (0..3).map(|c| (p[c] - f64::from(t[c])).powi(2)).sum::<f64>()
        })
        .sum();
    error / (3 * scene.data.len()) as f64
}

/// Mean colourfulness (distance from neutral of luminance-normalised RGB) of `render` over `scene`.
fn mean_colourfulness(scene: &Rgb32f, render: impl Fn([f64; 3]) -> [f64; 3]) -> f64 {
    let chroma = |p: [f64; 3]| {
        let y = luma(p);
        if y > 0.0 { p.iter().map(|v| (v / y - 1.0).powi(2)).sum::<f64>().sqrt() } else { 0.0 }
    };
    scene.data.iter().map(|x| chroma(render(x.map(f64::from)))).sum::<f64>() / scene.data.len() as f64
}

/// What `look` renders a scene colour as (matrix, hue/saturation table, tone and chroma curve).
fn rendered(look: &CameraLook, x: [f64; 3]) -> [f64; 3] {
    let p = look.matrix.apply(x);
    let p = look.hue_sat.as_ref().and_then(HueSat::new).map_or(p, |c| c.apply(p.map(|v| v as f32)).map(f64::from));
    displayed(p, &ToneMap::camera(&look.tone, 0.0, 0.0, 0.0))
}

/// A dark frame full of colour has too little channel spread to count as coloured by the spread alone (a spread is
/// brightness times saturation), so no colour was learnt from it: it took the partial look with the identity matrix
/// and rendered near-monochrome. By saturation it is coloured, and its look follows the camera's colour.
#[test]
fn a_dark_coloured_scene_learns_its_colour() {
    let (sensor, reference) = dim_interior(1.0);
    let n = reference.data.len();
    let spread = reference.data.iter().filter(|p| Coloured::Spread(COLOUR_SPREAD).is(**p)).count();
    let saturated = reference.data.iter().filter(|p| Coloured::Learnable.is(**p)).count();
    assert!(spread * 20 < n, "too dark for the spread: {spread} of {n} pixels");
    assert!(saturated * 2 > n, "but coloured: {saturated} of {n} pixels");
    assert!(collect_pairs(&sensor, &reference, COLOUR_SPREAD, None, false).is_none(), "the spread alone finds no colour");

    let look = fit_look(&sensor, &reference, &vec![false; n], None).expect("a look");
    assert_ne!(look.matrix, Mat3::IDENTITY, "the photo's own colour, not the neutral fallback's");
    let neutral = ToneMap::new(0.0, 0.0, 0.0);
    let camera = mean_colourfulness(&sensor, |x| dim_camera(x.map(|v| v as f32)).map(f64::from));
    let fitted = mean_colourfulness(&sensor, |x| rendered(&look, x)) / camera;
    let fallback = mean_colourfulness(&sensor, |x| displayed(x, &neutral)) / camera;
    assert!((0.9..1.1).contains(&fitted), "colourfulness against the camera: {fitted:.3}");
    assert!(fallback < 0.7, "the neutral fallback keeps {fallback:.3} of it");
    // pixel for pixel, far closer to the camera than the neutral fallback
    let error = mean_error(&sensor, dim_camera, |x| rendered(&look, x));
    assert!(error < 0.05 * mean_error(&sensor, dim_camera, |x| displayed(x, &neutral)), "{error:e}");

    // a dull scene stays dull whatever its brightness: no colour is learnt from grey
    let mut grey = sensor.clone();
    grey.map_in_place(|p| {
        let y = luminance_2020(p);
        [y * 1.04, y, y * 0.97]
    });
    let mut reference = grey.clone();
    reference.map_in_place(dim_camera);
    assert!(collect_pairs(&grey, &reference, Coloured::Learnable, None, false).is_none(), "grey is not coloured for being dark");
}

/// [`dim_interior`] with detail everywhere (so nothing is left away from edges) and `lamps` pixels of a lamp: the
/// camera clips it to white inside proxy pixels that are dark otherwise, which no curve applied to a pixel's mean
/// follows, while the dark default curve happens to land on it.
fn dim_interior_with_lamps(lamps: usize) -> (Rgb32f, Rgb32f) {
    let (mut sensor, mut reference) = dim_interior(4.0);
    let lamp = [0.1, 0.09, 0.15];
    let camera = displayed(lamp.map(f64::from), &ToneMap::new(0.0, 0.0, 0.0)).map(|v| v as f32);
    let n = sensor.data.len();
    for k in 0..lamps {
        let i = (200 + k * 397) % n;
        if let (Some(s), Some(r)) = (sensor.data.get_mut(i), reference.data.get_mut(i)) {
            (*s, *r) = (lamp, camera);
        }
    }
    (sensor, reference)
}

/// A few lamp pixels hold the linear error of a dark frame, so no look cut it by the usual 30% and the photo kept the
/// grey neutral fallback. The partial look is clearly closer where it is seen (gamma-encoded) and, without the
/// held-out pairs it misses most, closer in linear values too: it is used.
#[test]
fn partial_look_when_a_lamp_holds_the_linear_error() {
    let (sensor, reference) = dim_interior_with_lamps(7);
    let none = vec![false; sensor.data.len()];
    let check = HighlightCheck { pairs: highlights(&sensor, &reference), dro: false, grey: false };
    assert!(fit_pairs(&sensor, &reference).is_none(), "the full look misses the gates");
    assert!(fit_partial_on(&sensor, &reference, None, &check, MIN_IMPROVEMENT).is_none(), "and so does the partial look in linear values");
    assert!(fit_partial_on(&sensor, &reference, Some(EDGE_CONTRAST), &check, MIN_IMPROVEMENT).is_none(), "nothing is left away from edges");
    assert!(fit_partial_on(&sensor, &reference, None, &check, PARTIAL_LINEAR_LIMIT).is_some(), "the fork's second pass");
    let look = fit_look(&sensor, &reference, &none, None).expect("a partial look instead of the neutral fallback");
    assert!(look.hue_sat.is_none(), "the partial look");
    // away from the lamp the picture is far closer to the camera's, in brightness and in colour
    let (scene, _) = dim_interior(4.0);
    let neutral = ToneMap::new(0.0, 0.0, 0.0);
    let encoded = |p: [f64; 3]| p.map(|v| v.max(0.0).powf(1.0 / 2.2));
    let seen = |render: &dyn Fn([f64; 3]) -> [f64; 3]| {
        mean_error(&scene, |x| encoded(dim_camera(x).map(f64::from)).map(|v| v as f32), |x| encoded(render(x)))
    };
    let (partial, fallback) = (seen(&|x| rendered(&look, x)), seen(&|x| displayed(x, &neutral)));
    assert!(partial < fallback * 0.3, "partial {partial:.6}, neutral fallback {fallback:.6}");
    let camera = mean_colourfulness(&scene, |x| dim_camera(x.map(|v| v as f32)).map(f64::from));
    let colour = (mean_colourfulness(&scene, |x| rendered(&look, x)) / camera, mean_colourfulness(&scene, |x| displayed(x, &neutral)) / camera);
    assert!(colour.0 > 0.8 && colour.1 < 0.7, "colourfulness against the camera: partial {:.3}, neutral fallback {:.3}", colour.0, colour.1);

    // a look that misses more than a few pixels is not rescued by leaving out its worst 1%: with a lamp over 1.3% of
    // the frame the photo keeps the fallback
    let (sensor, reference) = dim_interior_with_lamps(80);
    assert!(fit_look(&sensor, &reference, &none, None).is_none(), "more linear error than the neutral fallback's");
}

/// The relaxed pass leaves out the worst 1% (at least one pair, never all of them) by the look's own error, on both
/// sides; the first pass leaves out nothing.
#[test]
fn linear_errors_leave_out_the_worst_pairs_only_when_trimmed() {
    let pairs: Vec<(f64, f64)> = (0..250).map(|i| (1.0, if i == 7 { 500.0 } else { 0.5 })).collect();
    assert_eq!(linear_errors(pairs.clone(), false), (250.0, 249.0 * 0.5 + 500.0));
    // ceil(2.5) = 3 pairs: the outlier and two ordinary ones
    assert_eq!(linear_errors(pairs, true), (247.0, 247.0 * 0.5));
    assert_eq!(linear_errors(vec![(2.0, 3.0)], true), (2.0, 3.0), "a single pair is kept");
    assert_eq!(linear_errors(Vec::new(), true), (0.0, 0.0));
}
