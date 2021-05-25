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

fn process_file(input_file:&str, x:usize, y:usize, width:usize, height:usize) {
    let mut raw = rgbimage::RgbImage::open(String::from(input_file), enums::Instrument::None).unwrap();

    if x >= raw.width {
        eprintln!("X parameter is out of bounds: {}. Must be between 0 and {}", x, raw.width - 1);
        process::exit(2);
    }

    if y >= raw.height {
        eprintln!("Y parameter is out of bounds: {}. Must be between 0 and {}", x, raw.height - 1);
        process::exit(2);
    }

    if width > raw.width - x {
        eprintln!("Specified width exceeds maximum allowable value");
        process::exit(2);
    }

    if height > raw.height - y {
        eprintln!("Specified height exceeds maximum allowable value");
        process::exit(2);
    }

    let out_file = util::append_file_name(input_file, "crop");


    vprintln!("Cropping with x={}, y={}, width={}, height={}", x, y, width, height);
    raw.crop(x, y, width, height).unwrap();

    vprintln!("Saving output to {}", out_file);

    match raw.save(&out_file) {
        Ok(_) => {
            vprintln!("Process completed");
        },
        Err(e) => {
            eprintln!("Error saving file: {}", e);
            process::exit(3);
        }
    }

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
                    .arg(Arg::with_name("crop")
                        .short("c")
                        .long("crop")
                        .value_name("WINDOW_SIZE")
                        .help("Crop as x,y,width,height")
                        .required(true)
                        .takes_value(true))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    // If, for some weird reason, clap misses the missing parameter...
    if ! matches.is_present(constants::param::PARAM_INPUTS) {
        println!("{}", matches.usage());
        process::exit(1);
    }

    let crop_str = match matches.is_present("crop") {
        true => {
            matches.value_of("crop").unwrap()
        },
        false => {
            eprintln!("Error: Missing crop parameter");
            eprintln!("{}", matches.usage());
            process::exit(1);
        }
    };

    //https://stackoverflow.com/questions/26536871/how-can-i-convert-a-string-of-numbers-to-an-array-or-vector-of-integers-in-rust
    let crop_numbers: Vec<usize> = crop_str.split(",")
                                    .map(|s| s.parse().expect("parse error"))
                                    .collect();
    if crop_numbers.len() != 4 {
        eprintln!("Invalid number of crop parameters specified.");
        eprintln!("{}", matches.usage());
        process::exit(1);
    }
    
    let x = crop_numbers[0];
    let y = crop_numbers[1];
    let width = crop_numbers[2];
    let height = crop_numbers[3];

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file, x, y, width, height);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }
}