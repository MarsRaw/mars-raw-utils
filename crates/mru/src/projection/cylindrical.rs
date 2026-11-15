use std::str::FromStr as _;

use crate::{
    enums::{Eye, Instrument},
    marsimage::MarsImage,
    prelude::WrapCylindrical as _,
    util,
};
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

static SPHERE_RADIUS: f64 = 100.0;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CylindricalMapExtent {
    pub top_lat: f64,
    pub bottom_lat: f64,
    pub left_lon: f64,
    pub right_lon: f64,
    pub width: usize,
    pub height: usize,
    pub degrees_per_pixel: f64,
}

impl Default for CylindricalMapExtent {
    fn default() -> Self {
        CylindricalMapExtent {
            top_lat: f64::MIN,
            bottom_lat: f64::MAX,
            left_lon: f64::MAX,
            right_lon: f64::MIN,
            width: 0,
            height: 0,
            degrees_per_pixel: 0.0,
        }
    }
}

impl CylindricalMapExtent {
    pub fn union_ll(&mut self, other: &LatLon) {
        self.bottom_lat = self.bottom_lat.min(other.lat);
        self.top_lat = self.top_lat.max(other.lat);
        self.left_lon = self.left_lon.min(other.lon);
        self.right_lon = self.right_lon.max(other.lon);
    }

    pub fn union(&mut self, other: &CylindricalMapExtent) {
        self.bottom_lat = self.bottom_lat.min(other.bottom_lat);
        self.top_lat = self.top_lat.max(other.top_lat);
        self.left_lon = self.left_lon.min(other.left_lon);
        self.right_lon = self.right_lon.max(other.right_lon);
        self.degrees_per_pixel = self.degrees_per_pixel.max(other.degrees_per_pixel);
    }

    pub fn wrap_coord_extents(&mut self) {
        self.top_lat = self.top_lat.wrap_pi_degrees();
        self.bottom_lat = self.bottom_lat.wrap_pi_degrees();
        self.left_lon = self.left_lon.wrap_tau_degrees();
        self.right_lon = self.right_lon.wrap_tau_degrees();
    }

    pub fn dimensions_from_extents(&mut self) {
        self.height = ((self.top_lat - self.bottom_lat) / self.degrees_per_pixel).floor() as usize;
        self.width = ((self.right_lon - self.left_lon) / self.degrees_per_pixel).floor() as usize;
    }
}

pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

pub fn vector_to_cylindrical(v: &Vector) -> LatLon {
    let az = v.y.atan2(v.x).wrap_tau();
    let el = (v.z).atan2(v.x.hypot(v.y)).wrap_pi();
    LatLon {
        lat: -el.to_degrees(),
        lon: az.to_degrees(),
    }
}

pub fn lookvector_to_cylindrical(
    lv: &LookVector,
    quat: Option<&Quaternion>,
    origin_diff: &Vector,
) -> LatLon {
    let ray = lv.intersect_to_sphere(SPHERE_RADIUS);
    let ray_moved = ray.subtract(origin_diff);
    let rotated = if let Some(quat) = quat {
        quat.rotate_vector(&ray_moved)
    } else {
        ray
    };
    vector_to_cylindrical(&rotated)
}

fn get_lat_lon(c: &Cahvore, x: f64, y: f64, quat: &Quaternion) -> Result<LatLon> {
    match c.ls_to_look_vector(&ImageCoordinate { line: y, sample: x }) {
        Ok(lv) => Ok(lookvector_to_cylindrical(
            &lv,
            Some(quat),
            &Vector::default(),
        )),
        Err(e) => Err(e),
    }
}

pub fn determine_map_extent(img: &MarsImage, quat: &Quaternion) -> Result<CylindricalMapExtent> {
    let mut context = CylindricalMapExtent::default();

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
    .for_each(|(x, y)| {
        if let Ok(ll) = get_lat_lon(&model, *x, *y, quat) {
            context.union_ll(&ll);
        }
    });

    let ang_horiz = model.pixel_angle_horiz().to_degrees();
    context.degrees_per_pixel = context.degrees_per_pixel.max(ang_horiz);

    Ok(context)
}

