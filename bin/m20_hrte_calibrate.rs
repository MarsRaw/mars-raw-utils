use mars_raw_utils::{
    constants, 
    print, 
    vprintln, 
    path, 
    m20
};

#[macro_use]
extern crate clap;
use clap::{Arg, App};

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
                    .arg(Arg::with_name(constants::param::PARAM_ONLY_NEW)
                        .short(constants::param::PARAM_ONLY_NEW_SHORT)
                        .help("Only new images. Skipped processed images."))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let mut only_new = false;
    if matches.is_present(constants::param::PARAM_ONLY_NEW) {
        only_new = true;
    }

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            m20::helirte::process_file(in_file, only_new);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }
}
