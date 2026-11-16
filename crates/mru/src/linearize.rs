use anyhow::Result;
use anyhow::anyhow;
use sciimg::vector::Vector;
use sciimg::{prelude::*, quaternion::Quaternion};

use crate::marsimage::MarsImage;
use crate::stereo::build_map;
use crate::stereo::model_for_image;

/// What to do with the boresight when leveling.
#[derive(Debug, Clone, Copy)]
pub enum Boresight {
    /// Keep the true pointing (azimuth and elevation) of the view. Only the
    /// roll about the boresight is removed, so the horizon is level and sits
    /// where it really is in the frame.
    Preserve,
    /// Keep the azimuth but force the boresight elevation (radians, positive
    /// above the horizon). 0.0 puts the horizon through the image center.
    /// Re-aiming far from the original view leaves empty (masked) edges.
    Elevation(f64),
}

/// Options for removing rover roll/pitch from the output.
#[derive(Debug, Clone)]
pub struct LevelOptions {
    pub boresight: Boresight,
    /// Rover-frame -> local-level (z down, e.g. site NED) attitude. `None`
    /// uses `metadata.attitude`.
    pub attitude: Option<Quaternion>,
    /// Flip this if the horizon comes out tilted the wrong way: it means the
    /// attitude is level -> rover rather than rover -> level.
    pub invert_attitude: bool,
}

impl Default for LevelOptions {
    fn default() -> Self {
        LevelOptions {
            boresight: Boresight::Preserve,
            attitude: None,
            invert_attitude: false,
        }
    }
}

/// Output canvas sizing.
#[derive(Debug, Clone, Copy)]
pub enum Extent {
    /// Fixed canvas. The FOV is cropped to the narrowest source edge angle, so
    /// there are no empty corners but edge pixels are lost.
    Crop { width: usize, height: usize },
    /// Grow the canvas until every source pixel that a perspective camera can
    /// represent is included. Pixel scale is the same as `Crop` at the source
    /// size, and is reduced uniformly if needed so that the canvas never exceeds
    /// `max_scale` x the source width and `max_scale` x the source height.
    Expand { max_scale: f64 },
}

/// Linearize with the canvas expanded to include all source pixels (up to
/// `max_scale` x the source size per dimension; 2.0 for double), optionally
/// leveled.
pub fn linearize_image_expanded(
    src: &MarsImage,
    level: Option<&LevelOptions>,
    extent: &Extent,
    apply_scaling: bool,
) -> Result<MarsImage> {
    if let Some(model) = model_for_image(src, apply_scaling) {
        linearize_image_with_model(src, &model, *extent, level)
    } else {
        Err(anyhow!(
            "Linearization with CAHVORE camera model not supported for nonsurface missions"
        ))
    }
}

/// Re-orient a linear (CAHV) model so its image "right" axis is horizontal.
///
/// The model lives in the rover frame, so the only thing needed from the
/// attitude is gravity ("down") expressed in that frame. The output basis is
/// built from it:
///     right = down x A      (horizontal)
///     down' = A x right
/// which is zero roll about A by construction. Focal scale and principal point
/// are carried over from `lin`; the camera center is unchanged. The result is
/// still expressed in the rover frame, so `metadata.attitude` stays valid.
pub fn level_model(
    lin: &Cahvore,
    attitude: &Quaternion,
    invert_attitude: bool,
    boresight: Boresight,
) -> Result<Cahvore> {
    let q = if invert_attitude {
        attitude.normalized().invert()
    } else {
        attitude.normalized()
    };
    let level_to_rover = q.invert();

    // Gravity in the rover frame (level frame has z pointing down).
    let down_r = level_to_rover
        .rotate_vector(&Vector::new(0.0, 0.0, 1.0))
        .normalized();

    // Pull scale/center back out of the linear model: H = hc*A + hs*right
    let a_old = lin.a.normalized();
    let hc = lin.h.dot_product(&a_old);
    let vc = lin.v.dot_product(&a_old);
    let hs = (lin.h - a_old * hc).norm();
    let vs = (lin.v - a_old * vc).norm();

    let a = match boresight {
        Boresight::Preserve => a_old,
        Boresight::Elevation(el) => {
            // Work in the level frame (x north, y east, z down).
            let a_lv = q.rotate_vector(&a_old);
            let az = a_lv.y.atan2(a_lv.x);
            let a_new = Vector::new(el.cos() * az.cos(), el.cos() * az.sin(), -el.sin());
            level_to_rover.rotate_vector(&a_new).normalized()
        }
    };

    let right = down_r.cross_product(&a);
    if right.norm() < 1e-6 {
        return Err(anyhow!(
            "level_model: boresight is (nearly) vertical, roll is undefined"
        ));
    }
    let right = right.normalized();
    let down = a.cross_product(&right).normalized();

    Ok(Cahvore::new_cahv(
        lin.c,
        a,
        a * hc + right * hs,
        a * vc + down * vs,
    ))
}

