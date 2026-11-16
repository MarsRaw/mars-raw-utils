use anyhow::{Result, anyhow};
use sciimg::{
    camera::{cahvore::Cahvore, model::ImageCoordinate},
    image::Image,
};

use crate::marsimage::MarsImage;

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

/// For each output pixel (row-major), the (sample, line) in the source image,
/// or None if the ray can't be projected or lands outside the source frame.
///
/// This depends only on the two camera models and the image sizes, so it can be
/// built once and reused for every frame taken with the same model.
pub fn build_map(
    src_model: &Cahvore,
    out_model: &Cahvore,
    src_w: usize,
    src_h: usize,
    out_w: usize,
    out_h: usize,
) -> Vec<Option<(f32, f32)>> {
    let max_s = (src_w - 1) as f64;
    let max_l = (src_h - 1) as f64;

    let mut map = Vec::with_capacity(out_w * out_h);

    for y in 0..out_h {
        for x in 0..out_w {
            let entry = out_model
                .ls_to_look_vector(&ImageCoordinate {
                    line: y as f64,
                    sample: x as f64,
                })
                .ok()
                .and_then(|lv| {
                    // Output camera shares C with the source, so the ray
                    // direction is all that's needed. `infinity = true` skips
                    // the entrance-pupil (E) shift.
                    src_model.xyz_to_ls(&lv.look_direction, true).ok()
                })
                .filter(|c| {
                    c.sample.is_finite()
                        && c.line.is_finite()
                        && c.sample >= 0.0
                        && c.sample <= max_s
                        && c.line >= 0.0
                        && c.line <= max_l
                })
                .map(|c| (c.sample as f32, c.line as f32));

            map.push(entry);
        }
    }

    map
}

/// Resample `src` (described by `src_model`) onto an arbitrary linear output
/// model and canvas. This is the shared back end for linearizing, leveling,
/// expanding and stereo rectification.
pub fn resample_to_model(
    src: &MarsImage,
    out_model: Cahvore,
    out_width: usize,
    out_height: usize,
) -> Result<MarsImage> {
    if src.is_empty() || src.image.num_bands() == 0 {
        return Err(anyhow!("resample_to_model: source image is empty"));
    }

    let src_w = src.image.width;
    let src_h = src.image.height;
    let num_bands = src.image.num_bands();

    let src_model = match model_for_image(src, true) {
        Some(model) => model,
        None => return Err(anyhow!("No camera model for image. Cannot resample.")),
    };

    let map = build_map(&src_model, &out_model, src_w, src_h, out_width, out_height);

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

/// The source camera model, adjusted to the pixel grid of the image as loaded.
///
/// Assumes the model in the metadata describes the full, unscaled frame, and
/// that `subframe_rect` is [x, y, width, height] with 1-based x/y in full-frame
/// pixels (the same convention `MarsImage::flatfield` uses to crop the flat).
/// If your metadata models are already adjusted, skip this and call
/// `linearize_image_with_model` directly.
pub fn model_for_image(img: &MarsImage, apply_scaling: bool) -> Option<Cahvore> {
    let rover_metadata = img.metadata.as_surface()?;

    let mut model = rover_metadata.camera_model_component_list.clone();

    if let Some(sf) = &rover_metadata.subframe_rect
        && sf.len() >= 2
    {
        // source pixel in the loaded image = full-frame pixel - (origin - 1)
        model = model.shift(-(sf[0] - 1.0), -(sf[1] - 1.0));
    }

    // This tends to lead to weird results in M20 NavCam images, so I'm going
    // to hide it behind a user option.
    if apply_scaling && rover_metadata.scale_factor > 1 {
        let s = 1.0 / rover_metadata.scale_factor as f64;
        model = model.scale(s, s);
    }

    Some(model)
}

pub fn remove_local_mean(g: &mut [f32], w: usize, h: usize, r: usize) {
    let integral = Integral::new(g, w, h);
    for y in 0..h {
        for x in 0..w {
            g[y * w + x] -= integral.box_mean(w, h, x, y, r) as f32;
        }
    }
}

pub struct Integral {
    w: usize,
    sum: Vec<f64>,
    sq: Vec<f64>,
}

impl Integral {
    pub fn new(data: &[f32], w: usize, h: usize) -> Self {
        let iw = w + 1;
        let mut sum = vec![0.0; iw * (h + 1)];
        let mut sq = vec![0.0; iw * (h + 1)];
        for y in 0..h {
            let (mut rs, mut rq) = (0.0, 0.0);
            for x in 0..w {
                let v = data[y * w + x] as f64;
                rs += v;
                rq += v * v;
                sum[(y + 1) * iw + x + 1] = sum[y * iw + x + 1] + rs;
                sq[(y + 1) * iw + x + 1] = sq[y * iw + x + 1] + rq;
            }
        }
        Self { w: iw, sum, sq }
    }

    // (sum, sum of squares) over the k×k window with top-left (x, y)
    pub fn window(&self, x: usize, y: usize, k: usize) -> (f64, f64) {
        let f = |t: &[f64]| {
            t[(y + k) * self.w + x + k] - t[y * self.w + x + k] - t[(y + k) * self.w + x]
                + t[y * self.w + x]
        };
        (f(&self.sum), f(&self.sq))
    }

    /// Mean over the (2r+1)^2 window at (x, y), clamped to the image.
    pub fn box_mean(&self, w: usize, h: usize, x: usize, y: usize, r: usize) -> f64 {
        let stride = w + 1;
        let x0 = x.saturating_sub(r);
        let x1 = (x + r + 1).min(w);
        let y0 = y.saturating_sub(r);
        let y1 = (y + r + 1).min(h);
        let sum =
            self.sum[y1 * stride + x1] - self.sum[y0 * stride + x1] - self.sum[y1 * stride + x0]
                + self.sum[y0 * stride + x0];
        sum / ((x1 - x0) * (y1 - y0)) as f64
    }
}
