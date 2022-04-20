use mars_raw_utils::prelude::*;

use sciimg::prelude::*;

#[macro_use]
extern crate clap;

use clap::{Arg, App};

use std::process;

fn process_file(input_file:&str, color_noise_reduction:i32) {
    let mut raw = RgbImage::open(&String::from(input_file)).unwrap();

    let out_file = util::append_file_name(input_file, "debayer");

    if !raw.is_grayscale() {
        vprintln!("WARNING: Image doesn't appear to be grayscale as would be expected.");
        vprintln!("Results may be inaccurate");
    }

    vprintln!("Debayering image...");
    raw.debayer();

    if color_noise_reduction > 0 {
        vprintln!("Color noise reduction...");
        raw.reduce_color_noise(color_noise_reduction);
    }

    vprintln!("Writing to disk...");
    raw.save(&out_file);
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
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .arg(Arg::with_name(constants::param::PARAM_COLOR_NOISE_REDUCTION)
                        .short(constants::param::PARAM_COLOR_NOISE_REDUCTION_SHORT)
                        .long(constants::param::PARAM_COLOR_NOISE_REDUCTION)
                        .value_name("COLOR_NOISE_REDUCTION")
                        .help("Color noise reduction amount in pixels")
                        .required(false)
                        .takes_value(true))
                    .get_matches_from(wild::args());

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    // If, for some weird reason, clap misses the missing parameter...
    if ! matches.is_present(constants::param::PARAM_INPUTS) {
        println!("{}", matches.usage());
        process::exit(1);
    }

    let mut color_noise_reduction = 0;
    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    if matches.is_present(constants::param::PARAM_COLOR_NOISE_REDUCTION) {
        let s = matches.value_of(constants::param::PARAM_COLOR_NOISE_REDUCTION).unwrap();
        if util::string_is_valid_i32(&s) {
            color_noise_reduction = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified for color noise reduction");
            process::exit(1);
        }
        if color_noise_reduction % 2 == 0 {
            eprintln!("Error: Color noise reduction value must be odd");
            process::exit(1);
        }
        if color_noise_reduction < 0 {
            eprintln!("Error: Color noise reduction value must a positive number");
            process::exit(1);
        }
    }

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file, color_noise_reduction);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }
}