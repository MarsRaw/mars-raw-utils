//! Stereo range from a left/right pair of CAHVORE-modeled images.
//!
//! Pipeline:
//!   1. Build a *rectified* pair of linear (CAHV) models: both share the same
//!      A, H, V (so epipolar lines are image rows) and differ only in camera
//!      center, offset by the baseline along the image x axis.
//!   2. Resample both images onto those models (`resample_to_model`).
//!   3. Block-match along rows to get disparity (with sub-pixel refinement and
//!      a left-right consistency check).
//!   4. Convert disparity to meters:  depth Z = f * B / d  along the boresight,
//!      or Euclidean distance from the left camera center.
//!
//! NOTE: written against the files provided, not compiled. Adjust `use` paths
//! to your crate layout.

use anyhow::{anyhow, Result};
use sciimg::camera::cahvore::Cahvore;
use sciimg::enums::ImageMode;
use sciimg::image::Image;

use crate::linearize::{model_for_image, resample_to_model};
use crate::marsimage::MarsImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeKind {
    /// Straight-line distance from the left camera center (what most people
    /// mean by "range").
    Distance,
    /// Distance along the rectified boresight (the stereo "Z" / depth).
    Depth,
}

#[derive(Debug, Clone)]
pub struct StereoParams {
    /// Nearest and farthest range to search, in meters. These set the
    /// disparity search window: d = f * B / Z. A small `min_range_m` costs
    /// time (search width grows as 1/Z).
    pub min_range_m: f64,
    pub max_range_m: f64,
    /// Match window half-size in pixels (4 => 9x9).
    pub window_radius: usize,
    /// Half-size of the box used to remove local mean brightness before
    /// matching. Makes the match tolerant of exposure differences. 0 disables.
    pub prefilter_radius: usize,
    /// Reject matches whose mean absolute difference (intensities normalized to
    /// 0..1, mean-removed) is above this. Tune per instrument.
    pub max_cost: f32,
    /// Max |disparity_left - disparity_right| in pixels for the left-right
    /// check. Negative disables the check.
    pub lr_tolerance: i32,
    pub range_kind: RangeKind,
}

impl Default for StereoParams {
    fn default() -> Self {
        StereoParams {
            min_range_m: 2.0,
            max_range_m: 150.0,
            window_radius: 4,
            prefilter_radius: 7,
            max_cost: 0.04,
            lr_tolerance: 1,
            range_kind: RangeKind::Distance,
        }
    }
}

/// Rectified model pair.
#[derive(Debug, Clone)]
pub struct RectifiedPair {
    pub left: Cahvore,
    pub right: Cahvore,
    /// Focal length in pixels (same for both, square pixels).
    pub focal_px: f64,
    /// Baseline length in the units of the camera models (meters).
    pub baseline_m: f64,
    /// Principal point, shared by both rectified images.
    pub hc: f64,
    pub vc: f64,
}

/// Result of `compute_stereo_range`. All arrays are row-major in the rectified
/// left image geometry (`model`), `width` x `height`. Invalid pixels are NaN.
#[derive(Debug, Clone)]
pub struct StereoRange {
    pub width: usize,
    pub height: usize,
    pub range_m: Vec<f32>,
    pub disparity_px: Vec<f32>,
    /// Rectified left camera model (rover frame). Use it with `range_m` to
    /// turn pixels into XYZ: P = C + d * look_vector(u, v).
    pub model: Cahvore,
    pub focal_px: f64,
    pub baseline_m: f64,
}

