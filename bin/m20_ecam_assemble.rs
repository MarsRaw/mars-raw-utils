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

#[derive(Debug)]
struct Multiples {
    r: f32,
    g: f32,
    b: f32,
    c: f32
}

fn get_multiples_at_point(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, c_x:usize, c_y:usize, i_x:usize, i_y:usize) -> error::Result<Multiples> {

    let c_r = combined.red().get(c_x, c_y).unwrap();
    let c_g = combined.green().get(c_x, c_y).unwrap();
    let c_b = combined.blue().get(c_x, c_y).unwrap();

    if c_r == 0.0 && c_g == 0.0 && c_b == 0.0 {
        return Err("Point is black, don't use");
    }

    let i_r = img.red().get(i_x, i_y).unwrap();
    let i_g = img.green().get(i_x, i_y).unwrap();
    let i_b = img.blue().get(i_x, i_y).unwrap();

    if i_r == 0.0 && i_g == 0.0 && i_b == 0.0 {
        return Err("Point is black, don't use");
    }

    let m_r = c_r / i_r;
    let m_g = c_g / i_g;
    let m_b = c_b / i_b;

    // if m_r.is_nan() || m_g.is_nan() || m_b.is_nan() {
    //     return Err("NaN pixel value found");
    // }
    
    Ok(Multiples{
        r:m_r,
        g:m_g,
        b:m_b,
        c:0.0
    })
}

#[derive(Debug)]
struct TileCoords {
    combined_x:usize,
    combined_y:usize,
    combined_right_x:usize,
    combined_bottom_y:usize,
    tile_width:usize,
    tile_height:usize,
    
}

fn get_tile_coords(img:&rgbimage::RgbImage) -> error::Result<TileCoords> {
    let md = img.get_metadata().unwrap();
    let scale = md.scale_factor;
    if let Some(ref sf) = md.subframe_rect {
        if sf.len() == 4 {
            let tl_x = sf[0] / scale as f64;
            let tl_y = sf[1] / scale as f64;
            let right_x = (tl_x as f64 + sf[2] / scale as f64) as usize;
            let bottom_y = (tl_y as f64 + sf[3] / scale as f64) as usize;

            Ok(TileCoords{
                combined_x: tl_x as usize,
                combined_y: tl_y as usize,
                combined_right_x: right_x,
                combined_bottom_y: bottom_y,
                tile_width: (sf[2] / scale as f64) as usize,
                tile_height: (sf[3] / scale as f64) as usize
            })
        } else {
            Err("Tile lacks expected subrect data")
        }
    } else {
        Err("Tile lacks metadata")
    }
    
}

fn get_multiples_tl(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords) -> error::Result<Multiples> {

    get_multiples_at_point(&combined, &img, tc.combined_x, tc.combined_y, 0, 0)
}

fn get_multiples_bl(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords) -> error::Result<Multiples> {

    get_multiples_at_point(&combined, &img, tc.combined_x, tc.combined_bottom_y - 1, 0, tc.tile_height - 1)
}

fn get_multiples_br(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords) -> error::Result<Multiples> {

    get_multiples_at_point(&combined, &img, tc.combined_right_x, tc.combined_bottom_y - 1, tc.tile_width - 1, tc.tile_height - 1)
}

fn get_multiples_tr(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords) -> error::Result<Multiples> {

    get_multiples_at_point(&combined, &img, tc.combined_right_x, tc.combined_y, tc.tile_width - 1, 0)
}

fn is_top_valid(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords) -> bool {
    let a = get_multiples_tl(&combined, &img, &tc);
    let b = get_multiples_tr(&combined, &img, &tc);
    
    a.is_ok() && b.is_ok()
}

fn is_left_valid(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords) -> bool { 
    let a = get_multiples_tl(&combined, &img, &tc);
    let b = get_multiples_bl(&combined, &img, &tc);
    
    a.is_ok() && b.is_ok()
}

fn is_right_valid(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords) -> bool { 
    let a = get_multiples_tr(&combined, &img, &tc);
    let b = get_multiples_br(&combined, &img, &tc);
    
    a.is_ok() && b.is_ok()
}

fn is_bottom_valid(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords) -> bool { 
    let a = get_multiples_bl(&combined, &img, &tc);
    let b = get_multiples_br(&combined, &img, &tc);
    
    a.is_ok() && b.is_ok()
}


