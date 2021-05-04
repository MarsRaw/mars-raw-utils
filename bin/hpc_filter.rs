use mars_raw_utils::{
    constants, 
    print, 
    vprintln, 
    rgbimage, 
    enums, 
    path,
    util
};

#[macro_use]
extern crate clap;

use clap::{Arg, App};

use std::process;

fn process_file(input_file:&str, hpc_threshold:f32, hpc_window_size:i32) {

    let mut raw = rgbimage::RgbImage::open(String::from(input_file), enums::Instrument::None).unwrap();

    if hpc_threshold > 0.0 {
        vprintln!("Hot pixel correction with variance threshold {}...", hpc_threshold);
        raw.hot_pixel_correction(hpc_window_size, hpc_threshold).unwrap();
    }
    
    // DON'T ASSUME THIS!
    let data_max = 255.0;

    vprintln!("Normalizing...");
    raw.normalize_to_16bit_with_max(data_max).unwrap();

    vprintln!("Writing to disk...");

    let out_file = input_file.replace(".jpg", "-hpc.png")
                            .replace(".JPG", "-hpc.png")
                            .replace(".png", "-hpc.png")
                            .replace(".PNG", "-hpc.png")
                            .replace(".tif", "-hpc.png")
                            .replace(".TIF", "-hpc.png");
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
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
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
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let mut hpc_threshold = 0.0;
    let mut hpc_window_size = 3;

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

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file, hpc_threshold, hpc_window_size);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

    
}
