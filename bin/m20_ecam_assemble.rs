use mars_raw_utils::{
    constants, 
    print, 
    vprintln, 
    path,
    util,
    m20,
    enums,
    rgbimage,
    error,
    metadata
};

#[macro_use]
extern crate clap;

use std::process;

use clap::{Arg, App};

/*
  "subframe_rect": [
    2545.0,
    1905.0,
    2576.0,
    1936.0
  ],
  "scale_factor": 2,

    width: 1288
    height: 968

*/


struct Tile {
    pub image:rgbimage::RgbImage,
    pub top_left_x:usize,
    pub top_left_y:usize,
    pub bottom_right_x:usize,
    pub bottom_right_y:usize
}

impl Tile {

    pub fn load(in_file:&str) -> error::Result<Self> {
        if ! path::file_exists(in_file) {
            panic!("File not found: {}", in_file);
        }

        let instrument = enums::Instrument::M20NavcamRight;
        let image = rgbimage::RgbImage::open(String::from(in_file), instrument).unwrap();
        if ! image.has_metadata() {
            eprintln!("ERROR: Metadata file not found for {}", in_file);
            eprintln!("Each image must have the associated metadata");
            panic!("ERROR: Metadata file not found for {}", in_file);
        }
        
        let md = image.get_metadata().unwrap();
        let scale = md.scale_factor;
        if let Some(ref sf) = md.subframe_rect {
            if sf.len() == 4 {
                let tl_x = (sf[0] / scale as f64).floor();
                let tl_y = (sf[1] / scale as f64).floor();
                let right_x = (tl_x as f64 + sf[2] / scale as f64) as usize;
                let bottom_y = (tl_y as f64 + sf[3] / scale as f64) as usize;
                
                vprintln!("Image top left x/y: {}/{}", tl_x, tl_y);
                vprintln!("Image bottom right x/y: {}/{}", right_x, bottom_y);

                Ok(Tile{
                    image:image.clone(),
                    top_left_x: tl_x as usize,
                    top_left_y: tl_y as usize,
                    bottom_right_x: right_x,
                    bottom_right_y: bottom_y
                })
            } else {
                Err("Tile lacks expected subrect data")
            }
        } else {
            Err("Tile lacks metadata")
        }
        
    }

    pub fn determine_composite_size(tiles:&Vec<Tile>) -> (usize, usize) {

        let mut max_x : usize = 0;
        let mut max_y : usize = 0;

        tiles.iter().for_each(|t| {

            max_x = if t.bottom_right_x > max_x { t.bottom_right_x } else { max_x };
            max_y = if t.bottom_right_y > max_y { t.bottom_right_y } else { max_y };

        });

        (max_x, max_y)
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

    let mut tiles: Vec<Tile> = vec!();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            match Tile::load(in_file) {
                Ok(tile) => tiles.push(tile),
                Err(why) => {
                    eprintln!("{}", why);
                    process::exit(2);
                }
            }

        } else {
            eprintln!("File not found: {}", in_file);
            process::exit(2);
        }
    }

    let (composite_width, composite_height) = Tile::determine_composite_size(&tiles);

    vprintln!("Combined image width: {}", composite_width);
    vprintln!("Combined image height: {}", composite_height);

    let mut composite = rgbimage::RgbImage::new_with_size(composite_width, composite_height, enums::Instrument::None, enums::ImageMode::U16BIT).unwrap();
    
    tiles.iter().for_each(|t| {
        composite.paste(&t.image, t.top_left_x, t.top_left_y).expect("Failed to paste tile to composite");
    });

    vprintln!("Normalizing...");
    composite.normalize_to_16bit_with_max(255.0).unwrap();

    match composite.save(&output_file) {
        Err(why) => eprintln!("Error saving to output file: {}", why),
        Ok(_) => vprintln!("File saved.")
    };
}