pub struct Extent {
    pub min_x: f64,
    pub min_y: f64,
    pub width: usize,
    pub height: usize,
}

fn project_to_output(
    in_model: &Cahvore,
    out_model: &Cahvore,
    ground: &Vector,
    sample: f64,
    line: f64,
) -> Option<ImageCoordinate> {
    let lv = in_model
        .ls_to_look_vector(&ImageCoordinate { line, sample })
        .ok()?;
    match lv.intersect_to_plane(ground) {
        Some(pt) => out_model.xyz_to_ls(&pt, false).ok(),
        None => out_model.xyz_to_ls(&lv.look_direction, true).ok(),
    }
}

/// Bounding box, in the output camera, of all input images' footprints.
/// `max_dim` caps the result: rays near the horizon project very far away.
pub fn map_extent(
    inputs: &[(&Cahvore, usize, usize)], // (model, width, height) for each eye
    out_model: &Cahvore,
    ground: &Vector,
    samples_per_edge: usize,
    max_dim: usize,
) -> Option<Extent> {
    let (mut min_x, mut min_y) = (f64::MAX, f64::MAX);
    let (mut max_x, mut max_y) = (f64::MIN, f64::MIN);
    let n = samples_per_edge.max(2);

    for &(model, w, h) in inputs {
        let (wm, hm) = (w as f64 - 1.0, h as f64 - 1.0);
        for i in 0..=n {
            let t = i as f64 / n as f64;
            for &(s, l) in &[(t * wm, 0.0), (t * wm, hm), (0.0, t * hm), (wm, t * hm)] {
                if let Some(p) = project_to_output(model, out_model, ground, s, l)
                    && p.sample.is_finite()
                    && p.line.is_finite()
                {
                    min_x = min_x.min(p.sample);
                    max_x = max_x.max(p.sample);
                    min_y = min_y.min(p.line);
                    max_y = max_y.max(p.line);
                }
            }
        }
    }

    if min_x > max_x || min_y > max_y {
        return None;
    }

    let min_x = min_x.floor();
    let min_y = min_y.floor();
    Some(Extent {
        min_x,
        min_y,
        width: (((max_x - min_x).ceil() as usize) + 1).min(max_dim),
        height: (((max_y - min_y).ceil() as usize) + 1).min(max_dim),
    })
}

pub fn determine_map_extent_for_files(
    input_files: &[String],
    quat: &Quaternion,
) -> Result<CylindricalMapExtent> {
    let mut context = CylindricalMapExtent {
        top_lat: f64::MIN,
        bottom_lat: f64::MAX,
        left_lon: f64::MAX,
        right_lon: f64::MIN,
        width: 0,
        height: 0,
        degrees_per_pixel: 0.0,
    };

    input_files.iter().try_for_each(|input_file| {
        let img = match MarsImage::open(input_file) {
            Ok(img) => img,
            Err(why) => bail!("Failed to open file {}: {:?}", input_file, why),
        };

        let map_extent = determine_map_extent(&img, quat)?;
        context.union(&map_extent);
        Ok(())
    })?;

    context.wrap_coord_extents();
    context.dimensions_from_extents();

    Ok(context)
}

fn get_ls_from_map_xy(
    model: &Cahvore,
    map_context: &CylindricalMapExtent,
    x: usize,
    y: usize,
    quat: &Quaternion,
    origin_diff: &Vector,
) -> (f64, f64) {
    let img_x = x as f64;
    let img_y = y as f64;

    let lv = match model.ls_to_look_vector(&ImageCoordinate {
        line: img_y,
        sample: img_x,
    }) {
        Ok(lv) => lv,
        Err(_) => panic!("Unable to convert ls to look vector"),
    };

    let ll = lookvector_to_cylindrical(&lv, Some(quat), origin_diff);
    let lat = ll.lat;
    let lon = ll.lon;

    let out_y_f = map_context.height as f64
        - ((lat - map_context.bottom_lat) / (map_context.top_lat - map_context.bottom_lat)
            * map_context.height as f64);
    let out_x_f = (lon - map_context.left_lon) / (map_context.right_lon - map_context.left_lon)
        * map_context.width as f64;

    (out_x_f, out_y_f)
}

