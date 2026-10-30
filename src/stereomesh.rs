use crate::prelude::*;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use sciimg::{prelude::*, vector::Vector};

struct Triangulated {
    point: Vector,
    miss_distance: f64,
}

fn triangulate(p1: &Vector, d1: &Vector, p2: &Vector, d2: &Vector) -> Option<Triangulated> {
    let d1 = d1.normalized();
    let d2 = d2.normalized();
    let w = p1 - p2;

    let b = d1.dot_product(&d2);
    let d = d1.dot_product(&w);
    let e = d2.dot_product(&w);

    let denom = 1.0 - b * b;
    if denom < 1e-12 {
        return None; // near-parallel rays
    }

    let t = (b * e - d) / denom;
    let s = (e - b * d) / denom;
    if t < 0.0 || s < 0.0 {
        return None; // intersection behind a camera
    }

    let q1 = p1.add(&d1.scale(t));
    let q2 = p2.add(&d2.scale(s));
    Some(Triangulated {
        point: (q1 + q2) * 0.5,
        miss_distance: (q1 - q2).len(),
    })
}

pub fn get_cahvor(img: &MarsImage) -> Option<Cahvore> {
    img.metadata
        .as_rover()
        .map(|rover_metadata| rover_metadata.camera_model_component_list.clone())
}

struct Integral {
    w: usize,
    sum: Vec<f64>,
    sq: Vec<f64>,
}

impl Integral {
    fn new(data: &[f32], w: usize, h: usize) -> Self {
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
    fn window(&self, x: usize, y: usize, k: usize) -> (f64, f64) {
        let f = |t: &[f64]| {
            t[(y + k) * self.w + x + k] - t[y * self.w + x + k] - t[(y + k) * self.w + x]
                + t[y * self.w + x]
        };
        (f(&self.sum), f(&self.sq))
    }
}

#[allow(clippy::too_many_arguments)]
fn ncc(
    left: &[f32],
    lw: usize,
    lx: usize,
    ly: usize,
    lstats: (f64, f64),
    right: &[f32],
    rw: usize,
    rx: usize,
    ry: usize,
    rint: &Integral,
    k: usize,
) -> f64 {
    let n = (k * k) as f64;
    let mut dot = 0.0f64;
    for j in 0..k {
        let l = &left[(ly + j) * lw + lx..][..k];
        let r = &right[(ry + j) * rw + rx..][..k];
        for i in 0..k {
            dot += l[i] as f64 * r[i] as f64;
        }
    }
    let (ls, lq) = lstats;
    let (rs, rq) = rint.window(rx, ry, k);
    let vl = lq - ls * ls / n;
    let vr = rq - rs * rs / n;
    if vl < 1e-9 || vr < 1e-9 {
        return 0.0; // textureless window, correlation meaningless
    }
    (dot - ls * rs / n) / (vl * vr).sqrt()
}

/// Candidate window centers (sample, line) in the right image for a left pixel.
fn epipolar_candidates(
    left: &Cahvore,
    right: &Cahvore,
    x0: usize,
    y0: usize,
    near: f64, // meters, e.g. 2.0
    far: f64,  // meters, e.g. 100.0
) -> Vec<(i64, i64)> {
    let lv = match left.ls_to_look_vector(&ImageCoordinate {
        line: y0 as f64,
        sample: x0 as f64,
    }) {
        Ok(lv) => lv,
        Err(_) => return vec![],
    };

    let project = |range: f64| -> Option<(f64, f64)> {
        let p = lv.origin + lv.look_direction * range;
        right
            .xyz_to_ls(&p, false)
            .ok()
            .map(|ic| (ic.sample, ic.line))
    };

    let (s_near, l_near) = match project(near) {
        Some(p) => p,
        None => return vec![],
    };
    let (s_far, l_far) = match project(far) {
        Some(p) => p,
        None => return vec![],
    };

    // About one sample per pixel along the segment
    let len = ((s_near - s_far).powi(2) + (l_near - l_far).powi(2)).sqrt();
    let n = (len.ceil() as usize).max(2);

    let (inv_near, inv_far) = (1.0 / near, 1.0 / far);
    let mut out: Vec<(i64, i64)> = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let inv = inv_far + (inv_near - inv_far) * (i as f64 / n as f64);
        if let Some((s, l)) = project(1.0 / inv) {
            let c = (s.round() as i64, l.round() as i64);
            if out.last() != Some(&c) {
                out.push(c);
            }
        }
    }
    out
}

#[derive(Default)]
struct PointMatch {
    score: f64,
    rx: usize,
    ry: usize,
}

#[derive(Clone)]
pub struct StereoMeshOptions {
    pub kernel_size: usize,
    pub point_density: usize,
    pub near: f64,            // distance in meters
    pub far: f64,             // distance in meters
    pub max_lr_error_px: i64, // Left/right consistency tolerance
    pub max_miss_distance: f64,
    pub min_correlation: f64,
}

