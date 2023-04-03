use crate::prelude::*;
use sciimg::{prelude::*, vector::Vector};

pub fn process_image(
    img: &MarsImage,
    map: &mut Image,
    input_model: &CameraModel,
    output_model: &CameraModel,
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

                let ls_in = match ray {
                    Some(ray) => {
                        let diff = input_model.c().subtract(&output_model.c());
                        let ray_moved = ray.subtract(&diff);
                        input_model.xyz_to_ls(&ray_moved, false)
                    }
                    None => input_model.xyz_to_ls(&lv.look_direction, true),
                };

                let in_x = ls_in.sample.round() as usize;
                let in_y = ls_in.line.round() as usize;

                if ls_in.sample >= 0.0
                    && ls_in.line >= 0.0
                    && in_x < img.image.width - 1
                    && in_y < img.image.height - 1
                {
                    let tl = Point::create_rgb(
                        x as f32,
                        y as f32,
                        img.image.get_band(0).get(in_x, in_y).unwrap(),
                        img.image.get_band(1).get(in_x, in_y).unwrap(),
                        img.image.get_band(2).get(in_x, in_y).unwrap(),
                    );

                    let bl = Point::create_rgb(
                        x as f32,
                        (y + 1) as f32,
                        img.image.get_band(0).get(in_x, in_y).unwrap(),
                        img.image.get_band(1).get(in_x, in_y).unwrap(),
                        img.image.get_band(2).get(in_x, in_y).unwrap(),
                    );

                    let tr = Point::create_rgb(
                        (x + 1) as f32,
                        y as f32,
                        img.image.get_band(0).get(in_x, in_y).unwrap(),
                        img.image.get_band(1).get(in_x, in_y).unwrap(),
                        img.image.get_band(2).get(in_x, in_y).unwrap(),
                    );

                    let br = Point::create_rgb(
                        (x + 1) as f32,
                        (y + 1) as f32,
                        img.image.get_band(0).get(in_x, in_y).unwrap(),
                        img.image.get_band(1).get(in_x, in_y).unwrap(),
                        img.image.get_band(2).get(in_x, in_y).unwrap(),
                    );

                    map.paint_square(&tl, &bl, &br, &tr, false, eye);
                }
            }
        }
    }
}
