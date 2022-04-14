use mars_raw_utils::prelude::*;

use sciimg::{
    enums::ImageMode,
    rgbimage,
    imagebuffer
};

#[macro_use]
extern crate clap;

use std::process;

use clap::{Arg, App};

// /*
//   "subframe_rect": [
//     2545.0,
//     1905.0,
//     2576.0,
//     1936.0
//   ],
//   "scale_factor": 2,

//     width: 1288
//     height: 968

// */

// #[derive(Debug)]
// struct Rgb {
//     pub r:f32,
//     pub g:f32,
//     pub b:f32
// }

// impl Rgb {

//     pub fn get_multipliers(self:&Rgb, other:&Rgb) -> Rgb {

//         if self.is_zero() || other.is_zero() {
//             return Rgb{r:0.0, g:0.0, b:0.0};
//         } 

//         let r = if other.r > 0.0 { self.r / other.r } else { 1.0 };
//         let g = if other.g > 0.0 { self.g / other.g } else { 1.0 };
//         let b = if other.b > 0.0 { self.b / other.b } else { 1.0 };

//         Rgb{
//             r:r,
//             g:g,
//             b:b
//         }
//     }

//     pub fn is_zero(self:&Rgb) -> bool {
//         self.r == 0.0 && self.g == 0.0 && self.b == 0.0
//     }
// }

// #[derive(Debug, Clone)]
// struct ColorDiff {
//     pub r:f32,
//     pub g:f32,
//     pub b:f32,
//     pub composite_min:f32,
//     pub composite_max:f32,
//     pub tile_min:f32,
//     pub tile_max:f32,
// }

// impl ColorDiff {

//     pub fn new() -> Self {
//         ColorDiff{
//             r:0.0,
//             g:0.0,
//             b:0.0,
//             composite_min:0.0,
//             composite_max:0.0,
//             tile_min:0.0,
//             tile_max:0.0
//         }
//     }

//     pub fn is_zero(self:&ColorDiff) -> bool {
//         self.r == 0.0 && self.g == 0.0 && self.b == 0.0
//     }

//     pub fn check_rgb_composite(self:&mut ColorDiff, rgb:&Rgb) {
//         self.composite_max = max!(self.composite_max, rgb.r, rgb.g, rgb.b);
//         self.composite_min = min!(self.composite_min, rgb.r, rgb.g, rgb.b);
//     }

//     pub fn check_rgb_tile(self:&mut ColorDiff, rgb:&Rgb) {
//         self.tile_max = max!(self.tile_max, rgb.r, rgb.g, rgb.b);
//         self.tile_min = min!(self.tile_min, rgb.r, rgb.g, rgb.b);
//     }

// }

// #[derive(Debug, Clone)]
// struct Tile {
//     pub image:rgbimage::RgbImage,
//     pub source_file_path:String,
//     pub top_left_x:usize,
//     pub top_left_y:usize,
//     pub bottom_right_x:usize,
//     pub bottom_right_y:usize
// }


// impl Tile {

//     pub fn load(in_file:&str) -> error::Result<Self> {
//         if ! path::file_exists(in_file) {
//             panic!("File not found: {}", in_file);
//         }

//         let instrument = enums::Instrument::M20NavcamRight;
//         let mut image = rgbimage::RgbImage::open(String::from(in_file), instrument).unwrap();
//         if ! image.has_metadata() {
//             eprintln!("ERROR: Metadata file not found for {}", in_file);
//             eprintln!("Each image must have the associated metadata");
//             panic!("ERROR: Metadata file not found for {}", in_file);
//         }
        
//         image.crop(2, 2, image.width - 4, image.height - 4).unwrap();

//         let md = image.get_metadata().unwrap();
//         let scale = md.scale_factor;
//         if let Some(ref sf) = md.subframe_rect {
//             if sf.len() == 4 {
//                 let tl_x = (sf[0] / scale as f64).floor() as usize + 2;
//                 let tl_y = (sf[1] / scale as f64).floor() as usize + 2;
//                 let right_x = tl_x + image.width;
//                 let bottom_y = tl_y + image.height;
                
//                 vprintln!("Image top left x/y: {}/{}", tl_x, tl_y);
//                 vprintln!("Image bottom right x/y: {}/{}", right_x, bottom_y);

//                 Ok(Tile{
//                     image:image.clone(),
//                     source_file_path:in_file.to_string(),
//                     top_left_x: tl_x,
//                     top_left_y: tl_y,
//                     bottom_right_x: right_x,
//                     bottom_right_y: bottom_y
//                 })
//             } else {
//                 Err("Tile lacks expected subrect data")
//             }
//         } else {
//             Err("Tile lacks metadata")
//         }
        
