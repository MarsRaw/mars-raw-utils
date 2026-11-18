use std::f64::consts::FRAC_PI_2;

use anyhow::{Result, anyhow, bail};
use sciimg::{
    camera::{
        cahvore::Cahvore,
        model::{ImageCoordinate, LookVector},
    },
    drawable::{Drawable, Point},
    image::Image,
    path,
    quaternion::Quaternion,
    vector::Vector,
};

use crate::{enums::Projection, marsimage::MarsImage};

const SPHERE_RADIUS: f64 = 100.0;
const RECTILINEAR_MAX_THETA: f64 = std::f64::consts::FRAC_PI_2 - 1e-6;

#[derive(Debug, Clone, Copy)]
pub struct PolarXY {
    pub x: f64,
    pub y: f64,

    // Angle off the chosen pole
    pub theta: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum PolarView {
    /// Nadir-centered, out to a ground distance
    Ground {
        max_radius: f64,
        eye_height: Option<f64>,
    },
    /// Nadir-centered, up to the horizon (plus optional degrees above it)
    GroundAngle { max_zenith_deg: f64 },
    /// Zenith-centered, down to the horizon (plus optional degrees below it)
    Sky { max_zenith_deg: f64 },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PolarMapExtent {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub max_radius: f64,
    pub max_edge_length: usize,
    pub eye_height: f64,
    pub pole: Pole,
    pub max_theta: f64,
}

impl PolarMapExtent {
    pub fn xy_to_map_xy(&self, x: f64, y: f64) -> (f64, f64) {
        let h_len = (self.max_edge_length / 2) as f64;
        let m_x = x / self.max_radius * h_len + h_len;
        let m_y = y / self.max_radius * h_len + h_len;
        (m_x, m_y)
    }

    pub fn empty() -> Self {
        PolarMapExtent {
            min_x: f64::MAX,
            max_x: f64::MIN,
            min_y: f64::MAX,
            max_y: f64::MIN,
            max_radius: f64::MIN,
            max_edge_length: 0,
            eye_height: 0.0,
            pole: Pole::Zenith,
            max_theta: 0.0,
        }
    }

    pub fn include(&mut self, x: f64, y: f64) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }

    pub fn union_xy(&mut self, o: &PolarXY) {
        self.include(o.x, o.y);
    }

    pub fn union(&mut self, o: &PolarMapExtent) {
        if o.min_x <= o.max_x {
            self.include(o.min_x, o.min_y);
            self.include(o.max_x, o.max_y);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min_x >= self.max_x
    }
}

/// Which pole sits at the center of the polar plot.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Pole {
    #[default]
    Zenith, // up
    Nadir, // down
}

/// Radial mapping from the angle off the center pole.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum RadialMap {
    /// r = angle from pole (equidistant / azimuthal equidistant)
    #[default]
    Equidistant,
    /// r = 2 * tan(angle / 2) (stereographic, conformal)
    Stereographic,
    /// r = 2 * sin(angle / 2) (equal-area, Lambert azimuthal)
    EqualArea,
    /// r = sin(angle) (orthographic, true "from above" look)
    Orthographic,
    /// r = tan(theta) (gnomonic / rectilinear, one hemisphere only)
    Rectilinear,
}

impl RadialMap {
    pub fn project(&self, theta: f64) -> f64 {
        match self {
            RadialMap::Equidistant => theta,
            RadialMap::Stereographic => 2.0 * (theta / 2.0).tan(),
            RadialMap::EqualArea => 2.0 * (theta / 2.0).sin(),
            RadialMap::Orthographic => theta.sin(),
            RadialMap::Rectilinear => theta.min(RECTILINEAR_MAX_THETA).tan(),
        }
    }

    pub fn from_projection(projection: &Projection) -> Result<RadialMap> {
        match projection {
            Projection::PolarEquidistant => Ok(RadialMap::Equidistant),
            Projection::PolarStereographic => Ok(RadialMap::Stereographic),
            Projection::PolarEqualArea => Ok(RadialMap::EqualArea),
            Projection::PolarOrthographic => Ok(RadialMap::Orthographic),
            Projection::PolarRectilinear => Ok(RadialMap::Rectilinear),
            _ => Err(anyhow!("Invalid polar projection")),
        }
    }
}