impl StereoRange {
    /// Wrap the range as a single-band MarsImage, DN = range_m * dn_per_meter
    /// (clamped to 16 bits; e.g. 100.0 => centimeters, max ~655 m). Invalid
    /// pixels are DN 0 with alpha off. Metadata is copied from `template`
    /// (normally the left image) with the rectified model swapped in.
    pub fn to_mars_image(&self, template: &MarsImage, dn_per_meter: f32) -> Result<MarsImage> {
        let mut img =
            Image::new_with_bands_masked(self.width, self.height, 1, ImageMode::U16BIT, false)?;
        for y in 0..self.height {
            for x in 0..self.width {
                let r = self.range_m[y * self.width + x];
                if r.is_finite() {
                    img.put(x, y, (r * dn_per_meter).clamp(0.0, 65535.0), 0);
                    img.put_alpha(x, y, true);
                }
            }
        }

        let mut out = MarsImage::from_image(&img, template.instrument);

        let mut out_rover_metadata = if let Some(rover_metadata) = template.metadata.as_rover() {
            rover_metadata.clone()
        } else {
            return Err(anyhow!(
                "Non surface metadata constructed when a surface (rover) metadata was expected"
            ));
        };

        out.metadata = template.metadata.clone();

        // The new model describes the *output* pixel grid, so the subframe offset
        // and scale have already been folded in.
        out_rover_metadata.camera_model_component_list = self.model.clone();
        out_rover_metadata.subframe_rect = None;
        // Adjust to whatever your pipeline expects for a distortion-free model:
        out_rover_metadata.camera_model_type = Some("CAHV".to_string());
        out.update_history();
        out.metadata.from_rover(&out_rover_metadata);

        Ok(out)
    }
}

/// Pixel scale (focal length in pixels) of a linear model.
fn pixel_scale(m: &Cahvore) -> f64 {
    let a = m.a.normalized();
    (m.h - a * m.h.dot_product(&a)).norm()
}

/// Build the rectified model pair for two cameras in the same frame.
///
///   x' = (C_R - C_L) / |C_R - C_L|         image "right"; epipolar direction
///   A' = mean boresight made orthogonal to x'
///   y' = A' x x'                           image "down"
///
/// Both models use A', H = hc*A' + f*x', V = vc*A' + f*y', so a point at depth
/// Z lands on the same row in both and has disparity f*B/Z.
pub fn rectify_models(
    left: &Cahvore,
    right: &Cahvore,
    width: usize,
    height: usize,
) -> Result<RectifiedPair> {
    let ll = left.linearize(width, height, width, height)?;
    let lr = right.linearize(width, height, width, height)?;
    let focal_px = 0.5 * (pixel_scale(&ll) + pixel_scale(&lr));

    let baseline = right.c - left.c;
    let baseline_m = baseline.norm();
    if baseline_m < 1e-6 {
        return Err(anyhow!("rectify_models: left and right centers coincide"));
    }

    // Sanity check: the right camera must be on the +x (image right) side of
    // the left camera, otherwise the images are swapped.
    let a_l = ll.a.normalized();
    let right_l = (ll.h - a_l * ll.h.dot_product(&a_l)).normalized();
    if baseline.dot_product(&right_l) <= 0.0 {
        return Err(anyhow!(
            "rectify_models: right camera is not to the right of the left; images swapped?"
        ));
    }

    let x = baseline / baseline_m;
    let a_mean = (ll.a.normalized() + lr.a.normalized()).normalized();
    let a = (a_mean - x * a_mean.dot_product(&x)).normalized();
    let down = a.cross_product(&x).normalized();

    let hc = (width as f64 - 1.0) / 2.0;
    let vc = (height as f64 - 1.0) / 2.0;

    let make = |c| Cahvore::new_cahv(c, a, a * hc + x * focal_px, a * vc + down * focal_px);

    Ok(RectifiedPair {
        left: make(left.c),
        right: make(right.c),
        focal_px,
        baseline_m,
        hc,
        vc,
    })
}