fn get_multiples_along_strip_horiz(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords, start_x:usize, start_y:usize) -> error::Result<Multiples> {

    let mut r = 0_f32;
    let mut g = 0_f32;
    let mut b = 0_f32;
    let mut c = 0_f32;

    for i in 0..tc.tile_width {
        let check_x = tc.combined_x + i;
        let check_y = tc.combined_y + start_y;
        match get_multiples_at_point(&combined, &img, check_x, check_y,  i, start_y) {
            Ok(m) => {
                if !m.r.is_nan() && !m.r.is_infinite() && !m.g.is_nan() && !m.g.is_infinite() && !m.b.is_nan() && !m.b.is_infinite() {
                    r = r + m.r;
                    g = g + m.g;
                    b = b + m.b;
                    c = c + 1.0;
                }
            },
            Err(_) => {}
        }
    }

    Ok(Multiples{
        r: r / c,
        g: g / c,
        b: b / c,
        c: c
    })
}

fn get_multiples_along_strip_vert(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage, tc:&TileCoords, start_x:usize, start_y:usize) -> error::Result<Multiples> {

    let mut r = 0_f32;
    let mut g = 0_f32;
    let mut b = 0_f32;
    let mut c = 0_f32;

    for i in 0..tc.tile_height {
        let check_x = tc.combined_x + start_x;
        let check_y = tc.combined_y + i;

        match get_multiples_at_point(&combined, &img, check_x, check_y, start_x, i) {
            Ok(m) => {
                r = r + m.r;
                g = g + m.g;
                b = b + m.b;
                c = c + 1.0;
            },
            Err(_) => {}
        }
    }

    Ok(Multiples{
        r: r / c,
        g: g / c,
        b: b / c,
        c: c
    })
}

fn get_multiples(combined:&rgbimage::RgbImage, img:&rgbimage::RgbImage) -> error::Result<Multiples> {

    match get_tile_coords(&img) {
        Ok(tc) => {
            vprintln!("Coords: {:?}", tc);
            if is_top_valid(&combined, &img, &tc) {
                vprintln!("Along Top");
                get_multiples_along_strip_horiz(&combined, &img, &tc, 0, 0)
            } else if is_left_valid(&combined, &img, &tc) {
                vprintln!("Along Left");
                get_multiples_along_strip_vert(&combined, &img, &tc, 0, 0)
            } else if is_right_valid(&combined, &img, &tc) {
                vprintln!("Along Right");
                get_multiples_along_strip_vert(&combined, &img, &tc, tc.tile_width - 1, 0)
            } else if is_bottom_valid(&combined, &img, &tc) {
                vprintln!("Along Bottom");
                get_multiples_along_strip_horiz(&combined, &img, &tc, 0, tc.tile_height - 1)
            } else {
                Err("No valid strip")
            }
        },
        Err(why) => {
            Err(why)
        }
    }

    // let md = img.get_metadata().unwrap();
    // let scale = md.scale_factor;
    // if let Some(ref sf) = md.subframe_rect {
    //     if sf.len() == 4 {
    //         get_multiples_tl(&combined, &img, sf, scale as f64)


    //     } else {
    //         Err("Tile lacks expected subrect data")
    //     }
    // } else {
    //     Err("Tile lacks metadata")
    // }
    
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
    let mut cnt = 0;
    for mut img in images {
        let md = img.get_metadata().unwrap();
        vprintln!("Pasting tile {}", md.imageid);
        if cnt > 0 {
            match get_multiples(&combined, &img) {
                Ok(m) => {
                    vprintln!("Applying weights: {:?}", m);
                    img.apply_weight(m.r, m.g, m.b).unwrap();
                },
                Err(why) => {
                    eprintln!("Unable to apply color adjustment on image: {}", why);
                }
            }
        }
        cnt += 1;

        
        
        if let Some(ref sf) = md.subframe_rect {
            if sf.len() == 4 {
                let tl_x = sf[0] / md.scale_factor as f64;
                let tl_y = sf[1] / md.scale_factor as f64;

                combined.paste(&img, tl_x as usize, tl_y as usize).unwrap();
            }
        }
        
    }


    vprintln!("Normalizing...");
    combined.normalize_to_16bit_with_max(255.0).unwrap();

    match combined.save(&output_file) {
        Err(why) => eprintln!("Error saving to output file: {}", why),
        Ok(_) => vprintln!("File saved.")
    };
}