use mars_raw_utils::{
    constants, 
    print, 
    quality,
    path,
    vprintln
};

use sciimg::{
    rgbimage,
    imagebuffer
};

#[macro_use]
extern crate clap;

use clap::{Arg, App};
use std::process;


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


    let mut images : Vec<rgbimage::RgbImage> = vec!();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            images.push(rgbimage::RgbImage::open16(&String::from(*in_file)).unwrap());
        } else {
            eprintln!("File not found: {}", in_file);
            process::exit(1);
        }
    }

    let mut b0_merge_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(images[0].width, images[0].height, 0.0, images[0].get_mode()).unwrap();
    let mut b1_merge_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(images[0].width, images[0].height, 0.0, images[0].get_mode()).unwrap();
    let mut b2_merge_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(images[0].width, images[0].height, 0.0, images[0].get_mode()).unwrap();

    // Super mega inefficient. This'll take a few minutes to run.
    for y in 0..b0_merge_buffer.height {
        for x in 0..b0_merge_buffer.width {
            let mut b0_value = 0.0_f32;
            let mut b1_value = 0.0_f32;
            let mut b2_value = 0.0_f32;
            let mut max_quality = 0.0_f32;

            for image in images.iter() {
                let q = quality::get_point_quality_estimation(image, 5, x, y);
                if q > max_quality {
                    max_quality = q;
                    b0_value = image.get_band(0).get(x, y).unwrap();
                    b1_value = image.get_band(1).get(x, y).unwrap();
                    b2_value = image.get_band(2).get(x, y).unwrap();
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