/// Resize and re-center a linear (CAHV) model so its canvas holds every source
/// pixel that can be shown, returning the new model and canvas size.
///
/// Every source boundary pixel is cast through `src_model` and projected onto
/// `base`'s image plane (tangent coordinates x = d.right/d.A, y = d.down/d.A).
/// The mapping is continuous, so the bounding box of the border is the bounding
/// box of the whole frame. Rays 85 degrees or more from the boresight can't be
/// represented by a perspective camera and are left out.
///
/// The scale (hs, vs) from `base` is kept unless the canvas would exceed
/// `max_scale` x the source size, in which case both are reduced by the same
/// factor so that the pixels stay square.
pub fn expand_model(
    src_model: &Cahvore,
    base: &Cahvore,
    src_w: usize,
    src_h: usize,
    max_scale: f64,
) -> Result<(Cahvore, usize, usize)> {
    if max_scale < 1.0 {
        return Err(anyhow!("expand_model: max_scale must be >= 1.0"));
    }

    let a = base.a.normalized();
    let hc0 = base.h.dot_product(&a);
    let vc0 = base.v.dot_product(&a);
    let h_perp = base.h - a * hc0;
    let v_perp = base.v - a * vc0;
    let hs = h_perp.norm();
    let vs = v_perp.norm();
    let right = h_perp.normalized();
    let down = v_perp.normalized();

    let min_cos = (85.0_f64).to_radians().cos();

    // Border of the source frame, one sample per pixel.
    let mut border: Vec<(f64, f64)> = Vec::with_capacity(2 * (src_w + src_h));
    for x in 0..src_w {
        border.push((x as f64, 0.0));
        border.push((x as f64, (src_h - 1) as f64));
    }
    for y in 0..src_h {
        border.push((0.0, y as f64));
        border.push(((src_w - 1) as f64, y as f64));
    }

    let (mut x_min, mut x_max) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut y_min, mut y_max) = (f64::INFINITY, f64::NEG_INFINITY);

    for (sample, line) in border {
        let Ok(lv) = src_model.ls_to_look_vector(&ImageCoordinate { line, sample }) else {
            continue;
        };
        let d = lv.look_direction.normalized();
        let along = d.dot_product(&a);
        if along <= min_cos {
            continue;
        }
        let x = d.dot_product(&right) / along;
        let y = d.dot_product(&down) / along;
        x_min = x_min.min(x);
        x_max = x_max.max(x);
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    }

    if !(x_min.is_finite() && y_min.is_finite()) {
        return Err(anyhow!(
            "expand_model: no source pixels project onto the output plane"
        ));
    }

    // Canvas size in pixels at the base scale (pixel centers on integers, so a
    // span of N pixels needs N + 1 columns).
    let span_w = (x_max - x_min) * hs;
    let span_h = (y_max - y_min) * vs;

    let max_w = (src_w as f64 * max_scale).floor();
    let max_h = (src_h as f64 * max_scale).floor();

    let k = 1.0_f64
        .min((max_w - 1.0) / span_w.max(1e-9))
        .min((max_h - 1.0) / span_h.max(1e-9));

    let hs = hs * k;
    let vs = vs * k;

    let out_w = ((span_w * k).ceil() as usize + 1).min(max_w as usize);
    let out_h = ((span_h * k).ceil() as usize + 1).min(max_h as usize);

    // Put the left/top extreme at pixel 0.
    let hc = -x_min * hs;
    let vc = -y_min * vs;

    let model = Cahvore::new_cahv(base.c, a, a * hc + right * hs, a * vc + down * vs);
    Ok((model, out_w, out_h))
}