/// Compute a range image (meters) from a left/right pair. Both images must be
/// the same size and carry camera models in the same frame; both are brought
/// to the pixel grid of their loaded image by `model_for_image`.
pub fn compute_stereo_range(
    left: &MarsImage,
    right: &MarsImage,
    params: &StereoParams,
) -> Result<StereoRange> {
    let (w, h) = (left.image.width, left.image.height);
    if (right.image.width, right.image.height) != (w, h) {
        return Err(anyhow!("compute_stereo_range: image sizes differ"));
    }
    if !(params.min_range_m > 0.0 && params.max_range_m > params.min_range_m) {
        return Err(anyhow!("compute_stereo_range: bad range limits"));
    }

    let model_l = if let Some(model) = model_for_image(left) {
        model
    } else {
        return Err(anyhow!("Camera model not available for image"));
    };
    let model_r = if let Some(model) = model_for_image(right) {
        model
    } else {
        return Err(anyhow!("Camera model not available for image"));
    };
    let pair = rectify_models(&model_l, &model_r, w, h)?;

    // Rectify both images onto the shared geometry.
    let rect_l = resample_to_model(left, &model_l, pair.left.clone(), w, h)?;
    let rect_r = resample_to_model(right, &model_r, pair.right.clone(), w, h)?;

    let (mut gl, valid_l) = luma(&rect_l);
    let (mut gr, valid_r) = luma(&rect_r);

    // Common normalization to 0..1, then local-mean removal.
    let max_v = gl
        .iter()
        .chain(gr.iter())
        .cloned()
        .fold(0.0_f32, f32::max)
        .max(1e-6);
    gl.iter_mut().for_each(|v| *v /= max_v);
    gr.iter_mut().for_each(|v| *v /= max_v);
    if params.prefilter_radius > 0 {
        remove_local_mean(&mut gl, w, h, params.prefilter_radius);
        remove_local_mean(&mut gr, w, h, params.prefilter_radius);
    }

    // Disparity search window from the range limits: d = f * B / Z.
    let fb = pair.focal_px * pair.baseline_m;
    let d_lo = ((fb / params.max_range_m).floor() as usize).max(1);
    let d_hi = ((fb / params.min_range_m).ceil() as usize).min(w - 1);
    if d_hi <= d_lo {
        return Err(anyhow!(
            "compute_stereo_range: empty disparity window ({}..{})",
            d_lo,
            d_hi
        ));
    }

    let disparity = match_blocks(&gl, &gr, &valid_l, &valid_r, w, h, d_lo, d_hi, params);

    // Disparity -> meters
    let mut range_m = vec![f32::NAN; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let d = disparity[i] as f64;
            if !d.is_finite() || d <= 0.0 {
                continue;
            }
            let z = fb / d;
            if z < params.min_range_m || z > params.max_range_m {
                continue;
            }
            range_m[i] = match params.range_kind {
                RangeKind::Depth => z as f32,
                RangeKind::Distance => {
                    let xn = (x as f64 - pair.hc) / pair.focal_px;
                    let yn = (y as f64 - pair.vc) / pair.focal_px;
                    (z * (1.0 + xn * xn + yn * yn).sqrt()) as f32
                }
            };
        }
    }

    Ok(StereoRange {
        width: w,
        height: h,
        range_m,
        disparity_px: disparity,
        model: pair.left,
        focal_px: pair.focal_px,
        baseline_m: pair.baseline_m,
    })
}

/// Mean of all bands and a validity mask from the alpha channel.
fn luma(img: &MarsImage) -> (Vec<f32>, Vec<bool>) {
    let (w, h) = (img.image.width, img.image.height);
    let nb = img.image.num_bands();
    let mut g = vec![0.0_f32; w * h];
    let mut valid = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0;
            for b in 0..nb {
                sum += img.image.get_band(b).get(x, y);
            }
            g[y * w + x] = sum / nb as f32;
            valid[y * w + x] = img.get_alpha_at(x, y);
        }
    }
    (g, valid)
}

/// (w+1) x (h+1) summed-area table.
fn integral(data: &[f32], w: usize, h: usize, out: &mut Vec<f64>) {
    let stride = w + 1;
    out.clear();
    out.resize(stride * (h + 1), 0.0);
    for y in 0..h {
        let mut row = 0.0_f64;
        for x in 0..w {
            row += data[y * w + x] as f64;
            out[(y + 1) * stride + (x + 1)] = out[y * stride + (x + 1)] + row;
        }
    }
}

/// Mean over the (2r+1)^2 window at (x, y), clamped to the image.
#[inline]
fn box_mean(ii: &[f64], w: usize, h: usize, x: usize, y: usize, r: usize) -> f64 {
    let stride = w + 1;
    let x0 = x.saturating_sub(r);
    let x1 = (x + r + 1).min(w);
    let y0 = y.saturating_sub(r);
    let y1 = (y + r + 1).min(h);
    let sum =
        ii[y1 * stride + x1] - ii[y0 * stride + x1] - ii[y1 * stride + x0] + ii[y0 * stride + x0];
    sum / ((x1 - x0) * (y1 - y0)) as f64
}

