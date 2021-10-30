/*
    Imaging upscale experiment using inpainting
*/
use mars_raw_utils::{
    constants, 
    print, 
    vprintln,
    path,
    util
};

use sciimg::{
    rgbimage,
    inpaint,
    imagebuffer,
    enums::ImageMode
};

#[macro_use]
extern crate clap;

use clap::{Arg, App};

use std::process;


fn process_file(input_file:&str, scale_factor:usize) {

    let raw = rgbimage::RgbImage::open(&String::from(input_file)).unwrap();

    let mut upscaled = rgbimage::RgbImage::new(raw.width * scale_factor, raw.height * scale_factor, ImageMode::U8BIT).unwrap();
    let mut fill_mask = imagebuffer::ImageBuffer::new(raw.width * scale_factor, raw.height * scale_factor).unwrap();

    for y in 0..(raw.height * scale_factor) {
        for x in 0..(raw.width * scale_factor) {
            if y % scale_factor == 0 && x % scale_factor == 0 {
                let r = raw.get_band(0).get(x / scale_factor, y / scale_factor).unwrap();
                let g = raw.get_band(1).get(x / scale_factor, y / scale_factor).unwrap();
                let b = raw.get_band(2).get(x / scale_factor, y / scale_factor).unwrap();
                upscaled.put(x, y, r, 0);
                upscaled.put(x, y, g, 1);
                upscaled.put(x, y, b, 2);
            } else {
                fill_mask.put(x, y, 255.0);
            }
        }
    }

    vprintln!("Inpainting based on generated mask...");
    let filled = match inpaint::apply_inpaint_to_buffer_with_mask(&upscaled, &fill_mask) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error in inpainting process: {}", e);
            process::exit(1);
        }
    };

    let out_file = util::append_file_name(input_file, "upscale");

    vprintln!("Saving output to {}", out_file);
    filled.save(&out_file);
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
                    .arg(Arg::with_name(constants::param::PARAM_SCALE_FACTOR)
                        .short(constants::param::PARAM_SCALE_FACTOR_SHORT)
                        .long(constants::param::PARAM_SCALE_FACTOR)
                        .value_name("FACTOR")
                        .help("Scale factor")
                        .required(true)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    // If, for some weird reason, clap misses the missing parameter...
    if ! matches.is_present(constants::param::PARAM_INPUTS) {
        println!("{}", matches.usage());
    }

    let mut scale_factor = 2;
    if matches.is_present(constants::param::PARAM_SCALE_FACTOR) {
        let s = matches.value_of(constants::param::PARAM_SCALE_FACTOR).unwrap();
        if util::string_is_valid_i32(&s) {
            scale_factor = s.parse::<i32>().unwrap();
            vprintln!("Applying scale factor of {}x", scale_factor);
        } else {
            eprintln!("Error: Invalid number specified for scale factor");
            process::exit(1);
        }
    }


    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file, scale_factor as usize);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

    
}
