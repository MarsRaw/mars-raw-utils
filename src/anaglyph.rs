use crate::prelude::*;
use sciimg::{drawable::*, prelude::*, vector::Vector};

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
pub fn anaglyph_extent(
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
                if let Some(p) = project_to_output(model, out_model, ground, s, l) {
                    if p.sample.is_finite() && p.line.is_finite() {
                        min_x = min_x.min(p.sample);
                        max_x = max_x.max(p.sample);
                        min_y = min_y.min(p.line);
                        max_y = max_y.max(p.line);
                    }
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

pub fn process_image(
    img: &MarsImage,
    map: &mut Image,
    input_model: &Cahvore,
    output_model: &Cahvore,
    ground: &Vector,
    eye: Eye,
) {
    for x in 0..map.width {
        for y in 0..map.height {
            if let Ok(lv) = output_model.ls_to_look_vector(&ImageCoordinate {
                line: y as f64,
                sample: x as f64,
            }) {
                let ray = lv.intersect_to_plane(ground);

                let ls_in_res = match ray {
                    Some(ray) => {
                        //let diff = input_model.c().subtract(&output_model.c());
                        //let ray_moved = ray.subtract(&diff);
                        input_model.xyz_to_ls(&ray, false)
                    }
                    None => input_model.xyz_to_ls(&lv.look_direction, true),
                };

                if let Ok(ls_in) = ls_in_res {
                    let in_x = ls_in.sample.round() as usize;
                    let in_y = ls_in.line.round() as usize;

                    if ls_in.sample >= 0.0
                        && ls_in.line >= 0.0
                        && in_x < img.image.width - 1
                        && in_y < img.image.height - 1
                    {
                        let tl = Point::create_rgb(
                            x as f64,
                            y as f64,
                            img.image.get_band(0).get(in_x, in_y) as f64,
                            img.image.get_band(1).get(in_x, in_y) as f64,
                            img.image.get_band(2).get(in_x, in_y) as f64,
                        );

                        let bl = Point::create_rgb(
                            x as f64,
                            (y + 1) as f64,
                            img.image.get_band(0).get(in_x, in_y) as f64,
                            img.image.get_band(1).get(in_x, in_y) as f64,
                            img.image.get_band(2).get(in_x, in_y) as f64,
                        );

                        let tr = Point::create_rgb(
                            (x + 1) as f64,
                            y as f64,
                            img.image.get_band(0).get(in_x, in_y) as f64,
                            img.image.get_band(1).get(in_x, in_y) as f64,
                            img.image.get_band(2).get(in_x, in_y) as f64,
                        );

                        let br = Point::create_rgb(
                            (x + 1) as f64,
                            (y + 1) as f64,
                            img.image.get_band(0).get(in_x, in_y) as f64,
                            img.image.get_band(1).get(in_x, in_y) as f64,
                            img.image.get_band(2).get(in_x, in_y) as f64,
                        );

                        map.paint_square_with_channel_rule(&tl, &bl, &br, &tr, false, |c| {
                            (c == 0 && matches!(eye, Eye::Left | Eye::DontCare))
                                || ((c == 1 || c == 2) && matches!(eye, Eye::Right | Eye::DontCare))
                        });
                    }
                }
            }
        }
    }
}
