use crate::enums::DownsampleScale;
use crate::prelude::*;
use crate::stereo::model_for_image;
use crate::stereo::resample_to_model;
use anyhow::Result;
use anyhow::anyhow;
use rayon::prelude::*;
use sciimg::camera::cahvore::Cahvore;
use sciimg::camera::model::ImageCoordinate;
use sciimg::integral::{ImageIntegral, Integral};
use sciimg::path;
use sciimg::vector::Vector;

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

#[derive(Clone, Default)]
pub struct Vertex {
    pub raster_x: usize,
    pub raster_y: usize,
    pub index: usize,
    pub xyz: Option<Vector>,
}

impl Vertex {
    pub fn has_xyz(&self) -> bool {
        self.xyz.is_some()
    }
}

pub struct XyzRaster {
    pub cols: usize,
    pub rows: usize,
    pub image_width: usize, // left image size, for UV generation
    pub image_height: usize,
    pub xyz_grid: Vec<Vec<Vertex>>,
    pub origin: Vector,
    pub left_eye_img: MarsImage,
    pub right_eye_img: MarsImage,
}

macro_rules! check_xyz_in_bounds_xyz_raster {
    ($self:ident, $y:expr, $x:expr) => {
        if $y >= $self.rows {
            return Err(anyhow!("Row index out of bounds: {}", $y));
        } else if $x >= $self.cols {
            return Err(anyhow!("Column index out of bounds: {}", $x));
        }
    };
}

impl XyzRaster {
    pub fn new(
        rows: usize,
        cols: usize,
        image_width: usize,
        image_height: usize,
        origin: &Vector,
        left: &MarsImage,
        right: &MarsImage,
    ) -> Result<Self> {
        if rows == 0 || cols == 0 {
            return Err(anyhow!(
                "Error: Cannot create raster with zero length dimension"
            ));
        }

        Ok(Self {
            cols,
            rows,
            image_width,
            image_height,

            // Defaults with x/y values of zero are technically inconsistent and invalid in this contxt, but no-one's gonna come to arrest me....
            xyz_grid: vec![vec![Vertex::default(); cols]; rows],
            origin: origin.to_owned(),
            left_eye_img: left.to_owned(),
            right_eye_img: right.to_owned(),
        })
    }

    pub fn put(&mut self, row: usize, col: usize, vtx: Vertex) -> Result<()> {
        check_xyz_in_bounds_xyz_raster!(self, row, col);

        self.xyz_grid[row][col] = vtx;
        Ok(())
    }

    pub fn get(&self, row: usize, col: usize) -> Result<&Vertex> {
        check_xyz_in_bounds_xyz_raster!(self, row, col);
        Ok(&self.xyz_grid[row][col])
    }

    pub fn point_has_xyz(&self, row: usize, col: usize) -> Result<bool> {
        check_xyz_in_bounds_xyz_raster!(self, row, col);
        Ok(self.xyz_grid[row][col].has_xyz())
    }

    pub fn xyz_for_point(&self, row: usize, col: usize) -> Result<&Option<Vector>> {
        check_xyz_in_bounds_xyz_raster!(self, row, col);
        Ok(&self.xyz_grid[row][col].xyz)
    }

    pub fn reject_height_outliers(&mut self, max_dev: f64) -> usize {
        let mut zs: Vec<f64> = self
            .xyz_grid
            .iter()
            .flatten()
            .filter_map(|v| v.xyz.as_ref().map(|p| p.z))
            .collect();
        if zs.is_empty() {
            return 0;
        }
        zs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = zs[zs.len() / 2];

        let mut removed = 0;
        for vtx in self.xyz_grid.iter_mut().flatten() {
            if let Some(p) = &vtx.xyz
                && (p.z - med).abs() > max_dev
            {
                vtx.xyz = None;
                removed += 1;
            }
        }
        removed
    }

    pub fn provide_mesh_indexes(&mut self) {
        self.xyz_grid
            .iter_mut()
            .flatten()
            .filter(|vtx| vtx.has_xyz())
            .enumerate()
            .for_each(|(idx, vtx)| {
                vtx.index = idx;
            });
    }

    pub fn as_flat_array(&self) -> Vec<&Vertex> {
        self.xyz_grid.iter().flatten().collect()
    }

    pub fn as_flat_array_only_valids(&self) -> Vec<&Vertex> {
        self.xyz_grid
            .iter()
            .flatten()
            .filter(|vtx| vtx.has_xyz())
            .collect()
    }
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
    pub max_height_deviations: f64,
    pub downsample: DownsampleScale,
    pub linearize: bool,
}

impl Default for StereoMeshOptions {
    fn default() -> Self {
        StereoMeshOptions {
            kernel_size: 25,
            point_density: 1,
            near: 1.0,
            far: 10.0,
            max_lr_error_px: 5,
            max_miss_distance: 0.1,
            min_correlation: 0.4,
            max_height_deviations: 0.3,
            downsample: DownsampleScale::ScaleFactor1,
            linearize: false,
        }
    }
}

