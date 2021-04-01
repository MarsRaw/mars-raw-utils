use mars_raw_utils::{constants, print, vprintln, rgbimage, enums, path};

#[macro_use]
extern crate clap;

use clap::{Arg, App};




fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32) {
    let mut raw = rgbimage::RgbImage::open(input_file).unwrap();

    if raw.width == 1632 && raw.height == 1200 {
        vprintln!("Cropping...");
        raw.crop(32, 16, 1584, 1184).unwrap();
    }
    
    vprintln!("Inpainting...");
    raw.apply_inpaint_fix(enums::Instrument::MslMAHLI).unwrap();

    vprintln!("Decompanding...");
    raw.decompand(enums::Instrument::MslMAHLI).unwrap();

    vprintln!("Flatfielding...");
    raw.flatfield(enums::Instrument::MslMAHLI).unwrap();

    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar).unwrap();

    vprintln!("Normalizing...");
    raw.normalize_to_16bit_with_max(2033.0).unwrap();

    vprintln!("Writing to disk...");

    let out_file = input_file.replace(".jpg", "-rjcal.png").replace(".JPG", "-rjcal.png");
    raw.save(&out_file).unwrap();
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
                    .arg(Arg::with_name(constants::param::PARAM_RED_WEIGHT)
                        .short(constants::param::PARAM_RED_WEIGHT_SHORT)
                        .long(constants::param::PARAM_RED_WEIGHT)
                        .value_name("RED")
                        .help("Red weight")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_GREEN_WEIGHT)
                        .short(constants::param::PARAM_GREEN_WEIGHT_SHORT)
                        .long(constants::param::PARAM_GREEN_WEIGHT)
                        .value_name("GREEN")
                        .help("Green weight")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_BLUE_WEIGHT)
                        .short(constants::param::PARAM_BLUE_WEIGHT_SHORT)
                        .long(constants::param::PARAM_BLUE_WEIGHT)
                        .value_name("BLUE")
                        .help("Blue weight")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let mut red_scalar = constants::DEFAULT_RED_WEIGHT;
    let mut green_scalar = constants::DEFAULT_GREEN_WEIGHT;
    let mut blue_scalar = constants::DEFAULT_BLUE_WEIGHT;

    // Check formatting and handle it
    if matches.is_present(constants::param::PARAM_RED_WEIGHT) {
        let s = matches.value_of(constants::param::PARAM_RED_WEIGHT).unwrap();
        red_scalar = s.parse::<f32>().unwrap();
    }

    if matches.is_present(constants::param::PARAM_GREEN_WEIGHT) {
        let s = matches.value_of(constants::param::PARAM_GREEN_WEIGHT).unwrap();
        green_scalar = s.parse::<f32>().unwrap();
    }

    if matches.is_present(constants::param::PARAM_BLUE_WEIGHT) {
        let s = matches.value_of(constants::param::PARAM_BLUE_WEIGHT).unwrap();
        blue_scalar = s.parse::<f32>().unwrap();
    }

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file, red_scalar, green_scalar, blue_scalar);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

    
}