impl Default for StereoMeshOptions {
    fn default() -> Self {
        StereoMeshOptions {
            kernel_size: 17,
            point_density: 5,
            near: 1.0,
            far: 5.0,
            max_lr_error_px: 2,
            max_miss_distance: 0.05,
            min_correlation: 0.7,
        }
    }
}

pub fn find_xyz_points(
    left_file_path: &str,
    right_file_path: &str,
    options_opt: &Option<StereoMeshOptions>,
) -> Result<Vec<Vector>> {
    if !path::file_exists(left_file_path) {
        return Err(anyhow!(
            "Error: File not found (left eye): {}",
            left_file_path
        ));
    }

    if !path::file_exists(right_file_path) {
        return Err(anyhow!(
            "Error: File not found (right eye): {}",
            right_file_path
        ));
    }

    let m0 = match MarsImage::open(left_file_path) {
        Ok(img) => img,
        Err(why) => return Err(anyhow!("Failed to open file {}: {:?}", left_file_path, why)),
    };
    let m1 = match MarsImage::open(right_file_path) {
        Ok(img) => img,
        Err(why) => {
            return Err(anyhow!(
                "Failed to open file {}: {:?}",
                right_file_path,
                why
            ))
        }
    };

    let options = match options_opt {
        Some(options) => options,
        None => &StereoMeshOptions::default(),
    };

    let kernel_size = options.kernel_size;
    let k_half = kernel_size / 2;
    let point_density = options.point_density;
    let limit_near = options.near;
    let limit_far = options.far;
    let max_lr_error_px = options.max_lr_error_px;
    let max_miss_distance = options.max_miss_distance;
    let min_correlation = options.min_correlation;

    let cahv0 = match get_cahvor(&m0) {
        Some(cahv) => cahv,
        None => bail!("No CAHVORE model for image 1"),
    };

    let cahv1 = match get_cahvor(&m1) {
        Some(cahv) => cahv,
        None => bail!("No CAHVORE model for image 2"),
    };

    let b0 = m0.image.get_band(0);
    let b1 = m1.image.get_band(0);

    let (lw, lh) = (b0.width, b0.height);
    let (rw, rh) = (b1.width, b1.height);

    let left_px: Vec<f32> = b0.to_vector();
    let right_px: Vec<f32> = b1.to_vector();

    let left_int = Integral::new(&left_px, lw, lh);
    let right_int = Integral::new(&right_px, rw, rh);

    let x0_lo = k_half;
    let x0_hi = m0.image.width - k_half;

    let y0_lo = k_half;
    let y0_hi = m0.image.height - k_half;

    let mut xyz_points: Vec<Vector> = Vec::new();

    for x0 in (x0_lo..x0_hi).step_by(point_density) {
        for y0 in (y0_lo..y0_hi).step_by(point_density) {
            let (lx, ly) = (x0 - k_half, y0 - k_half);
            if lx + kernel_size > lw || ly + kernel_size > lh {
                continue;
            }

            let cands = epipolar_candidates(&cahv0, &cahv1, x0, y0, limit_near, limit_far);

            let lstats = left_int.window(lx, ly, kernel_size);

            let mut best = PointMatch::default();

            for (cx, cy) in cands {
                for dy in -max_lr_error_px..=max_lr_error_px {
                    let (ccx, ccy) = (cx, cy + dy);
                    let (rx, ry) = (ccx - k_half as i64, ccy - k_half as i64); // top-left
                    if rx < 0
                        || ry < 0
                        || rx as usize + kernel_size > rw
                        || ry as usize + kernel_size > rh
                    {
                        continue;
                    }
                    let (rx, ry) = (rx as usize, ry as usize);

                    let c = ncc(
                        &left_px,
                        lw,
                        lx,
                        ly,
                        lstats,
                        &right_px,
                        rw,
                        rx,
                        ry,
                        &right_int,
                        kernel_size,
                    );
                    if c > best.score {
                        best.score = c;
                        best.rx = rx;
                        best.ry = ry;
                    }
                }
            }

            if best.score >= min_correlation {
                let lv0 = cahv0
                    .ls_to_look_vector(&ImageCoordinate {
                        line: y0 as f64,
                        sample: x0 as f64,
                    })
                    .unwrap();
                let lv1 = cahv1
                    .ls_to_look_vector(&ImageCoordinate {
                        line: (best.ry + k_half) as f64,
                        sample: (best.rx + k_half) as f64,
                    })
                    .unwrap();

                if let Some(pt) = triangulate(
                    &lv0.origin,
                    &lv0.look_direction,
                    &lv1.origin,
                    &lv1.look_direction,
                ) {
                    if pt.miss_distance < max_miss_distance {
                        xyz_points.push(Vector::new(pt.point.x, pt.point.y, pt.point.z));
                        // reject bad matches
                        // writeln!(
                        //     obj_file_ptr,
                        //     "v {} {} {}",
                        //     pt.point.x, pt.point.y, pt.point.z
                        // )?;
                    }
                }
            }
        }
    }

    Ok(xyz_points)
}
