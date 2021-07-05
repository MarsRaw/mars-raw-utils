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
                    .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
                        .short(constants::param::PARAM_OUTPUT_SHORT)
                        .long(constants::param::PARAM_OUTPUT)
                        .value_name("OUTPUT")
                        .help("Output")
                        .required(true)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();


    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let output_file = matches.value_of(constants::param::PARAM_OUTPUT).unwrap();

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    let mut max_x = 0_f64;
    let mut max_y = 0_f64;
    let mut images: Vec<rgbimage::RgbImage> = vec!();
    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            let instrument = enums::Instrument::M20NavcamRight;
            let img = rgbimage::RgbImage::open(String::from(*in_file), instrument).unwrap();
            if ! img.has_metadata() {
                eprintln!("ERROR: Metadata file not found for {}", in_file);
                eprintln!("Each image must have the associated metadata");
                process::exit(1);
            }

            let md = img.get_metadata().unwrap();
            if let Some(ref sf) = md.subframe_rect {
                if sf.len() == 4 {
                    let right_x = sf[2];
                    let bottom_y = sf[3];
                    max_x = if max_x > right_x { max_x } else { right_x };
                    max_y = if max_y > bottom_y { max_y } else { bottom_y };
                }
            }

            images.push(img);
        } else {
            eprintln!("File not found: {}", in_file);
            process::exit(2);
        }
    }

    vprintln!("Combined image width: {}", max_x);
    vprintln!("Combined image height: {}", max_y);

    let mut combined = rgbimage::RgbImage::new_with_size(max_x as usize, max_y as usize, enums::Instrument::None, enums::ImageMode::U16BIT).unwrap();
    
    for img in images {
        let md = img.get_metadata().unwrap();
        if let Some(ref sf) = md.subframe_rect {
            if sf.len() == 4 {
                let tl_x = sf[0] / md.scale_factor as f64;
                let tl_y = sf[1] / md.scale_factor as f64;
                combined.paste(&img, tl_x as usize, tl_y as usize).unwrap();
            }
        }
        
    }

    match combined.save(&output_file) {
        Err(why) => eprintln!("Error saving to output file: {}", why),
        Ok(_) => vprintln!("File saved.")
    };
}