fn remove_local_mean(g: &mut [f32], w: usize, h: usize, r: usize) {
    let mut ii = Vec::new();
    integral(g, w, h, &mut ii);
    for y in 0..h {
        for x in 0..w {
            g[y * w + x] -= box_mean(&ii, w, h, x, y, r) as f32;
        }
    }
}

/// SAD block matching along rows on a rectified pair. Returns left-referenced
/// disparity (pixels, NaN where invalid).
///
/// For each candidate disparity d the per-pixel |L(x) - R(x - d)| is boxed via
/// a summed-area table, so cost is O(W * H) per disparity regardless of window
/// size. Winner-take-all is tracked for the left image and, from the same
/// costs, for the right image (for the left-right check), without storing the
/// full cost volume. The costs at best_d -/+ 1 are kept for parabola sub-pixel
/// refinement.
#[allow(clippy::too_many_arguments)]
fn match_blocks(
    gl: &[f32],
    gr: &[f32],
    valid_l: &[bool],
    valid_r: &[bool],
    w: usize,
    h: usize,
    d_lo: usize,
    d_hi: usize,
    params: &StereoParams,
) -> Vec<f32> {
    let n = w * h;
    let r = params.window_radius;
    // Cost charged for pixels that have no counterpart (outside the frame or
    // masked). Keeps windows over invalid areas from winning.
    let penalty = 0.25_f32;

    let mut diff = vec![0.0_f32; n];
    let mut ii: Vec<f64> = Vec::new();
    let mut c_cur = vec![0.0_f32; n];
    let mut c_prev = vec![f32::INFINITY; n];

    let mut best_c = vec![f32::INFINITY; n];
    let mut best_d = vec![-1_i32; n];
    let mut c_minus = vec![f32::NAN; n];
    let mut c_plus = vec![f32::NAN; n];

    let mut best_rc = vec![f32::INFINITY; n];
    let mut best_rd = vec![-1_i32; n];

    for d in d_lo..=d_hi {
        let di = d as i32;

        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                diff[i] = if x >= d && valid_l[i] && valid_r[i - d] {
                    (gl[i] - gr[i - d]).abs()
                } else {
                    penalty
                };
            }
        }
        integral(&diff, w, h, &mut ii);

        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let c = box_mean(&ii, w, h, x, y, r) as f32;
                c_cur[i] = c;

                // Left-referenced winner (and the +1 neighbor cost).
                if best_d[i] == di - 1 {
                    c_plus[i] = c;
                }
                if c < best_c[i] {
                    best_c[i] = c;
                    best_d[i] = di;
                    c_minus[i] = if d > d_lo { c_prev[i] } else { f32::NAN };
                    c_plus[i] = f32::NAN;
                }

                // Right-referenced winner: left (x, y) pairs with right (x-d, y).
                if x >= d {
                    let ir = i - d;
                    if c < best_rc[ir] {
                        best_rc[ir] = c;
                        best_rd[ir] = di;
                    }
                }
            }
        }

        std::mem::swap(&mut c_prev, &mut c_cur);
    }

    let mut disparity = vec![f32::NAN; n];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let d = best_d[i];
            if d < 0 || !valid_l[i] || best_c[i] > params.max_cost {
                continue;
            }
            let du = d as usize;
            if x < du {
                continue;
            }
            if params.lr_tolerance >= 0 && (best_rd[i - du] - d).abs() > params.lr_tolerance {
                continue;
            }

            // Parabola through (d-1, d, d+1) for sub-pixel disparity.
            let mut ds = d as f32;
            let (cm, c0, cp) = (c_minus[i], best_c[i], c_plus[i]);
            if cm.is_finite() && cp.is_finite() {
                let denom = cm - 2.0 * c0 + cp;
                if denom > 1e-9 {
                    ds += (0.5 * (cm - cp) / denom).clamp(-0.5, 0.5);
                }
            }
            disparity[i] = ds;
        }
    }

    disparity
}