/// Linearize with an explicit source model and canvas extent, optionally leveled.
///
/// `src_model` must be in the pixel coordinates of `src.image`.
pub fn linearize_image_with_model(
    src: &MarsImage,
    src_model: &Cahvore,
    extent: Extent,
    level: Option<&LevelOptions>,
) -> Result<MarsImage> {
    if src.is_empty() || src.image.num_bands() == 0 {
        return Err(anyhow!("linearize_image: source image is empty"));
    }

    let rover_metadata = if let Some(rover_metadata) = src.metadata.as_surface() {
        rover_metadata
    } else {
        return Err(anyhow!(
            "Linearization with CAHVORE camera model not supported for nonsurface missions"
        ));
    };

    let src_w = src.image.width;
    let src_h = src.image.height;
    let num_bands = src.image.num_bands();

    // Ideal pinhole camera: same center, boresight through the middle of the
    // source frame, FOV chosen from the source edge rays.
    let (mut out_model, mut out_width, mut out_height) = match extent {
        Extent::Crop { width, height } => (
            src_model.linearize(src_w, src_h, width, height)?,
            width,
            height,
        ),
        // Start from the source-size model; the canvas is resized below.
        Extent::Expand { .. } => (
            src_model.linearize(src_w, src_h, src_w, src_h)?,
            src_w,
            src_h,
        ),
    };

    if let Some(opts) = level {
        let attitude = opts
            .attitude
            .or(rover_metadata.attitude)
            .ok_or_else(|| anyhow!("linearize_image: leveling requested but no attitude"))?;
        out_model = level_model(&out_model, &attitude, opts.invert_attitude, opts.boresight)?;
    }

    // Expansion goes last so it sees the final (leveled) orientation.
    if let Extent::Expand { max_scale } = extent {
        let (m, w, h) = expand_model(src_model, &out_model, src_w, src_h, max_scale)?;
        out_model = m;
        out_width = w;
        out_height = h;
    }

    let map = build_map(src_model, &out_model, src_w, src_h, out_width, out_height);

    // Alpha starts fully masked (false); only pixels that map into the source
    // frame are switched on below. Pixels that never get written stay 0.
    let mut out_img = Image::new_with_bands_masked(
        out_width,
        out_height,
        num_bands,
        src.image.get_mode(),
        false,
    )?;

    for y in 0..out_height {
        for x in 0..out_width {
            let Some((s, l)) = map[y * out_width + x] else {
                continue;
            };

            // Respect the source alpha at the nearest source pixel.
            let nx = (s.round() as usize).min(src_w - 1);
            let ny = (l.round() as usize).min(src_h - 1);
            if !src.get_alpha_at(nx, ny) {
                continue;
            }

            for band in 0..num_bands {
                let v = src.image.get_band(band).get_interpolated(s, l)?;
                out_img.put(x, y, v, band);
            }
            out_img.put_alpha(x, y, true);
        }
    }

    let mut out = MarsImage::from_image(&out_img, src.instrument);

    let mut out_rover_metadata = if let Some(rover_metadata) = src.metadata.as_surface() {
        rover_metadata.clone()
    } else {
        return Err(anyhow!(
            "Non surface metadata constructed when a surface (rover) metadata was expected"
        ));
    };

    out.metadata = src.metadata.clone();
    out.set_linearized();

    // The new model describes the *output* pixel grid, so the subframe offset
    // and scale have already been folded in.
    out_rover_metadata.camera_model_component_list = out_model;
    out_rover_metadata.subframe_rect = None;
    // Adjust to whatever your pipeline expects for a distortion-free model:
    out_rover_metadata.camera_model_type = Some("CAHV".to_string());
    out.update_history();
    out.metadata.from_surface(&out_rover_metadata);

    Ok(out)
}
