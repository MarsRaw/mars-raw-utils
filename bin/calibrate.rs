use mars_raw_utils::prelude::*;

#[macro_use]
extern crate clap;

use rayon::prelude::*;

use std::process;

use clap::{Arg, App};

use std::panic;

fn get_calibrator_for_file(input_file:&str, default_instrument:Option<&str>) -> Option<&'static CalContainer>  {
    let metadata_file = util::replace_image_extension(&input_file, "-metadata.json");
    vprintln!("Checking for metadata file at {}", metadata_file);
    if path::file_exists(metadata_file.as_str()) {
        vprintln!("Metadata file exists for loaded image: {}", metadata_file);
        match metadata::load_image_metadata(&metadata_file) {
            Err(_) => None, // Error loading the metadata file
            Ok(md) => {
                calibrator_for_instrument_from_str(&md.instrument.as_str())
            }
        }
    } else { // metadata file is missing

        // If a default instrument was passed in, try and use that
        if let Some(instrument) = default_instrument {
            calibrator_for_instrument_from_str(instrument)
        } else {
            None // Otherwise, we don't know the instrument.
        }
    }
}


fn main() {
    // init_panic_handler();
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
                    .arg(Arg::with_name(constants::param::PARAM_INSTRUMENT)
                        .short(constants::param::PARAM_INSTRUMENT_SHORT)
                        .long(constants::param::PARAM_INSTRUMENT)
                        .value_name("INSTRUMENT")
                        .help("Default instrument (if missing)")
                        .required(false)
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
                    .arg(Arg::with_name(constants::param::PARAM_COLOR_NOISE_REDUCTION)
                        .short(constants::param::PARAM_COLOR_NOISE_REDUCTION_SHORT)
                        .long(constants::param::PARAM_COLOR_NOISE_REDUCTION)
                        .value_name("COLOR_NOISE_REDUCTION")
                        .help("Color noise reduction amount in pixels")
                        .required(false)
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

    let cal_context = CalProfile{
        apply_ilt: !matches.is_present(constants::param::PARAM_RAW_COLOR),
        red_scalar: match matches.is_present(constants::param::PARAM_RED_WEIGHT) {
            true => {
                let s = matches.value_of(constants::param::PARAM_RED_WEIGHT).unwrap();
                if util::string_is_valid_f32(&s) {
                    s.parse::<f32>().unwrap()
                } else {
                    eprintln!("Error: Invalid number specified for red scalar");
                    process::exit(1);
                }
            },
            false => 1.0
        },
        green_scalar: match matches.is_present(constants::param::PARAM_GREEN_WEIGHT) {
            true => {
                let s = matches.value_of(constants::param::PARAM_GREEN_WEIGHT).unwrap();
                if util::string_is_valid_f32(&s) {
                    s.parse::<f32>().unwrap()
                } else {
                    eprintln!("Error: Invalid number specified for red scalar");
                    process::exit(1);
                }
            },
            false => 1.0
        },
        blue_scalar: match matches.is_present(constants::param::PARAM_BLUE_WEIGHT) {
            true => {
                let s = matches.value_of(constants::param::PARAM_BLUE_WEIGHT).unwrap();
                if util::string_is_valid_f32(&s) {
                    s.parse::<f32>().unwrap()
                } else {
                    eprintln!("Error: Invalid number specified for red scalar");
                    process::exit(1);
                }
            },
            false => 1.0
        },
        color_noise_reduction: matches.is_present(constants::param::PARAM_COLOR_NOISE_REDUCTION),
        color_noise_reduction_amount: match matches.is_present(constants::param::PARAM_COLOR_NOISE_REDUCTION) {
            true => {
                let s = matches.value_of(constants::param::PARAM_COLOR_NOISE_REDUCTION).unwrap();
                if ! util::string_is_valid_i32(&s) {
                    eprintln!("Error: Invalid number specified for color noise reduction");
                    process::exit(1);
                }
                let color_noise_reduction = s.parse::<i32>().unwrap();
                if color_noise_reduction % 2 == 0 {
                    eprintln!("Error: Color noise reduction value must be odd");
                    process::exit(1);
                }
                if color_noise_reduction < 0 {
                    eprintln!("Error: Color noise reduction value must a positive number");
                    process::exit(1);
                }
                color_noise_reduction
            },
            false => 0
        },
        hot_pixel_detection_threshold: match matches.is_present(constants::param::PARAM_HPC_THRESHOLD) {
            true => {
                let s = matches.value_of(constants::param::PARAM_HPC_THRESHOLD).unwrap();
                if util::string_is_valid_f32(&s) {
                    s.parse::<f32>().unwrap()
                } else {
                    eprintln!("Error: Invalid number specified for HPC variance threshold");
                    process::exit(1);
                }
            },
            false => 0.0
        },
        hot_pixel_window_size: match matches.is_present(constants::param::PARAM_HPC_WINDOW_SIZE) {
            true => {
                let s = matches.value_of(constants::param::PARAM_HPC_WINDOW_SIZE).unwrap();
                if util::string_is_valid_i32(&s) {
                    s.parse::<i32>().unwrap()
                } else {
                    eprintln!("Error: Invalid number specified for HPC window size");
                    process::exit(1);
                }
            },
            false => 3
        },
        filename_suffix: String::from(constants::OUTPUT_FILENAME_APPEND)
    };

    let default_instrument = match matches.is_present(constants::param::PARAM_INSTRUMENT) {
        true => {
            Some(matches.value_of(constants::param::PARAM_INSTRUMENT).unwrap())
        },
        false => None
    };

    let only_new = matches.is_present(constants::param::PARAM_ONLY_NEW);

    let profiles: Vec<&str> = match matches.values_of(constants::param::PARAM_CAL_PROFILE) {
        Some(profiles) => profiles.collect(),
        None => vec!()
    };

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    panic::set_hook(Box::new(|_info| {
        print_fail(&format!("Internal Error!"));
    }));

    input_files.par_iter().for_each(|input_file| {
        let calibrator = get_calibrator_for_file(&input_file, default_instrument);
        match calibrator {
            Some(cal) => {

                if profiles.len() > 0 {
                    process_with_profiles(&cal, input_file, only_new, &profiles, |result| {
                        match result {
                            Ok(cc) => print_complete(&format!("{} ({})", path::basename(input_file), cc.cal_context.filename_suffix), cc.status),
                            Err(why) => {
                                eprintln!("Error: {}", why);
                                print_fail(&input_file.to_string());
                            }
                        }
                    });
                } else {
                    
                    
                    match cal.calibrator.process_file(input_file, &cal_context, only_new) {
                        Ok(cc) => print_complete(&format!("{} ({})", path::basename(input_file), cc.cal_context.filename_suffix), cc.status),
                        Err(why) => {
                            eprintln!("Error: {}", why);
                            print_fail(&input_file.to_string());
                        }
                    }
                }
            },
            None => {
                print_fail(&format!("{} - Error: Instrument Unknown!", path::basename(input_file)));
            }
        }
        
    });

}