//     }

//     pub fn determine_composite_size(tiles:&Vec<Tile>) -> (usize, usize) {

//         let mut max_x : usize = 0;
//         let mut max_y : usize = 0;

//         tiles.iter().for_each(|t| {

//             max_x = if t.bottom_right_x > max_x { t.bottom_right_x } else { max_x };
//             max_y = if t.bottom_right_y > max_y { t.bottom_right_y } else { max_y };

//         });

//         (max_x, max_y)
//     }

//     pub fn rgb_at_point(&self, x:usize, y:usize) -> Rgb {
//         Rgb{
//             r: self.image.red().get(x, y).unwrap(),
//             g: self.image.green().get(x, y).unwrap(),
//             b: self.image.blue().get(x, y).unwrap()
//         }
//     }

//     pub fn rgb_at_composite_point(&self, x:usize, y:usize) -> Rgb {
//         let tile_x = x - self.top_left_x;
//         let tile_y = y - self.top_left_y;
//         self.rgb_at_point(tile_x, tile_y)
//     }

//     pub fn apply_color_diff(&mut self, cdiff:&ColorDiff) {

//         let maxdiff = cdiff.composite_max - cdiff.tile_max;
//         let mindiff = cdiff.composite_min - cdiff.tile_min;

//         let (tile_min, tile_max) = self.image.get_min_max_all_channel();

//         vprintln!("Normalize min/max diff {} {}", mindiff, maxdiff);
//         vprintln!("Tile min/max {}, {}", tile_min, tile_max);
//         vprintln!("Normalize between {}, {}", tile_min + mindiff, tile_max + maxdiff);
//         self.image.normalize_between(tile_min + mindiff, tile_max + maxdiff).unwrap();
//     }

//     pub fn apply_multipliers(&mut self, multiplier:&Rgb) {
//         self.image.apply_weight(multiplier.r, multiplier.g, multiplier.b).unwrap();
//     }

// }



// struct Composite {
//     image:rgbimage::RgbImage
// }

// impl Composite {

//     pub fn new(width:usize, height:usize) -> Composite {
//         Composite {
//             image:rgbimage::RgbImage::new_with_size(width, height, ImageMode::U16BIT).unwrap()
//         }
//     }

//     pub fn rgb_at_point(self:&Composite, x:usize, y:usize) -> Rgb {
//         Rgb{
//             r: self.image.red().get(x, y).unwrap(),
//             g: self.image.green().get(x, y).unwrap(),
//             b: self.image.blue().get(x, y).unwrap()
//         }
//     }



//     fn color_diff_vert(self:&Composite, tile:&Tile, x_offset:usize) -> ColorDiff {

//         let mut cdiff = ColorDiff::new();
//         cdiff.composite_min = 65535.0;
//         cdiff.tile_min = 65535.0;
//         let mut count = 0;

//         (0..tile.image.height).into_iter().for_each(|i| {

//             let composite_rgb = self.rgb_at_point(tile.top_left_x + x_offset, tile.top_left_y + i);
//             let tile_rgb = tile.rgb_at_point(x_offset, i);
//             if ! composite_rgb.is_zero()  {
//                 let multiplier = composite_rgb.get_multipliers(&tile_rgb);
//                 if ! multiplier.is_zero() {

//                     cdiff.check_rgb_composite(&composite_rgb);
//                     cdiff.check_rgb_tile(&tile_rgb);

//                     cdiff.r += multiplier.r;
//                     cdiff.g += multiplier.g;
//                     cdiff.b += multiplier.b;
//                     count += 1; 
//                 }
                

//                 //vprintln!("{:?} {:?} {:?} {:?} -- {}", avg_mult, composite_rgb, tile_rgb, multiplier, i);
//             }


//         });

        

//         if count > 0 {
//             cdiff.r /= count as f32;
//             cdiff.g /= count as f32;
//             cdiff.b /= count as f32;
//         }
        
//         cdiff
//     }

//     fn color_diff_horiz(self:&Composite, tile:&Tile, y_offset:usize) -> ColorDiff {

//         let mut cdiff = ColorDiff::new();
//         cdiff.composite_min = 65535.0;
//         cdiff.tile_min = 65535.0;
//         let mut count = 0;

//         (0..tile.image.width).into_iter().for_each(|i| {

//             let composite_rgb = self.rgb_at_point(tile.top_left_x + i, tile.top_left_y + y_offset);
//             let tile_rgb = tile.rgb_at_point(i, y_offset);

            

//             if ! composite_rgb.is_zero() {
//                 let multiplier = composite_rgb.get_multipliers(&tile_rgb);
//                 if !multiplier.is_zero() {