pub fn vector_to_polar(v: &Vector, pole: Pole, map: RadialMap) -> PolarXY {
    let cos_theta = match pole {
        Pole::Zenith => -v.z,
        Pole::Nadir => v.z,
    };

    let theta = v.x.hypot(v.y).atan2(cos_theta);

    let r = map.project(theta);

    // Azimuth, same definition as the cylindrical version
    let az = v.y.atan2(v.x);

    PolarXY {
        x: r * az.cos(),
        y: r * az.sin(),
        theta,
    }
}

pub fn lookvector_to_polar(
    lv: &LookVector,
    quat: Option<&Quaternion>,
    origin_diff: &Vector,
    pole: Pole,
    map: RadialMap,
) -> PolarXY {
    let ray = lv.intersect_to_sphere(SPHERE_RADIUS);
    let ray_moved = ray.subtract(origin_diff);
    let rotated = if let Some(quat) = quat {
        quat.rotate_vector(&ray_moved)
    } else {
        ray
    };
    vector_to_polar(&rotated, pole, map)
}

fn get_polar_xy(
    c: &Cahvore,
    x: f64,
    y: f64,
    quat: &Quaternion,
    origin_diff: &Vector,
    pole: Pole,
    radial_map: RadialMap,
) -> Result<PolarXY> {
    match c.ls_to_look_vector(&ImageCoordinate { line: y, sample: x }) {
        Ok(lv) => Ok(lookvector_to_polar(
            &lv,
            Some(quat),
            origin_diff,
            pole,
            radial_map,
        )),
        Err(why) => Err(anyhow!("Unable to convert ls to look vector: {:?}", why)),
    }
}

fn get_map_xy_from_img_xy(
    model: &Cahvore,
    map_extent: &PolarMapExtent,
    x: usize,
    y: usize,
    quat: &Quaternion,
    origin_diff: &Vector,
    radial_map: RadialMap,
) -> Result<Option<(f64, f64)>> {
    let lv = model
        .ls_to_look_vector(&ImageCoordinate {
            line: y as f64,
            sample: x as f64,
        })
        .map_err(|why| anyhow!("Unable to convert ls to look vector: {:?}", why))?;

    let xy = lookvector_to_polar(&lv, Some(quat), origin_diff, map_extent.pole, radial_map);

    if xy.theta > map_extent.max_theta {
        return Ok(None); // beyond the horizon (or the ground radius)
    }
    Ok(Some(map_extent.xy_to_map_xy(xy.x, xy.y)))
}

pub fn determine_map_extent(
    img: &MarsImage,
    quat: &Quaternion,
    pole: Pole,
    radial_map: RadialMap,
) -> Result<PolarMapExtent> {
    let mut extent = PolarMapExtent::default();

    let scale_factor = img.get_scale_factor();
    let model = match img.get_cahvor() {
        Some(model) => model.scale(scale_factor as f64, scale_factor as f64),
        None => bail!("Error: Invalid or missing camera model"),
    };

    let (x0, xh, x1) = (
        -0.5,
        (img.image.width as f64 / 2.0 - 0.5) * scale_factor as f64,
        (img.image.width as f64 - 0.5) * scale_factor as f64,
    );
    let (y0, yh, y1) = (
        -0.5,
        (img.image.height as f64 / 2.0 - 0.5) * scale_factor as f64,
        (img.image.height as f64 - 0.5) * scale_factor as f64,
    );

    // Here we check the four corners and the middle point of each four sides
    [
        (x0, y0),
        (x1, y0),
        (x0, y1),
        (x1, y1),
        (xh, y0),
        (xh, y1),
        (x0, yh),
        (x1, yh),
    ]
    .iter()
    .try_for_each(|(x, y)| {
        let polar_xy =
            match get_polar_xy(&model, *x, *y, quat, &Vector::default(), pole, radial_map) {
                Ok(polar_xy) => polar_xy,
                Err(why) => bail!("Failed to determine polar xy point: {:?}", why),
            };
        extent.union_xy(&polar_xy);
        Ok(())
    })?;

    Ok(extent)
}