pub fn find_xyz_points(
    left_file_path: &str,
    right_file_path: &str,
    options_opt: &Option<StereoMeshOptions>,
) -> Result<XyzRaster> {
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

    let options = match options_opt {
        Some(options) => options,
        None => &StereoMeshOptions::default(),
    };

    let m0 = MarsImage::open(left_file_path)?.downsample(options.downsample)?;
    let m1 = MarsImage::open(right_file_path)?.downsample(options.downsample)?;

    let kernel_size = options.kernel_size;
    let k_half = kernel_size / 2;
    let point_density = options.point_density;
    let limit_near = options.near;
    let limit_far = options.far;
    let max_lr_error_px = options.max_lr_error_px;
    let max_miss_distance = options.max_miss_distance;
    let min_correlation = options.min_correlation;
    let max_height_deviations = options.max_height_deviations;

    let cahv0 = match model_for_image(&m0, true) {
        Some(model) => model,
        None => return Err(anyhow!("No camera model for image. Cannot resample.")),
    };

    let cahv1 = match model_for_image(&m1, true) {
        Some(model) => model,
        None => return Err(anyhow!("No camera model for image. Cannot resample.")),
    };

    let (left_model, right_model, left_img, right_img) = if options.linearize {
        let left_linearized_model = cahv0.linearize(
            m0.image.width,
            m0.image.height,
            m0.image.width,
            m0.image.height,
        )?;

        let right_linearized_model = cahv1.linearize(
            m1.image.width,
            m1.image.height,
            m1.image.width,
            m1.image.height,
        )?;

        let rect_l = resample_to_model(
            &m0,
            left_linearized_model.clone(),
            m0.image.width,
            m0.image.height,
        )?;
        let rect_r = resample_to_model(
            &m1,
            right_linearized_model.clone(),
            m0.image.width,
            m0.image.height,
        )?;

        (
            left_linearized_model,
            right_linearized_model,
            rect_l,
            rect_r,
        )
    } else {
        (cahv0, cahv1, m0, m1)
    };

    let left_int = left_img.image.to_mono_integral_basic(kernel_size)?;
    let right_int = right_img.image.to_mono_integral_basic(kernel_size)?;

    let (lw, lh) = (left_img.image.width, left_img.image.height);
    let (rw, rh) = (right_img.image.width, right_img.image.height);

    let x0_lo = k_half;
    let x0_hi = left_img.image.width - k_half;

    let y0_lo = k_half;
    let y0_hi = left_img.image.height - k_half;

    let grid_width = (x0_hi - x0_lo).div_ceil(point_density);
    let grid_height = (y0_hi - y0_lo).div_ceil(point_density);

    let match_point = |x0: usize, y0: usize| -> Vertex {
        let mut vtx = Vertex {
            raster_x: x0,
            raster_y: y0,
            ..Default::default()
        };

        let (lx, ly) = (x0 - k_half, y0 - k_half);
        if lx + kernel_size > lw || ly + kernel_size > lh {
            return vtx;
        }

        let cands = epipolar_candidates(&left_model, &right_model, x0, y0, limit_near, limit_far);
        let lstats = left_int.window(lx, ly, kernel_size);

        let mut best = PointMatch::default();
        for (cx, cy) in cands {
            for dy in -max_lr_error_px..=max_lr_error_px {
                let (rx, ry) = (cx - k_half as i64, cy + dy - k_half as i64);
                if rx < 0
                    || ry < 0
                    || rx as usize + kernel_size > rw
                    || ry as usize + kernel_size > rh
                {
                    continue;
                }
                let (rx, ry) = (rx as usize, ry as usize);
                let c = ncc(
                    &left_int.data,
                    lw,
                    lx,
                    ly,
                    lstats,
                    &right_int.data,
                    rw,
                    rx,
                    ry,
                    &right_int,
                    kernel_size,
                );
                if c > best.score {
                    best = PointMatch { score: c, rx, ry };
                }
            }
        }
        if best.score < min_correlation {
            return vtx;
        }

        let (Ok(lv0), Ok(lv1)) = (
            left_model.ls_to_look_vector(&ImageCoordinate {
                line: y0 as f64,
                sample: x0 as f64,
            }),
            right_model.ls_to_look_vector(&ImageCoordinate {
                line: (best.ry + k_half) as f64,
                sample: (best.rx + k_half) as f64,
            }),
        ) else {
            return vtx;
        };

        vtx.xyz = triangulate(
            &lv0.origin,
            &lv0.look_direction,
            &lv1.origin,
            &lv1.look_direction,
        )
        .filter(|t| t.miss_distance < max_miss_distance)
        .map(|t| Vector::new(t.point.x, t.point.y, t.point.z));
        vtx
    };

    // Parallel over grid rows, results come back in order
    let xyz_grid: Vec<Vec<Vertex>> = (0..grid_height)
        .into_par_iter()
        .map(|gy| {
            let y0 = y0_lo + gy * point_density;
            (0..grid_width)
                .map(|gx| match_point(x0_lo + gx * point_density, y0))
                .collect()
        })
        .collect();

    let mut grid = XyzRaster::new(
        grid_height,
        grid_width,
        lw,
        lh,
        &left_model.c,
        &left_img,
        &right_img,
    )?;
    grid.xyz_grid = xyz_grid;

    grid.reject_height_outliers(max_height_deviations);
    grid.provide_mesh_indexes();

    Ok(grid)
}