pub fn project_file_to_canvas<D: Drawable>(
    input_file: &str,
    map_context: &CylindricalMapExtent,
    map: &mut D,
    anaglyph: bool,
    quat: &Quaternion,
    initial_origin: &Vector,
) -> Result<()> {
    let mut img = MarsImage::open(input_file)?;

    img.instrument = if let Some(rover_metadata) = img.metadata.as_surface() {
        Instrument::from_str(rover_metadata.instrument.as_str()).unwrap()
    } else {
        panic!("Composite via CAHVORE camera model not supported for non-surface missions")
    };

    let eye = if anaglyph {
        match util::filename_char_at_pos(input_file, 1) {
            'R' => Eye::Right,
            'L' => Eye::Left,
            _ => Eye::DontCare,
        }
    } else {
        Eye::DontCare
    };

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
            let full_frame_x = (x + subframe.x) * scale_factor;
            let full_frame_y = (y + subframe.y) * scale_factor;

            let (tl_x, tl_y) = get_ls_from_map_xy(
                &input_model,
                map_context,
                full_frame_x,
                full_frame_y,
                quat,
                &origin_diff,
            );
            let (tr_x, tr_y) = get_ls_from_map_xy(
                &input_model,
                map_context,
                full_frame_x + scale_factor,
                full_frame_y,
                quat,
                &origin_diff,
            );
            let (bl_x, bl_y) = get_ls_from_map_xy(
                &input_model,
                map_context,
                full_frame_x,
                full_frame_y + scale_factor,
                quat,
                &origin_diff,
            );
            let (br_x, br_y) = get_ls_from_map_xy(
                &input_model,
                map_context,
                full_frame_x + scale_factor,
                full_frame_y + scale_factor,
                quat,
                &origin_diff,
            );

            if !band_0.get_mask_at_point(x, y) {
                continue;
            }

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

            map.paint_square_with_channel_rule(&tl, &bl, &br, &tr, false, |c| {
                (c == 0 && matches!(eye, Eye::Left | Eye::DontCare))
                    || ((c == 1 || c == 2) && matches!(eye, Eye::Right | Eye::DontCare))
            });
        }
    }

    Ok(())
}

pub fn create_composite(
    in_files: &[String],
    anaglyph: bool,
    azimuth_rotation: f64,
) -> Result<Image> {
    let quat = Quaternion::from_pitch_roll_yaw(0.0, 0.0, azimuth_rotation.to_radians());

    let map_context = determine_map_extent_for_files(in_files, &quat)?;
    debug!("Map Context: {:?}", map_context);
    debug!(
        "FOV Vertical: {}",
        map_context.top_lat - map_context.bottom_lat
    );
    debug!(
        "FOV Horizontal: {}",
        map_context.right_lon - map_context.left_lon
    );

    if map_context.width == 0 {
        return Err(anyhow!("Error: Output expected to have zero width"));
    } else if map_context.height == 0 {
        return Err(anyhow!("Error: output expected to have zero height"));
    }

    let mut map = Image::create_masked(map_context.width, map_context.height, true);

    let first_image = MarsImage::open(&in_files[0])?;
    let initial_origin = if let Some(model) = first_image.get_cahvor() {
        model.c()
    } else {
        return Err(anyhow!("Error: Cannot determine initial camera origin"));
    };

    in_files.iter().try_for_each(|in_file| {
        if path::file_exists(in_file) {
            info!("Processing File: {}", in_file);
            project_file_to_canvas(
                in_file,
                &map_context,
                &mut map,
                anaglyph,
                &quat,
                &initial_origin,
            )
        } else {
            bail!("Error: input file not found: {}", in_file)
        }
    })?;

    Ok(map)
}
