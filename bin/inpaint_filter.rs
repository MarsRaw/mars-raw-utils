use mars_raw_utils::{
    constants, 
    print, 
    vprintln, 
    path,
    util
};

use sciimg::{
    rgbimage,
    inpaint
};

#[macro_use]
extern crate clap;

use clap::{Arg, App};

use std::process;

fn process_file(input_file:&str) {

    let raw = rgbimage::RgbImage::open(&String::from(input_file)).unwrap();
    
    vprintln!("Generating mask from red pixels...");
    let mask = inpaint::make_mask_from_red(&raw).unwrap();
    //mask.save("/data/MSL/inpaint_test/test-mask.png", enums::ImageMode::U8BIT).unwrap();

    vprintln!("Inpainting based on generated mask...");
    let filled = match inpaint::apply_inpaint_to_buffer_with_mask(&raw, &mask) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error in inpainting process: {}", e);
            process::exit(1);
        }
    };

    let out_file = util::append_file_name(input_file, "inpaint");

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

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

    
}
