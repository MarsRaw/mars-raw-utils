use mars_raw_utils::{
    prelude::*,
    diffgif
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
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .arg(Arg::with_name(constants::param::PARAM_LEVELS_BLACK_LEVEL)
                        .short(constants::param::PARAM_LEVELS_BLACK_LEVEL_SHORT)
                        .long(constants::param::PARAM_LEVELS_BLACK_LEVEL)
                        .value_name("BLACK_LEVEL")
                        .help("Black level")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_LEVELS_WHITE_LEVEL)
                        .short(constants::param::PARAM_LEVELS_WHITE_LEVEL_SHORT)
                        .long(constants::param::PARAM_LEVELS_WHITE_LEVEL)
                        .value_name("WHITE_LEVEL")
                        .help("White level")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_DELAY)
                        .short(constants::param::PARAM_DELAY_SHORT)
                        .long(constants::param::PARAM_DELAY)
                        .value_name("PARAM_DELAY")
                        .help("Interframe delay in increments of 10ms")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_GAMMA)
                        .short(constants::param::PARAM_GAMMA_SHORT)
                        .long(constants::param::PARAM_GAMMA)
                        .value_name("PARAM_GAMMA")
                        .help("Gamma")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_LOWPASS)
                        .short(constants::param::PARAM_LOWPASS_SHORT)
                        .long(constants::param::PARAM_LOWPASS)
                        .value_name("PARAM_LOWPASS")
                        .help("Lowpass window size")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
                        .short(constants::param::PARAM_OUTPUT_SHORT)
                        .long(constants::param::PARAM_OUTPUT)
                        .value_name("OUTPUT")
                        .help("Output")
                        .required(true)
                        .takes_value(true)) 
                    .arg(Arg::with_name(constants::param::PARAM_PRODUCT_TYPE)
                        .short(constants::param::PARAM_PRODUCT_TYPE_SHORT)
                        .long(constants::param::PARAM_PRODUCT_TYPE)
                        .value_name("PARAM_PRODUCT_TYPE")
                        .help("Product type (std, diff, stacked)")
                        .required(false)
                        .takes_value(true))
                    .get_matches_from(wild::args());

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let black_level : f32 = match matches.is_present(constants::param::PARAM_LEVELS_BLACK_LEVEL) {
        true => {
            let s = matches.value_of(constants::param::PARAM_LEVELS_BLACK_LEVEL).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap() / 100.0
            } else {
                eprintln!("Error: Invalid number specified for black level");
                process::exit(1);
            }
        },
        false => {
            0.0
        }
    };



    let white_level : f32 = match matches.is_present(constants::param::PARAM_LEVELS_WHITE_LEVEL) {
        true => {
            let s = matches.value_of(constants::param::PARAM_LEVELS_WHITE_LEVEL).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap() / 100.0
            } else {
                eprintln!("Error: Invalid number specified for white level");
                process::exit(1);
            }
        },
        false => {
            1.0
        }
    };

    let gamma : f32 = match matches.is_present(constants::param::PARAM_GAMMA) {
        true => {
            let s = matches.value_of(constants::param::PARAM_GAMMA).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for gamma");
                process::exit(1);
            }
        },
        false => {
            1.0
        }
    };

    let delay : u16 = match matches.is_present(constants::param::PARAM_DELAY) {
        true => {
            let s = matches.value_of(constants::param::PARAM_DELAY).unwrap();
            if util::string_is_valid_u16(&s) {
                s.parse::<u16>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for interframe delay");
                process::exit(1);
            }
        },
        false => {
            10
        }
    };

    let lowpass_window_size : u8 = match matches.is_present(constants::param::PARAM_LOWPASS) {
        true => {
            let s = matches.value_of(constants::param::PARAM_LOWPASS).unwrap();
            if util::string_is_valid_u16(&s) {
                s.parse::<u8>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for lowpass window size");
                process::exit(1);
            }
        },
        false => {
            0
        }
    };

    let product_type = match matches.is_present(constants::param::PARAM_PRODUCT_TYPE) {
        true => {
            let s = matches.value_of(constants::param::PARAM_PRODUCT_TYPE).unwrap();
            match diffgif::ProductType::from_str(&s) {
                None => {
                    eprintln!("Invalid output product type {}. Must be 'std', 'diff', or 'stacked'", s);
                    process::exit(1);
                },
                Some(pt) => pt
            }
        },
        false => diffgif::ProductType::STANDARD
    };


    let output = matches.value_of("output").unwrap();

    // Some rules on the parameters
    // TODO: Keep an eye on floating point errors
    if white_level < 0.0 || black_level < 0.0{
        eprintln!("Levels cannot be negative");
        process::exit(1);
    }

    if white_level < black_level {
        eprintln!("White level cannot be less than black level");
        process::exit(1);
    }

    // if white_level > 1.0 || black_level > 1.0 {
    //     eprintln!("Levels cannot exceed 1.0");
    //     process::exit(1);
    // }

    if gamma <= 0.0 {
        eprintln!("Gamma cannot be zero or negative");
        process::exit(1);
    }

    let input_files_str: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();
    let in_files : Vec<String> = input_files_str.iter().map(|s| String::from(*s)).collect();

    diffgif::process(&diffgif::DiffGif{
        input_files: in_files,
        output: String::from(output),
        product_type: product_type,
        black_level: black_level,
        white_level: white_level,
        gamma: gamma,
        delay: delay,
        lowpass_window_size: lowpass_window_size
    });

    
}