fn compute_eye_height_from_model(img: &MarsImage) -> Result<f64> {
    let initial_origin = if let Some(model) = img.get_cahvor() {
        model.c()
    } else {
        return Err(anyhow!("Error: Cannot determine initial camera origin"));
    };

    let mut surface_origin = initial_origin;
    surface_origin.z = 0.0;
    let computed_eye_height = surface_origin.distance_to(&initial_origin);
    Ok(computed_eye_height)
}

fn view_limits(view: PolarView, map: RadialMap, first: &MarsImage) -> Result<(Pole, f64, f64)> {
    // returns (pole, max_theta, eye_height)
    match view {
        PolarView::Ground {
            max_radius,
            eye_height,
        } => {
            let h = match eye_height {
                Some(h) => h,
                None => compute_eye_height_from_model(first)?,
            };
            Ok((Pole::Nadir, max_radius.atan2(h), h))
        }
        PolarView::GroundAngle { max_zenith_deg } => {
            if !(max_zenith_deg > 0.0 && max_zenith_deg <= 180.0) {
                bail!(
                    "max zenith angle must be in (0, 180] degrees, got {}",
                    max_zenith_deg
                );
            }
            let max_theta = max_zenith_deg.to_radians();

            match map {
                RadialMap::Rectilinear if max_theta >= RECTILINEAR_MAX_THETA => {
                    bail!(
                        "Rectilinear cannot reach 90 degrees from nadir; use a smaller angle or another radial map"
                    )
                }
                RadialMap::Orthographic if max_theta > FRAC_PI_2 => {
                    bail!("Orthographic folds back past 90 degrees from nadir; use 90 or less")
                }
                RadialMap::Stereographic if max_theta >= std::f64::consts::PI - 1e-6 => {
                    bail!("Stereographic diverges at 180 degrees from nadir; use a smaller angle")
                }
                _ => {}
            }
            Ok((Pole::Nadir, max_theta, 0.0))
        }

        PolarView::Sky { max_zenith_deg } => {
            if !(max_zenith_deg > 0.0 && max_zenith_deg <= 180.0) {
                bail!(
                    "max zenith angle must be in (0, 180] degrees, got {}",
                    max_zenith_deg
                );
            }
            let max_theta = max_zenith_deg.to_radians();

            match map {
                RadialMap::Rectilinear if max_theta >= RECTILINEAR_MAX_THETA => {
                    bail!(
                        "Rectilinear cannot reach 90 degrees from zenith; use a smaller angle or another radial map"
                    )
                }
                RadialMap::Orthographic if max_theta > FRAC_PI_2 => {
                    bail!("Orthographic folds back past 90 degrees from zenith; use 90 or less")
                }
                RadialMap::Stereographic if max_theta >= std::f64::consts::PI - 1e-6 => {
                    bail!("Stereographic diverges at 180 degrees from zenith; use a smaller angle")
                }
                _ => {}
            }
            Ok((Pole::Zenith, max_theta, 0.0))
        }
    }
}

pub fn determine_map_extent_for_files(
    input_files: &[String],
    quat: &Quaternion,
    max_edge_length: usize,
    view: PolarView,
    radial_map: RadialMap,
) -> Result<PolarMapExtent> {
    let mut extent = PolarMapExtent::default();

    if input_files.is_empty() {
        return Err(anyhow!("Cannot determine map extent: no input files"));
    }

    let first = MarsImage::open(input_files.first().unwrap())?;
    let (pole, max_theta, eye_height) = view_limits(view, radial_map, &first)?;
    extent.pole = pole;
    extent.max_theta = max_theta;
    extent.max_radius = radial_map.project(max_theta);
    extent.eye_height = eye_height;
    extent.max_edge_length = max_edge_length;

    input_files.iter().try_for_each(|input_file| {
        let img = match MarsImage::open(input_file) {
            Ok(img) => img,
            Err(why) => bail!("Failed to open file {}: {:?}", input_file, why),
        };

        let map_extent = determine_map_extent(&img, quat, pole, radial_map)?;
        extent.union(&map_extent);
        Ok(())
    })?;

    Ok(extent)
}

