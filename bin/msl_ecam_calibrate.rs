use mars_raw_utils::{
    constants, 
    print, 
    vprintln, 
    rgbimage, 
    enums, 
    path,
    util,
    inpaint
};

#[macro_use]
extern crate clap;

use clap::{Arg, App};

use std::process;

// Doesn't support subframed images yet since we won't know what part of the sensor was
// used from the raws alone. If it's in the JSON response from the raw image site, then
// maybe I can embed that data into the jpegs (EXIF) when downloaded using msl_fetch_raws
// and trigger off of that. Still need to think of times when someone downloads the image
// directly from the webpage using their browser as the website often prepends a wonky
// prefix to the image filename.
//
// Also leaving in the ILT parameter until I iron out the cases in which it's needed
// for ECAM. 
fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, _no_ilt:bool) {
    
    let mut instrument = enums::Instrument::MslNavCamRight;

    // Attempt to figure out camera from file name
    if util::filename_char_at_pos(&input_file, 0) == 'N' {         // NAVCAMS
        if util::filename_char_at_pos(&input_file, 1) == 'L' {     // Left
            instrument = enums::Instrument::MslNavCamLeft;
        } else {                                   // Assume Right
            instrument = enums::Instrument::MslNavCamRight;
        }
    } else if util::filename_char_at_pos(&input_file, 0) == 'F' {  // FHAZ
        if util::filename_char_at_pos(&input_file, 1)  == 'L' {     // Left
            instrument = enums::Instrument::MslFrontHazLeft;
        } else {                                   // Assume Right
            instrument = enums::Instrument::MslFrontHazRight;
        }  
    } else if util::filename_char_at_pos(&input_file, 0) == 'R' {  // RHAZ
        if util::filename_char_at_pos(&input_file, 1)  == 'L' {     // Left
            instrument = enums::Instrument::MslRearHazLeft;
        } else {                                   // Assume Right
            instrument = enums::Instrument::MslRearHazRight;
        }
    }

    let mut raw = rgbimage::RgbImage::open(input_file, instrument).unwrap();

    if inpaint::inpaint_supported_for_instrument(instrument) {
        vprintln!("Inpainting...");
        raw.apply_inpaint_fix().unwrap();
    } else {
        vprintln!("Inpainting not supported for instrument {:?}", instrument);
    }
    

    let data_max = 255.0;

    // if ! no_ilt {
    //     vprintln!("Decompanding...");
    //     raw.decompand().unwrap();
    //     data_max = decompanding::get_max_for_instrument(instrument) as f32;
    // }
    

    vprintln!("Flatfielding...");
    raw.flatfield().unwrap();
    
    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar).unwrap();

    vprintln!("Normalizing...");
    raw.normalize_to_16bit_with_max(data_max).unwrap();

    vprintln!("Writing to disk...");

    let out_file = input_file.replace(".jpg", "-rjcal.png").replace(".JPG", "-rjcal.png");
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
                    .arg(Arg::with_name(constants::param::PARAM_RAW_COLOR)
                        .short(constants::param::PARAM_RAW_COLOR_SHORT)
                        .long(constants::param::PARAM_RAW_COLOR)
                        .help("Raw color, skip ILT (not currently used)"))
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
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let mut red_scalar = constants::DEFAULT_RED_WEIGHT;
    let mut green_scalar = constants::DEFAULT_GREEN_WEIGHT;
    let mut blue_scalar = constants::DEFAULT_BLUE_WEIGHT;
    let mut no_ilt = false;
    
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

    if matches.is_present(constants::param::PARAM_RAW_COLOR) {
        no_ilt = true;
    }

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file, red_scalar, green_scalar, blue_scalar, no_ilt);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

    
}
