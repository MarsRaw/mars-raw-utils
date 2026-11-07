use crate::prelude::*;
use anyhow::{Result, bail};
use sciimg::{
    drawable::{Drawable, Point},
    prelude::*,
    quaternion::Quaternion,
    vector::Vector,
};
use std::str::FromStr;

pub fn get_cahvor(img: &MarsImage) -> Option<Cahvore> {
    img.metadata
        .as_rover()
        .map(|rover_metadata| rover_metadata.camera_model_component_list.clone())
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MapContext {
    pub top_lat: f64,
    pub bottom_lat: f64,
    pub left_lon: f64,
    pub right_lon: f64,
    pub width: usize,
    pub height: usize,
    pub degrees_per_pixel: f64,
}

pub struct LatLon {
    lat: f64,
    lon: f64,
}

fn vector_to_cylindrical(v: &Vector) -> LatLon {
    let az = v.y.atan2(v.x).wrap_tau();
    let el = (v.z).atan2(v.x.hypot(v.y)).wrap_pi();
    LatLon {
        lat: el.to_degrees(),
        lon: az.to_degrees(),
    }
}

fn lookvector_to_cylindrical(
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

static SPHERE_RADIUS: f64 = 100.0;

fn get_lat_lon(c: &Cahvore, x: f64, y: f64, _quat: &Quaternion) -> Result<LatLon> {
    match c.ls_to_look_vector(&ImageCoordinate { line: y, sample: x }) {
        Ok(lv) => Ok(lookvector_to_cylindrical(
            &lv,
            Some(_quat),
            &Vector::default(),
        )),
        Err(e) => Err(e),
    }
}

pub fn determine_map_context(input_files: &[String], quat: &Quaternion) -> Result<MapContext> {
    let mut context = MapContext {
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

        let model = match get_cahvor(&img) {
            Some(model) => model,
            None => bail!("Error: Invalid or missing camera model"),
        };

        let (x0, xh, x1) = (
            -0.5,
            img.image.width as f64 / 2.0 - 0.5,
            img.image.width as f64 - 0.5,
        );
        let (y0, yh, y1) = (
            -0.5,
            img.image.height as f64 / 2.0 - 0.5,
            img.image.height as f64 - 0.5,
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
                context.bottom_lat = context.bottom_lat.min(ll.lat);
                context.top_lat = context.top_lat.max(ll.lat);
                context.left_lon = context.left_lon.min(ll.lon);
                context.right_lon = context.right_lon.max(ll.lon);
            }
        });

        let ang_horiz = model.pixel_angle_horiz().to_degrees();
        context.degrees_per_pixel = context.degrees_per_pixel.max(ang_horiz);

        Ok(())
    })?;

    context.top_lat = context.top_lat.wrap_pi_degrees();
    context.bottom_lat = context.bottom_lat.wrap_pi_degrees();

    context.left_lon = context.left_lon.wrap_tau_degrees();
    context.right_lon = context.right_lon.wrap_tau_degrees();

    context.height =
        ((context.top_lat - context.bottom_lat) / context.degrees_per_pixel).floor() as usize;
    context.width =
        ((context.right_lon - context.left_lon) / context.degrees_per_pixel).floor() as usize;

    Ok(context)
}

fn get_ls_from_map_xy(
    model: &Cahvore,
    map_context: &MapContext,
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

    let out_y_f = (lat - map_context.bottom_lat) / (map_context.top_lat - map_context.bottom_lat)
        * map_context.height as f64;
    let out_x_f = (lon - map_context.left_lon) / (map_context.right_lon - map_context.left_lon)
        * map_context.width as f64;

    (out_x_f, out_y_f)
}

pub fn process_file<D: Drawable>(
    input_file: &str,
    map_context: &MapContext,
    map: &mut D,
    anaglyph: bool,
    quat: &Quaternion,
    initial_origin: &Vector,
) -> Result<()> {
    let mut img = MarsImage::open(input_file)?;

    img.instrument = if let Some(rover_metadata) = img.metadata.as_rover() {
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

    //let pitch = (map_context.top_lat + map_context.bottom_lat) / 2.0;
    //vprintln!("Rotating pitch to {}", pitch);
    //let quat = Quaternion::from_pitch_roll_yaw(0.0, -20.0f64.to_radians(), azimuth_rotation.to_radians());

    match get_cahvor(&img) {
        Some(input_model) => {
            // let input_model = match input_model_nonlinear.model_type() {
            //     ModelType::CAHVORE => {
            //         input_model_nonlinear.convert_to_type(ModelType::CAHVOR).expect("Failed to linearize camera model (forced CAHVOR)")
            //     },
            //     _ => input_model_nonlinear
            // };

            // debug!("");
            // debug!("Input Model C: {:?}", input_model.c());
            // debug!("Input Model A: {:?}", input_model.a());
            // debug!("Input Model H: {:?}", input_model.h());
            // debug!("Input Model V: {:?}", input_model.v());
            // debug!("Input Model O: {:?}", input_model.o());
            // debug!("Input Model R: {:?}", input_model.r());
            // debug!("Input Model E: {:?}", input_model.e());
            // debug!("Input Model Pupil Type: {:?}", input_model.pupil_type);
            // debug!("Input Model Linearity: {:?}", input_model.linearity);
            // debug!("");

            let band_0 = img.image.get_band(0);
            let band_1 = img.image.get_band(1);
            let band_2 = img.image.get_band(2);

            for x in 0..(img.image.width - 1) {
                for y in 0..(img.image.height - 1) {
                    let origin_diff = input_model.c().subtract(initial_origin);

                    let (tl_x, tl_y) =
                        get_ls_from_map_xy(&input_model, map_context, x, y, quat, &origin_diff);
                    let (tr_x, tr_y) =
                        get_ls_from_map_xy(&input_model, map_context, x + 1, y, quat, &origin_diff);
                    let (bl_x, bl_y) =
                        get_ls_from_map_xy(&input_model, map_context, x, y + 1, quat, &origin_diff);
                    let (br_x, br_y) = get_ls_from_map_xy(
                        &input_model,
                        map_context,
                        x + 1,
                        y + 1,
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
        }
        None => {
            error!("CAHVOR not found for image, cannot continue");
            panic!("CAHVOR not found for image, cannot continue");
        }
    }

    Ok(())
}
