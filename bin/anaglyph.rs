
use mars_raw_utils::prelude::*;
use sciimg::{
    prelude::*,
    vector::Vector
};

#[macro_use]
extern crate clap;
use clap::{Arg, App};
use std::process;

fn intersect_to_plane(lv:&LookVector, ground:&Vector) -> Vector {
    let normal = Vector::new(0.0, 0.0, -1.0);
    

    let dot = lv.look_direction.dot_product(&normal);
    if dot == 0.0 {
        return lv.look_direction.clone();
    }

    let ratio = ground.subtract(&lv.origin).dot_product(&normal) / dot;

    let intersect_point = lv.origin.add(&lv.look_direction.scale(ratio));
    
    if ratio < 0.0 {
        lv.look_direction.clone()
    } else {
        intersect_point
    }

}

fn process_image(img:&MarsImage, map:&mut RgbImage, input_model:&CameraModel, output_model:&Cahv, ground:&Vector, eye:Eye) {
    for x in 0..map.width {
        for y in 0..map.height {

            if let Ok(lv) = output_model.ls_to_look_vector(&ImageCoordinate{line: y as f64, sample: x as f64}) {

                let ray = intersect_to_plane(&lv, &ground);

                let diff = input_model.c().subtract(&output_model.c());
                let ray_moved = ray.subtract(&diff);

                let ls_in = input_model.xyz_to_ls(&ray_moved, false);

                let in_x = ls_in.sample.round() as usize;
                let in_y = ls_in.line.round() as usize;

                if in_x < img.image.width - 1 && in_y < img.image.height - 1 {
                    let tl = Point::create(
                        x as f64,
                        y as f64,
                        img.image.get_band(0).get(in_x, in_y).unwrap() as f64,
                        img.image.get_band(1).get(in_x, in_y).unwrap() as f64,
                        img.image.get_band(2).get(in_x, in_y).unwrap() as f64
                    );

                    let bl = Point::create(
                        x as f64,
                        (y + 1) as f64,
                        img.image.get_band(0).get(in_x, in_y).unwrap() as f64,
                        img.image.get_band(1).get(in_x, in_y).unwrap() as f64,
                        img.image.get_band(2).get(in_x, in_y).unwrap() as f64
                    );

                    let tr = Point::create(
                        (x + 1) as f64,
                        y as f64,
                        img.image.get_band(0).get(in_x, in_y).unwrap() as f64,
                        img.image.get_band(1).get(in_x, in_y).unwrap() as f64,
                        img.image.get_band(2).get(in_x, in_y).unwrap() as f64
                    );
                    
                    let br = Point::create(
                        (x + 1) as f64,
                        (y + 1) as f64,
                        img.image.get_band(0).get(in_x, in_y).unwrap() as f64,
                        img.image.get_band(1).get(in_x, in_y).unwrap() as f64,
                        img.image.get_band(2).get(in_x, in_y).unwrap() as f64
                    );


                    map.paint_square(&tl, &bl, &br, &tr, false, eye);
                }
            }
        }
    }
}

fn main() {
    let matches = App::new(crate_name!())
                    .version(crate_version!())
                    .author(crate_authors!())
                    .arg(Arg::with_name("left")
                        .short("l")
                        .long("left")
                        .value_name("left")
                        .help("Left eye image")
                        .required(true)
                        .multiple(false)
                        .takes_value(true))
                    .arg(Arg::with_name("right")
                        .short("r")
                        .long("right")
                        .value_name("right")
                        .help("Right eye image")
                        .required(true)
                        .multiple(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
                        .short(constants::param::PARAM_OUTPUT_SHORT)
                        .long(constants::param::PARAM_OUTPUT)
                        .value_name("OUTPUT")
                        .help("Output")
                        .required(true)
                        .takes_value(true)) 
                    .arg(Arg::with_name("mono")
                        .short("m")
                        .long("mono")
                        .value_name("MONO")
                        .help("Monochrome color (before converting to red/blue)")
                        .required(false)
                        .takes_value(false)) 
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let left_eye_image_path = matches.value_of("left").unwrap();
    let right_eye_image_path = matches.value_of("right").unwrap();
    let output = matches.value_of("output").unwrap();

    let mut left_img = MarsImage::open(String::from(left_eye_image_path), Instrument::M20MastcamZLeft);
    let mut right_img = MarsImage::open(String::from(right_eye_image_path), Instrument::M20MastcamZRight);

    if matches.is_present("mono") {
        vprintln!("Converting input images to monochrome...");
        left_img.to_mono();
        right_img.to_mono();
    }

    let mut map = RgbImage::create(left_img.image.width, left_img.image.height);

    let left_cahv = if let Some(left_md) = &left_img.metadata {
        if left_md.camera_model_component_list.is_valid() {
            left_md.camera_model_component_list.clone()
        } else {
            process::exit(2);
        }
    } else {
        process::exit(1);
    };

    let right_cahv = if let Some(right_md) = &right_img.metadata {
        if right_md.camera_model_component_list.is_valid() {
            right_md.camera_model_component_list.clone()
        } else {
            process::exit(2);
        }
    } else {
        process::exit(1);
    };

    let ground = Vector::new(0.0, 0.0, 1.84566);


    let output_model = left_cahv.linearize(left_img.image.width, left_img.image.height, left_img.image.width, left_img.image.height).unwrap();

    process_image(&right_img, &mut map, &right_cahv, &output_model, &ground, Eye::Right);
    process_image(&left_img, &mut map, &left_cahv, &output_model, &ground, Eye::Left);

    map.normalize_to_16bit_with_max(255.0);
    map.save(output);
}