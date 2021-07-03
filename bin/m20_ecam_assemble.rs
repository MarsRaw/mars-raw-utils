use mars_raw_utils::{
    constants, 
    print, 
    vprintln, 
    path,
    util,
    m20,
    enums,
    rgbimage
};

#[macro_use]
extern crate clap;

use std::process;

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
                    .get_matches();


    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            let instrument = enums::Instrument::M20NavcamRight;
            let img = rgbimage::RgbImage::open(String::from(*in_file), instrument).unwrap();
            if ! img.has_metadata() {
                eprintln!("ERROR: Metadata file not found for {}", in_file);
                eprintln!("Each image must have the associated metadata");
            }

            // Do more stuff...
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

}