//                     cdiff.check_rgb_composite(&composite_rgb);
//                     cdiff.check_rgb_tile(&tile_rgb);

//                     cdiff.r += multiplier.r;
//                     cdiff.g += multiplier.g;
//                     cdiff.b += multiplier.b;
//                     count += 1; 
//                 }
                
//             }
//         });

//         if count > 0 {
//             cdiff.r /= count as f32;
//             cdiff.g /= count as f32;
//             cdiff.b /= count as f32;
//         }
//         cdiff
//     }


//     fn compute_multipliers(self:&Composite, tile:&Tile) -> ColorDiff {
//         let mut cdiff = ColorDiff::new();
//         let mut count = 0;

//         let left = self.color_diff_vert(&tile, 0);
//         let right = self.color_diff_vert(&tile, tile.image.width - 1);
//         let top = self.color_diff_horiz(&tile, 0);
//         let bottom = self.color_diff_horiz(&tile, tile.image.height - 1);

//         if ! left.is_zero() {
//             vprintln!("Including left side multiples");
//             cdiff.r += left.r;
//             cdiff.g += left.g;
//             cdiff.b += left.b;
//             count += 1;
//         }

//         if ! right.is_zero() {
//             vprintln!("Including right side multiples");
//             cdiff.r += right.r;
//             cdiff.g += right.g;
//             cdiff.b += right.b;
//             count += 1;
//         }

//         if ! top.is_zero() {
//             vprintln!("Including top side multiples");
//             cdiff.r += top.r;
//             cdiff.g += top.g;
//             cdiff.b += top.b;
//             count += 1;
//         }

//         if ! bottom.is_zero() {
//             vprintln!("Including bottom side multiples");
//             cdiff.r += bottom.r;
//             cdiff.g += bottom.g;
//             cdiff.b += bottom.b;
//             count += 1;
//         }

//         if count > 0 {
//             cdiff.r /= count as f32;
//             cdiff.g /= count as f32;
//             cdiff.b /= count as f32;
//         }

//         cdiff.composite_max = max!(left.composite_max, right.composite_max, top.composite_max, bottom.composite_max);
//         cdiff.tile_max = max!(left.tile_max, right.tile_max, top.tile_max, bottom.tile_max);

//         cdiff.composite_min = min!(left.composite_min, right.composite_min, top.composite_min, bottom.composite_min);
//         cdiff.tile_min = min!(left.tile_min, right.tile_min, top.tile_min, bottom.tile_min);

//         vprintln!("{:?} {}", cdiff, count);
//         cdiff
//     }

//     pub fn paste(self:&mut Composite, tile:&Tile) -> error::Result<&str> {
//         let mut tile_copy = tile.to_owned();

//         let cdiff = self.compute_multipliers(&tile);

//         if ! cdiff.is_zero() {
//             vprintln!("Applying color multiples");
//             tile_copy.apply_color_diff(&cdiff);
//         }

//         self.image.paste(&tile_copy.image, tile_copy.top_left_x, tile_copy.top_left_y)
//     }

//     pub fn finalize_and_save(self:&mut Composite, output_file:&str) -> error::Result<&str> {
//         self.image.normalize_to_16bit().unwrap();
//         self.image.save(&output_file)
//     }
// }



// fn main() {
    
//     let matches = App::new(crate_name!())
//                     .version(crate_version!())
//                     .author(crate_authors!())
//                     .arg(Arg::with_name(constants::param::PARAM_INPUTS)
//                         .short(constants::param::PARAM_INPUTS_SHORT)
//                         .long(constants::param::PARAM_INPUTS)
//                         .value_name("INPUT")
//                         .help("Input")
//                         .required(true)
//                         .multiple(true)
//                         .takes_value(true))
//                     .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
//                         .short(constants::param::PARAM_OUTPUT_SHORT)
//                         .long(constants::param::PARAM_OUTPUT)
//                         .value_name("OUTPUT")
//                         .help("Output")
//                         .required(true)
//                         .takes_value(true))
//                     .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
//                         .short(constants::param::PARAM_VERBOSE)
//                         .help("Show verbose output"))
//                     .get_matches();


//     if matches.is_present(constants::param::PARAM_VERBOSE) {
//         print::set_verbose(true);
//     }

//     let output_file = matches.value_of(constants::param::PARAM_OUTPUT).unwrap();

//     let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

//     let mut tiles: Vec<Tile> = vec!();

//     for in_file in input_files.iter() {
//         if path::file_exists(in_file) {
//             match Tile::load(in_file) {
//                 Ok(tile) => tiles.push(tile),
//                 Err(why) => {
//                     eprintln!("{}", why);
//                     process::exit(2);
//                 }
//             }