pub fn project_file_to_canvas<D: Drawable>(
    input_file: &str,
    map_extent: &PolarMapExtent,
    map: &mut D,
    quat: &Quaternion,
    initial_origin: &Vector,
    radial_map: RadialMap,
) -> Result<()> {
    let img = MarsImage::open(input_file)?;

    let subframe = img.get_subframe_region();
    let scale_factor = img.get_scale_factor();

    let mut input_model = match img.get_cahvor() {
        Some(input_model) => input_model,
        None => return Err(anyhow!("CAHVOR not found for image, cannot continue")),
    };

    input_model = input_model.scale(scale_factor as f64, scale_factor as f64);

    let band_0 = img.image.get_band(0);
    let band_1 = img.image.get_band(1);
    let band_2 = img.image.get_band(2);

    let origin_diff = input_model.c().subtract(initial_origin);

    for x in 0..(img.image.width - 1) {
        for y in 0..(img.image.height - 1) {
            if !band_0.get_mask_at_point(x, y) {
                continue;
            }

            let full_frame_x = (x + subframe.x) * scale_factor;
            let full_frame_y = (y + subframe.y) * scale_factor;

            let (Some((tl_x, tl_y)), Some((tr_x, tr_y)), Some((bl_x, bl_y)), Some((br_x, br_y))) = (
                get_map_xy_from_img_xy(
                    &input_model,
                    map_extent,
                    full_frame_x,
                    full_frame_y,
                    quat,
                    &origin_diff,
                    radial_map,
                )?,
                get_map_xy_from_img_xy(
                    &input_model,
                    map_extent,
                    full_frame_x + scale_factor,
                    full_frame_y,
                    quat,
                    &origin_diff,
                    radial_map,
                )?,
                get_map_xy_from_img_xy(
                    &input_model,
                    map_extent,
                    full_frame_x,
                    full_frame_y + scale_factor,
                    quat,
                    &origin_diff,
                    radial_map,
                )?,
                get_map_xy_from_img_xy(
                    &input_model,
                    map_extent,
                    full_frame_x + scale_factor,
                    full_frame_y + scale_factor,
                    quat,
                    &origin_diff,
                    radial_map,
                )?,
            ) else {
                continue;
            };

            let tl = Point::create_rgb(
                tl_x,
                tl_y,
                band_0.get(x, y) as f64,
                band_1.get(x, y) as f64,
                band_2.get(x, y) as f64,
            );

            let tr = Point::create_rgb(
                tr_x,
                tr_y,
                band_0.get(x + 1, y) as f64,
                band_1.get(x + 1, y) as f64,
                band_2.get(x + 1, y) as f64,
            );

            let bl = Point::create_rgb(
                bl_x,
                bl_y,
                band_0.get(x, y + 1) as f64,
                band_1.get(x, y + 1) as f64,
                band_2.get(x, y + 1) as f64,
            );

            let br = Point::create_rgb(
                br_x,
                br_y,
                band_0.get(x + 1, y + 1) as f64,
                band_1.get(x + 1, y + 1) as f64,
                band_2.get(x + 1, y + 1) as f64,
            );

            map.paint_square(&tl, &bl, &br, &tr, false);
        }
    }

    Ok(())
}

pub fn create_composite(
    in_files: &[String],
    _anaglyph: bool,
    azimuth_rotation: f64,
    radial_map: RadialMap,
    max_edge_length: usize,
    view: PolarView,
) -> Result<Image> {
    if _anaglyph {
        return Err(anyhow!("Anaglyph unsupported for polar projections"));
    }

    let first_image = MarsImage::open(&in_files[0])?;
    let initial_origin = if let Some(model) = first_image.get_cahvor() {
        model.c()
    } else {
        return Err(anyhow!("Error: Cannot determine initial camera origin"));
    };

    let quat = Quaternion::from_pitch_roll_yaw(0.0, 0.0, azimuth_rotation.to_radians());

    let map_extent =
        determine_map_extent_for_files(in_files, &quat, max_edge_length, view, radial_map)?;
    debug!("Map Context: {:?}", map_extent);

    let mut map = Image::create_masked(max_edge_length, max_edge_length, true);

    in_files.iter().try_for_each(|in_file| {
        if path::file_exists(in_file) {
            info!("Processing File: {}", in_file);
            project_file_to_canvas(
                in_file,
                &map_extent,
                &mut map,
                &quat,
                &initial_origin,
                radial_map,
            )
        } else {
            bail!("Error: input file not found: {}", in_file)
        }
    })?;

    Ok(map)
}
