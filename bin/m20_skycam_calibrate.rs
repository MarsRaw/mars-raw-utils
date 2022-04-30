
use mars_raw_utils::prelude::*;

#[macro_use]
extern crate clap;
use clap::{Arg, App};

use std::process;

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
                    .arg(Arg::with_name(constants::param::PARAM_HPC_THRESHOLD)
                        .short(constants::param::PARAM_HPC_THRESHOLD_SHORT)
                        .long(constants::param::PARAM_HPC_THRESHOLD)
                        .value_name("THRESHOLD")
                        .help("Hot pixel correction variance threshold")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_HPC_WINDOW_SIZE)
                        .short(constants::param::PARAM_HPC_WINDOW_SIZE_SHORT)
                        .long(constants::param::PARAM_HPC_WINDOW_SIZE)
                        .value_name("WINDOW_SIZE")
                        .help("Hot pixel correction window size")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .arg(Arg::with_name(constants::param::PARAM_ONLY_NEW)
                        .short(constants::param::PARAM_ONLY_NEW_SHORT)
                        .help("Only new images. Skipped processed images."))
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

    let filename_suffix: String = String::from(constants::OUTPUT_FILENAME_APPEND);

    let mut only_new = false;
    if matches.is_present(constants::param::PARAM_ONLY_NEW) {
        only_new = true;
    }

    let mut hpc_threshold = 0.0;
    let mut hpc_window_size = 10;

    if matches.is_present(constants::param::PARAM_HPC_THRESHOLD) {
        let s = matches.value_of(constants::param::PARAM_HPC_THRESHOLD).unwrap();
        if util::string_is_valid_f32(&s) {
            hpc_threshold = s.parse::<f32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified for HPC variance threshold");
            process::exit(1);
        }
    }
    
    if matches.is_present(constants::param::PARAM_HPC_WINDOW_SIZE) {
        let s = matches.value_of(constants::param::PARAM_HPC_WINDOW_SIZE).unwrap();
        if util::string_is_valid_i32(&s) {
            hpc_window_size = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified for HPC window size");
            process::exit(1);
        }
    }

    let profiles: Vec<&str> = match matches.values_of(constants::param::PARAM_CAL_PROFILE) {
        Some(profiles) => profiles.collect(),
        None => vec!()
    };

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    let calibrator = m20::skycam::M20SkyCam{};
    if profiles.len() > 0 {
        simple_calibration_with_profiles(&calibrator, &input_files, only_new, &profiles);
    } else {
        simple_calibration(&calibrator, &input_files, only_new, &CalProfile{
            apply_ilt: false,
            red_scalar: 1.0,
            green_scalar: 1.0,
            blue_scalar: 1.0,
            color_noise_reduction: false,
            color_noise_reduction_amount: 0,
            hot_pixel_detection_threshold: hpc_threshold,
            hot_pixel_window_size: hpc_window_size,
            filename_suffix: filename_suffix
        });
    }
}
