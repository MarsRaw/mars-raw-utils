use mars_raw_utils::prelude::*;

#[macro_use]
extern crate clap;

use std::process;

use clap::{Arg, App};

fn main() {
    init_panic_handler();
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
                    .arg(Arg::with_name(constants::param::PARAM_ONLY_NEW)
                        .short(constants::param::PARAM_ONLY_NEW_SHORT)
                        .help("Only new images. Skipped processed images."))
                    .arg(Arg::with_name(constants::param::PARAM_RAW_COLOR)
                        .short(constants::param::PARAM_RAW_COLOR_SHORT)
                        .long(constants::param::PARAM_RAW_COLOR)
                        .help("Raw color, skip ILT"))
                    .arg(Arg::with_name(constants::param::PARAM_CAL_PROFILE)
                        .short(constants::param::PARAM_CAL_PROFILE_SHORT)
                        .long(constants::param::PARAM_CAL_PROFILE)
                        .value_name("PARAM_CAL_PROFILE")
                        .help("Calibration profile file path")
                        .required(false)
                        .multiple(true)
                        .takes_value(true)) 
                    .get_matches_from(wild::args());

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let mut red_scalar = constants::DEFAULT_RED_WEIGHT;
    let mut green_scalar = constants::DEFAULT_GREEN_WEIGHT;
    let mut blue_scalar = constants::DEFAULT_BLUE_WEIGHT;
    let mut no_ilt = false;
    let filename_suffix: String = String::from(constants::OUTPUT_FILENAME_APPEND);
    
    let mut only_new = false;
    if matches.is_present(constants::param::PARAM_ONLY_NEW) {
        only_new = true;
    }

    if matches.is_present(constants::param::PARAM_RAW_COLOR) {
        no_ilt = true;
    }

    // Check formatting and handle it
    if matches.is_present(constants::param::PARAM_RED_WEIGHT) {
        let s = matches.value_of(constants::param::PARAM_RED_WEIGHT).unwrap();
        if util::string_is_valid_f32(&s) {
            red_scalar = s.parse::<f32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified for red scalar");
            process::exit(1);
        }
    }

    if matches.is_present(constants::param::PARAM_GREEN_WEIGHT) {
        let s = matches.value_of(constants::param::PARAM_GREEN_WEIGHT).unwrap();
        if util::string_is_valid_f32(&s) {
            green_scalar = s.parse::<f32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified for green scalar");
            process::exit(1);
        }
    }

    if matches.is_present(constants::param::PARAM_BLUE_WEIGHT) {
        let s = matches.value_of(constants::param::PARAM_BLUE_WEIGHT).unwrap();
        if util::string_is_valid_f32(&s) {
            blue_scalar = s.parse::<f32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified for blue scalar");
            process::exit(1);
        }
    }

    let profiles: Vec<&str> = match matches.values_of(constants::param::PARAM_CAL_PROFILE) {
        Some(profiles) => profiles.collect(),
        None => vec!()
    };
    
    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    let calibrator = nsyt::idc::NsytIdc{};
    if profiles.len() > 0 {
        simple_calibration_with_profiles(&calibrator, &input_files, only_new, &profiles);
    } else {
        simple_calibration(&calibrator, &input_files, only_new, &CalProfile{
            apply_ilt: !no_ilt,
            red_scalar: red_scalar,
            green_scalar: green_scalar,
            blue_scalar: blue_scalar,
            color_noise_reduction: false,
            color_noise_reduction_amount: 0,
            hot_pixel_detection_threshold: 0.0,
            hot_pixel_window_size: 0,
            filename_suffix: filename_suffix
        });
    }

}