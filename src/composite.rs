use crate::prelude::*;
use itertools::iproduct;
use sciimg::{
    camera::cahvore, drawable::*, matrix::Matrix, max, min, prelude::*, quaternion::Quaternion,
    vector::Vector,
};
use std::str::FromStr;

pub fn get_cahvor(img: &MarsImage) -> Option<CameraModel> {
    match &img.metadata {
        Some(md) => {
            if md.camera_model_component_list.is_valid() {
                Some(md.camera_model_component_list.clone())
            } else {
                None
            }
        }
        None => None,
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MapContext {
    pub width: usize,
    pub height: usize,
}

#[derive(PartialEq)]
enum PlaneIntersectionDirection {
    Parallel,
    Infinity,
    Normal,
}

struct CahvIPlane {
    ppnt: Vector, // Projection point
    ndir: Vector, // Normal direction
    hdir: Vector, // Horizontal direction
    vdir: Vector, // Vertical direction
    hc: f64,      // Horizontal center
    vc: f64,      // Vertical Center
}

struct Cahv3d {
    pos3: Vector,  // 3d origin of projection
    uvec3: Vector, // Unit vector ray of projection
    par: Matrix,   // Partial derivative of uvec3 and pos2
}

fn cahv_2d_to_3d(pos2: &[f64; 2], model: &CameraModel) -> Cahv3d {
    let pos3 = model.c();

    let f = model.v().subtract(&model.a().scale(pos2[1]));

    let g = model.h().subtract(&model.a().scale(pos2[0]));

    let mut uvec3 = g.cross_product(&f);
    let magi = 1.0 / uvec3.len();
    uvec3 = uvec3.scale(magi);

    let mut t = model.h().cross_product(&model.v());

    let sgn = if t.dot_product(&model.a()) < 0.0 {
        uvec3 = uvec3.scale(-1.0);
        -1.0
    } else {
        1.0
    };

    let mut irrt = Matrix::identity();
    for (i, j) in iproduct!(0..3, 0..3) {
        irrt.set(i, j, irrt.get(i, j) - uvec3.get(i) * uvec3.get(j));
    }

    t = model.a().cross_product(&f);
    let u = irrt.multiply_vector(&t);

    let mut par = Matrix::identity();
    par.set(0, 0, -sgn * u.x * magi);
    par.set(1, 0, -sgn * u.y * magi);
    par.set(2, 0, -sgn * u.z * magi);

    t = model.a().cross_product(&g);
    let u = irrt.multiply_vector(&t);
    par.set(0, 1, sgn * u.x * magi);
    par.set(1, 1, sgn * u.y * magi);
    par.set(2, 1, sgn * u.z * magi);

    Cahv3d { pos3, uvec3, par }
}

fn cahv_iplane(model: &CameraModel) -> CahvIPlane {
    let ppnt = model.c();
    let ndir = model.a();
    let hc = model.a().dot_product(&model.h());
    let vc = model.a().dot_product(&model.v());

    let p2: [f64; 2] = [hc, vc];

    let c3d = cahv_2d_to_3d(&p2, &model);
    let u3 = Vector::new(c3d.par.get(0, 0), c3d.par.get(1, 0), c3d.par.get(2, 0));
    let hdir = u3.unit_vector();
    let u3 = Vector::new(c3d.par.get(0, 1), c3d.par.get(1, 1), c3d.par.get(2, 1));
    let vdir = u3.unit_vector();

    CahvIPlane {
        ppnt,
        ndir,
        hdir,
        vdir,
        hc,
        vc,
    }
}

fn cahvore_warp_limit() {}

pub fn warp_cahvore_models(
    model1: &CameraModel,
    xdim1: usize,
    ydim1: usize,
    model2: &CameraModel,
    xdim2: usize,
    ydim2: usize,
    xdim: usize,
    ydim: usize,
) {
    let iplane1 = cahv_iplane(&model1);
    let iplane2 = cahv_iplane(&model2);
}

pub fn warp_cahv_models(model1: &CameraModel, model2: &CameraModel) -> CameraModel {
    let hc = (model1.h().dot_product(&model1.a()) + model2.h().dot_product(&model2.a())) / 2.0;
    let vc = (model1.v().dot_product(&model1.a()) + model2.v().dot_product(&model2.a())) / 2.0;
    let hs = (model1.h().cross_product(&model1.a()).len()
        + model2.h().cross_product(&model2.a()).len())
        / 2.0;
    let vs = (model1.v().cross_product(&model1.a()).len()
        + model2.v().cross_product(&model2.a()).len())
        / 2.0;

    let theta = -std::f64::consts::PI / 2.0;
    let mut app = model1.a().add(&model2.a());
    let mut f = model1.c().subtract(&model2.c());
    let hp = if f.dot_product(&model1.h()) > 0.0 {
        f.scale(hs / f.len())
    } else {
        f.scale(-(hs / f.len()))
    };
    app = app.scale(0.5);
    let g = hp.scale(app.dot_product(&hp) / (hs * hs));
    let ap = app.subtract(&g);
    let a = ap.unit_vector();
    f = hp.cross_product(&a);
    let vp = f.scale(vs / hs);
    f = a.scale(hc);
    let h = hp.add(&f);
    f = a.scale(vc);
    let v = vp.add(&f);

    CameraModel::new(Box::new(Cahv {
        c: model1.c(),
        a: a,
        h: h,
        v: v,
    }))
}

fn intersect_ray(lv: &LookVector) -> (PlaneIntersectionDirection, Vector) {
    let normal = Vector::new(0.0, 0.0, -1.0);
    let ground = Vector::new(0.0, 0.0, 1.90092);

    let look_direction = lv.look_direction;
    let origin = lv.origin;

    let dot = look_direction.dot_product(&normal);

    if dot == 0.0 {
        (PlaneIntersectionDirection::Parallel, look_direction)
    } else {
        let ratio = ground.subtract(&origin).dot_product(&normal) / dot;

        if ratio < 0.0 {
            (PlaneIntersectionDirection::Infinity, look_direction)
        } else {
            let intersect_point = origin.add(&look_direction.scale(ratio));
            // println!(
            //     "{}, {}, {}",
            //     intersect_point.x, intersect_point.y, intersect_point.z
            // );
            (PlaneIntersectionDirection::Normal, intersect_point)
        }
    }
}

pub fn determine_map_context(
    input_images: &[MarsImage],
    quat: &Quaternion,
    out_model: &CameraModel,
) -> MapContext {
    let mut x_min = std::f64::MAX;
    let mut y_min = std::f64::MAX;

    let mut x_max = std::f64::MIN;
    let mut y_max = std::f64::MIN;

    //input_files.iter().for_each(|input_file| {
    //    let img = MarsImage::open(input_file.to_owned(), Instrument::M20MastcamZLeft);
    input_images.iter().for_each(|img| {
        if let Some(in_model) = get_cahvor(&img) {
            if let Ok((y, x)) = project_ls(&in_model, &in_model, 0, 0) {
                vprintln!("{}, {}", y, x);
                x_min = min!(x_min, x);
                x_max = max!(x_max, x);

                y_min = min!(y_min, y);
                y_max = max!(y_max, y);
            }

            if let Ok((y, x)) = project_ls(&in_model, &in_model, img.image.width, img.image.height)
            {
                vprintln!("{}, {}", y, x);
                x_min = min!(x_min, x);
                x_max = max!(x_max, x);

                y_min = min!(y_min, y);
                y_max = max!(y_max, y);
            }
        }
    });
    vprintln!("Composite y range: {} - {}", y_max, y_min);
    vprintln!("Composite x range: {} - {}", x_max, x_min);
    MapContext {
        height: (y_max - y_min).round() as usize + 5000,
        width: (x_max - x_min).round() as usize + 5000,
    }
}

pub fn project_ls(
    in_model: &CameraModel,
    out_model: &CameraModel,
    x: usize,
    y: usize,
) -> error::Result<(f64, f64)> {
    let lv = if let Ok(lv) = in_model.ls_to_look_vector(&ImageCoordinate {
        line: y as f64,
        sample: x as f64,
    }) {
        lv
    } else {
        panic!("Error convering line sample to look vector");
    };

    let (intersection_type, surf_pt) = intersect_ray(&lv);

    let ls = if intersection_type != PlaneIntersectionDirection::Normal {
        let uvec3 = lv.look_direction.clone(); // If infinity
        let p = if out_model.a().dot_product(&uvec3) >= 0.0 {
            uvec3.add(&out_model.c())
        } else {
            uvec3.inversed().add(&out_model.c())
        };
        out_model.xyz_to_ls(&p, true)
    } else {
        out_model.xyz_to_ls(&surf_pt, false)
    };

    match ls {
        Ok(ls) => Ok((ls.line, ls.sample)),
        Err(why) => Err(why),
    }
}

/// The shit in this function isn't even close to correct. I know this. Please don't judge.
pub fn process_file<D: Drawable>(
    img: &MarsImage,
    map: &mut D,
    anaglyph: bool,
    quat: &Quaternion,
    out_model: &CameraModel,
    border: usize,
) {
    let eye = Eye::DontCare; /*if anaglyph {
                                 match util::filename_char_at_pos(&img.file_path.unwrap_or("NL".to_string()), 1) {
                                     'R' => Eye::Right,
                                     'L' => Eye::Left,
                                     _ => Eye::DontCare,
                                 }
                             } else {
                                 Eye::DontCare
                             };*/

    if let Some(input_model) = get_cahvor(&img) {
        vprintln!("");
        vprintln!("Input Model C: {:?}", input_model.c());
        vprintln!("Input Model A: {:?}", input_model.a());
        vprintln!("Input Model H: {:?}", input_model.h());
        vprintln!("Input Model V: {:?}", input_model.v());
        vprintln!("Input Model O: {:?}", input_model.o());
        vprintln!("Input Model R: {:?}", input_model.r());
        vprintln!("Input Model E: {:?}", input_model.e());
        vprintln!("");

        let band_0 = img.image.get_band(0);
        let band_1 = img.image.get_band(1);
        let band_2 = img.image.get_band(2);

        for x in border..(img.image.width - border) {
            for y in border..(img.image.height - border) {
                let (tl_y, tl_x) = if let Ok((x, y)) = project_ls(&input_model, &out_model, x, y) {
                    (x, y)
                } else {
                    break;
                };

                let (tr_y, tr_x) =
                    if let Ok((x, y)) = project_ls(&input_model, &out_model, x + 1, y) {
                        (x, y)
                    } else {
                        break;
                    };
                let (bl_y, bl_x) =
                    if let Ok((x, y)) = project_ls(&input_model, &out_model, x, y + 1) {
                        (x, y)
                    } else {
                        break;
                    };
                let (br_y, br_x) =
                    if let Ok((x, y)) = project_ls(&input_model, &out_model, x + 1, y + 1) {
                        (x, y)
                    } else {
                        break;
                    };

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

                map.paint_square_with_channel_rule(&tl, &bl, &br, &tr, true, |c| {
                    (c == 0 && matches!(eye, Eye::Left | Eye::DontCare))
                        || ((c == 1 || c == 2) && matches!(eye, Eye::Right | Eye::DontCare))
                });
            }
        }
    } else {
        eprintln!("CAHVOR not found for image, cannot continue");
        panic!("CAHVOR not found for image, cannot continue");
    }
}

pub fn process_files<D: Drawable>(
    input_images: &[MarsImage],
    map: &mut D,
    anaglyph: bool,
    quat: &Quaternion,
    out_model: &CameraModel,
    border: usize,
) {
    input_images
        .iter()
        .for_each(|img| process_file(img, map, anaglyph, quat, out_model, border));
}

pub fn ___process_files<D: Drawable>(
    input_images: &[MarsImage],
    map: &mut D,
    anaglyph: bool,
    quat: &Quaternion,
    out_model: &CameraModel,
    border: usize,
) {
    // Construct out_model

    let out_model = out_model.linearize(2556, 1916, 360 * 4, 180 * 4).unwrap();

    for (y, x) in iproduct!(0..map.get_height() - 1, 0..map.get_width() - 1) {
        let proj_cs = out_model
            .ls_to_look_vector(&ImageCoordinate {
                line: y as f64,
                sample: x as f64,
            })
            .unwrap();
        let (intersection_type, surf_pt) = intersect_ray(&proj_cs);

        input_images.iter().for_each(|img| {
            let in_model = get_cahvor(&img).unwrap();

            let ls = if intersection_type != PlaneIntersectionDirection::Normal {
                // vprintln!("INFINITY! {}, {}", x, y);
                let uvec3 = proj_cs.look_direction.clone(); // If infinity
                let p = if in_model.a().dot_product(&uvec3) >= 0.0 {
                    uvec3.add(&in_model.c())
                } else {
                    uvec3.inversed().add(&in_model.c())
                };
                in_model.xyz_to_ls(&p, true)
            } else {
                in_model.xyz_to_ls(&surf_pt, false)
            };

            if let Ok(ls) = ls {
                // println!(
                //     "Hey, It's ok! x = {}, y = {}, lat = {}, lon = {}",
                //     x, y, lat, lon
                // );

                if (ls.sample as usize) < img.image.width && (ls.line as usize) < img.image.height {
                    let r = img
                        .image
                        .get_band(0)
                        .get(ls.sample as usize, ls.line as usize);
                    let g = img
                        .image
                        .get_band(1)
                        .get(ls.sample as usize, ls.line as usize);
                    let b = img
                        .image
                        .get_band(2)
                        .get(ls.sample as usize, ls.line as usize);

                    let tl = Point::create_rgb(x as f64, y as f64, r as f64, g as f64, b as f64);
                    let tr =
                        Point::create_rgb(x as f64 + 1.0, y as f64, r as f64, g as f64, b as f64);
                    let bl =
                        Point::create_rgb(x as f64, y as f64 + 1.0, r as f64, g as f64, b as f64);
                    let br = Point::create_rgb(
                        x as f64 + 1.0,
                        y as f64 + 1.0,
                        r as f64,
                        g as f64,
                        b as f64,
                    );

                    map.paint_square_with_channel_rule(&tl, &bl, &br, &tr, true, |c| true);
                } else {
                    // println!("{:?}", ls);
                }
            }
        });
    }
}
