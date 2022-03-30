use mars_raw_utils::{
    constants, 
    print, 
    quality,
    path,
    vprintln,
    util
};

use sciimg::{
    rgbimage,
    imagebuffer,
    lowpass,
    stats
};

#[macro_use]
extern crate clap;

use clap::{Arg, App};
use std::process;


struct Diff {
    band_0:imagebuffer::ImageBuffer,
    band_1:imagebuffer::ImageBuffer,
    band_2:imagebuffer::ImageBuffer,
    image:rgbimage::RgbImage
}


fn make_diff_for_band(buffer:&imagebuffer::ImageBuffer, amount:usize) -> imagebuffer::ImageBuffer {
    let blurred = lowpass::lowpass_imagebuffer(&buffer, amount);
    blurred.subtract(&buffer).unwrap()
}

fn make_diff_container(image:&rgbimage::RgbImage, blur_amount:usize) -> Diff {
    Diff{
        band_0:make_diff_for_band(image.get_band(0), blur_amount),
        band_1:make_diff_for_band(image.get_band(1), blur_amount),
        band_2:make_diff_for_band(image.get_band(2), blur_amount),
        image:image.clone()
    }
}

fn main() {
    
    let matches = App::new(crate_name!())
                    .version(crate_version!())
                    .author(crate_authors!())
                    .arg(Arg::with_name(constants::param::PARAM_INPUTS)
                        .short(constants::param::PARAM_INPUTS_SHORT)
                        .long(constants::param::PARAM_INPUTS)
                        .value_name("INPUT")
                        .help("Input")
                        .required(true)
                        .multiple(true)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
                        .short(constants::param::PARAM_OUTPUT_SHORT)
                        .long(constants::param::PARAM_OUTPUT)
                        .value_name("OUTPUT")
                        .help("Output")
                        .required(true)
                        .takes_value(true)) 
                    .arg(Arg::with_name(constants::param::PARAM_QUALITY_WINDOW_SIZE)
                        .short(constants::param::PARAM_QUALITY_WINDOW_SIZE_SHORT)
                        .long(constants::param::PARAM_QUALITY_WINDOW_SIZE)
                        .value_name("QUALITY_WINDOW_SIZE")
                        .help("Quality determination window size (pixels)")
                        .required(false)
                        .takes_value(true)) 
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let output = matches.value_of("output").unwrap();

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();
    if input_files.len() == 0 {
        eprintln!("No files were specified. Please do so.");
        process::exit(1);
    }

    let quality_window_size : usize = match matches.is_present(constants::param::PARAM_QUALITY_WINDOW_SIZE) {
        true => {
            let s = matches.value_of(constants::param::PARAM_QUALITY_WINDOW_SIZE).unwrap();
            if util::string_is_valid_u16(&s) {
                s.parse::<usize>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for quality determination window size");
                process::exit(1);
            }
        },
        false => {
            15
        }
    };


    let mut images : Vec<Diff> = vec!();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);

            let image = rgbimage::RgbImage::open16(&String::from(*in_file)).unwrap();
            images.push(make_diff_container(&image, 5));
        } else {
            eprintln!("File not found: {}", in_file);
            process::exit(1);
        }
    }

    let mut b0_merge_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(images[0].band_0.width, images[0].band_0.height, 0.0, images[0].band_0.mode).unwrap();
    let mut b1_merge_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(images[0].band_0.width, images[0].band_0.height, 0.0, images[0].band_0.mode).unwrap();
    let mut b2_merge_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(images[0].band_0.width, images[0].band_0.height, 0.0, images[0].band_0.mode).unwrap();

    // Super mega inefficient. This'll take a few minutes to run.
    for y in 0..b0_merge_buffer.height {
        vprintln!("Row {} of {}. {}%", y, b0_merge_buffer.height, (y as f32 / b0_merge_buffer.height as f32 * 100.0));
        for x in 0..b0_merge_buffer.width {
            let mut b0_value = 0.0_f32;
            let mut b1_value = 0.0_f32;
            let mut b2_value = 0.0_f32;
            let mut max_quality = 0.0_f32;

            for image in images.iter() {
                let q0 = quality::get_point_quality_estimation_on_diff_buffer(&image.band_0, quality_window_size, x, y);
                let q1 = quality::get_point_quality_estimation_on_diff_buffer(&image.band_1, quality_window_size, x, y);
                let q2 = quality::get_point_quality_estimation_on_diff_buffer(&image.band_2, quality_window_size, x, y);
                let q = match stats::mean(&vec![q0, q1, q2]) {
                    Some(m) => m,
                    None => 0.0
                };

                if q > max_quality {
                    max_quality = q;
                    b0_value = image.image.get_band(0).get(x, y).unwrap();
                    b1_value = image.image.get_band(1).get(x, y).unwrap();
                    b2_value = image.image.get_band(2).get(x, y).unwrap();
                }
            }

            b0_merge_buffer.put(x, y, b0_value);
            b1_merge_buffer.put(x, y, b1_value);
            b2_merge_buffer.put(x, y, b2_value);
        }
    }

    let merge_buffer = rgbimage::RgbImage::new_from_buffers_rgb(&b0_merge_buffer, &b1_merge_buffer, &b2_merge_buffer, b0_merge_buffer.mode).unwrap();

    merge_buffer.save(&output);
}