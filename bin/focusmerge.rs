use mars_raw_utils::{
    constants, 
    print, 
    util,
    focusmerge
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
                    .arg(Arg::with_name(constants::param::PARAM_QUALITY_WINDOW_SIZE)
                        .short(constants::param::PARAM_QUALITY_WINDOW_SIZE_SHORT)
                        .long(constants::param::PARAM_QUALITY_WINDOW_SIZE)
                        .value_name("QUALITY_WINDOW_SIZE")
                        .help("Quality determination window size (pixels)")
                        .required(false)
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

    let quality_window_size : usize = match matches.is_present(constants::param::PARAM_QUALITY_WINDOW_SIZE) {
        true => {
            let s = matches.value_of(constants::param::PARAM_QUALITY_WINDOW_SIZE).unwrap();
            if util::string_is_valid_u16(&s) {
                s.parse::<usize>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for quality determination window size");
                process::exit(1);
            }
        },
        false => {
            15
        }
    };

    focusmerge::focusmerge(&input_files, quality_window_size, &output);
}