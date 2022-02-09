use mars_raw_utils::{
    constants, 
    print, 
    vprintln, 
    path,
    util
};

use sciimg::rgbimage;

#[macro_use]
extern crate clap;

use clap::{Arg, App};
use std::process;


fn process_file(input_file:&str, black_level:f32, white_level:f32, gamma:f32) {
    let mut raw = rgbimage::RgbImage::open(&String::from(input_file)).unwrap();

    vprintln!("Black: {}, White: {}, Gamma: {}, {}", black_level, white_level, gamma, input_file);
    raw.levels(black_level, white_level, gamma);

    let out_file = util::append_file_name(input_file, "lvls");
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
                    .arg(Arg::with_name(constants::param::PARAM_GAMMA)
                        .short(constants::param::PARAM_GAMMA_SHORT)
                        .long(constants::param::PARAM_GAMMA)
                        .value_name("PARAM_GAMMA")
                        .help("Gamma")
                        .required(false)
                        .takes_value(true))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let black_level : f32 = match matches.is_present(constants::param::PARAM_LEVELS_BLACK_LEVEL) {
        true => {
            let s = matches.value_of(constants::param::PARAM_LEVELS_BLACK_LEVEL).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
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
                s.parse::<f32>().unwrap()
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

    if white_level > 1.0 || black_level > 1.0 {
        eprintln!("Levels cannot exceed 1.0");
        process::exit(1);
    }

    if gamma <= 0.0 {
        eprintln!("Gamma cannot be zero or negative");
        process::exit(1);
    }

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file, black_level, white_level, gamma);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

}