//         } else {
//             eprintln!("File not found: {}", in_file);
//             process::exit(2);
//         }
//     }

//     let (composite_width, composite_height) = Tile::determine_composite_size(&tiles);

//     vprintln!("Combined image width: {}", composite_width);
//     vprintln!("Combined image height: {}", composite_height);


//     let mut composite = Composite::new(composite_width, composite_height);

//     let mut _composite = rgbimage::RgbImage::new_with_size(composite_width, composite_height, enums::Instrument::None, enums::ImageMode::U16BIT).unwrap();
    
//     tiles.iter().for_each(|t| {
//         vprintln!("Pasting tile {}", t.source_file_path);
//         composite.paste(&t).expect("Failed to paste tile to composite");
//     });

//     vprintln!("Finalize and save composite...");
//     match composite.finalize_and_save(&output_file) {
//         Err(why) => eprintln!("Error saving to output file: {}", why),
//         Ok(_) => vprintln!("File saved.")
//     };

// }



struct Tile {
    pub source_path:String,
    pub image:MarsImage,
    pub top_left_x:usize,
    pub top_left_y:usize,
    pub bottom_right_x:usize,
    pub bottom_right_y:usize,
    pub scale:u32

}

impl Tile {
    pub fn new(source_path:&str) -> Self {
        let instrument = Instrument::M20NavcamLeft;
        let image = MarsImage::open(String::from(&source_path.to_owned()), instrument);
        
        match image.metadata.clone() {
            Some(md) => {
                let scale = md.scale_factor;
                if let Some(ref sf) = md.subframe_rect {
                    if sf.len() != 4 {
                        panic!("Subframe rect field an invalid length");
                    }
                    let tl_x = (sf[0] / scale as f64).floor() as usize + 2;
                    let tl_y = (sf[1] / scale as f64).floor() as usize + 2;
                    let right_x = tl_x + image.image.width;
                    let bottom_y = tl_y + image.image.height;

                    Tile {
                        source_path:source_path.to_string(),
                        image:image,
                        top_left_x: tl_x,
                        top_left_y: tl_y,
                        bottom_right_x: right_x,
                        bottom_right_y: bottom_y,
                        scale:scale
                    }
                } else {
                    panic!("Subframe rect field is empty");
                }
            },
            None => {
                panic!("Metadata not found for image {}", source_path);
            }
        }
    }
}


struct Composite {

    pub scale:u32,
    pub width:usize,
    pub height:usize,
    composite_image:rgbimage::RgbImage
}

impl Composite {
    pub fn new(tiles:&Vec<Tile>) -> Self {

        if tiles.len() == 0 {
            panic!("Cannot assemble composite with no tiles!");
        }

        let scale = tiles[0].scale;

        let (max_x, max_y) = Composite::determine_composite_size(&tiles);

        let composite_image = rgbimage::RgbImage::new_with_bands(max_x, max_y, 3, ImageMode::U16BIT).unwrap();

        vprintln!("Composite has width {}px, height {}px, and scale factor of {}", max_x, max_y, scale);
        
        Composite {
            scale:scale,
            width:max_x,
            height:max_y,
            composite_image:composite_image
        }
    }

    fn determine_composite_size(tiles:&Vec<Tile>) -> (usize, usize) {

        let mut max_x : usize = 0;
        let mut max_y : usize = 0;

        tiles.iter().for_each(|t| {

            max_x = if t.bottom_right_x > max_x { t.bottom_right_x } else { max_x };
            max_y = if t.bottom_right_y > max_y { t.bottom_right_y } else { max_y };

        });

        (max_x, max_y)
    }

    pub fn paste_tiles(&mut self, tiles:&Vec<Tile>) {
        tiles.iter().for_each(|t| { 
            self.paste_tile(&t);
        });
    }

    pub fn paste_tile(&mut self, tile:&Tile) {
        self.composite_image.paste(&tile.image.image, tile.top_left_x, tile.top_left_y);
    }

    pub fn finalize_and_save(&mut self, output_path:&str) {
        self.composite_image.normalize_to_16bit();
        self.composite_image.save(output_path);
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

    let mut tiles:Vec<Tile> = vec!();

    for in_file in input_files.iter() {
        if ! path::file_exists(in_file) {
            eprintln!("File not found: {}", in_file);
            process::exit(1);
        }
        let tile = Tile::new(&in_file);
        tiles.push(tile);
    }

    

    // TODO: This is bad form.
    vprintln!("Creating composite structure");
    let mut composite = Composite::new(&tiles);

    vprintln!("Adding {} tiles to composite", tiles.len());
    composite.paste_tiles(&mut tiles);

    vprintln!("Saving composite to {}", output_file);
    composite.finalize_and_save(&output_file);
}