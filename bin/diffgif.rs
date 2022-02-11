use mars_raw_utils::{
    constants, 
    print, 
    vprintln, 
    path,
    util
};

use sciimg::{
    rgbimage,
    imagebuffer
};

use gif;
use gif::{Frame, Encoder, Repeat};

use std::fs::File;

#[macro_use]
extern crate clap;

use clap::{Arg, App};
use std::process;


fn imagebuffer_to_vec_v8(buff:&imagebuffer::ImageBuffer) -> Vec<u8> {
    let mut f : Vec<u8> = vec!(0; buff.width * buff.height * 3);
    for y in 0..buff.height {
        for x in 0..buff.width {
            let idx = (y * buff.width + x) * 3;
            f[idx + 0] = buff.get(x, y).unwrap().round() as u8;
            f[idx + 1] = f[idx + 0];
            f[idx + 2] = f[idx + 0];
        }
    }

    f
}

fn rgbimage_to_vec_u8(inp_img:&rgbimage::RgbImage) -> Vec<u8> {
    let mut f : Vec<u8> = vec!(0; inp_img.width * inp_img.height * inp_img.num_bands());

    
    for y in 0..inp_img.height {
        for x in 0..inp_img.width {
            let idx = (y * inp_img.width + x) * inp_img.num_bands();
            for b in 0..inp_img.num_bands() {
                let i = idx + b;
                let value = (inp_img.get_band(b).get(x, y).unwrap()).round() as u8;
                f[i] = value;
            }
        }
    }


    f
}

fn generate_mean_stack(input_files:&Vec<&str>) -> rgbimage::RgbImage {

    let mut mean : rgbimage::RgbImage = rgbimage::RgbImage::new_empty().unwrap();
    let mut count : imagebuffer::ImageBuffer = imagebuffer::ImageBuffer::new_empty().unwrap();
    let mut ones : imagebuffer::ImageBuffer = imagebuffer::ImageBuffer::new_empty().unwrap();

    vprintln!("Creating mean stack of all input frames...");

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            
            let raw = rgbimage::RgbImage::open(&String::from(*in_file)).unwrap();

            if mean.is_empty() {
                mean = raw;
                count = imagebuffer::ImageBuffer::new(mean.width, mean.height).unwrap();
                ones = imagebuffer::ImageBuffer::new_with_fill(mean.width, mean.height, 1.0).unwrap();
            } else {

                if raw.width != mean.width || raw.height != mean.height {
                    eprintln!("Input image has differing dimensions, cannot continue");
                    process::exit(1);
                }

                mean.add(&raw);
            }

            count = count.add(&ones).unwrap();
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

    if !mean.is_empty() {
        mean.divide_from_each(&count);
    }

    mean
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
                    .arg(Arg::with_name(constants::param::PARAM_LEVELS_BLACK_LEVEL)
                        .short(constants::param::PARAM_LEVELS_BLACK_LEVEL_SHORT)
                        .long(constants::param::PARAM_LEVELS_BLACK_LEVEL)
                        .value_name("BLACK_LEVEL")
                        .help("Black level")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_LEVELS_WHITE_LEVEL)
                        .short(constants::param::PARAM_LEVELS_WHITE_LEVEL_SHORT)
                        .long(constants::param::PARAM_LEVELS_WHITE_LEVEL)
                        .value_name("WHITE_LEVEL")
                        .help("White level")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_DELAY)
                        .short(constants::param::PARAM_DELAY_SHORT)
                        .long(constants::param::PARAM_DELAY)
                        .value_name("PARAM_DELAY")
                        .help("Interframe Delay")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_GAMMA)
                        .short(constants::param::PARAM_GAMMA_SHORT)
                        .long(constants::param::PARAM_GAMMA)
                        .value_name("PARAM_GAMMA")
                        .help("Gamma")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("OUTPUT")
                        .help("Output")
                        .required(true)
                        .takes_value(true))     
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let black_level : f32 = match matches.is_present(constants::param::PARAM_LEVELS_BLACK_LEVEL) {
        true => {
            let s = matches.value_of(constants::param::PARAM_LEVELS_BLACK_LEVEL).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for black level");
                process::exit(1);
            }
        },
        false => {
            0.0
        }
    };



    let white_level : f32 = match matches.is_present(constants::param::PARAM_LEVELS_WHITE_LEVEL) {
        true => {
            let s = matches.value_of(constants::param::PARAM_LEVELS_WHITE_LEVEL).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for white level");
                process::exit(1);
            }
        },
        false => {
            1.0
        }
    };

    let gamma : f32 = match matches.is_present(constants::param::PARAM_GAMMA) {
        true => {
            let s = matches.value_of(constants::param::PARAM_GAMMA).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for gamma");
                process::exit(1);
            }
        },
        false => {
            1.0
        }
    };

    let delay : f32 = match matches.is_present(constants::param::PARAM_DELAY) {
        true => {
            let s = matches.value_of(constants::param::PARAM_DELAY).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for interframe delay");
                process::exit(1);
            }
        },
        false => {
            0.2
        }
    };

    let output = matches.value_of("output").unwrap();

    // Some rules on the parameters
    // TODO: Keep an eye on floating point errors
    if white_level < 0.0 || black_level < 0.0{
        eprintln!("Levels cannot be negative");
        process::exit(1);
    }

    if white_level < black_level {
        eprintln!("White level cannot be less than black level");
        process::exit(1);
    }

    if white_level > 1.0 || black_level > 1.0 {
        eprintln!("Levels cannot exceed 1.0");
        process::exit(1);
    }

    if gamma <= 0.0 {
        eprintln!("Gamma cannot be zero or negative");
        process::exit(1);
    }

    if delay < 0.0 {
        eprintln!("Interframe delay cannot be negative");
        process::exit(1);
    }

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    let mut mean_stack = generate_mean_stack(&input_files);
    let (mm_min, mm_max) = mean_stack.get_min_max_all_channel();
    vprintln!("Min: {}, Max: {}", mm_min, mm_max);
    //mean_stack.normalize_16bit_to_8bit();
    mean_stack.save("test.png");


    //let mut first_pixels: Vec<u8> = imagebuffer_to_vec_v8(&mean_stack.get_band(0));
    //let first_frame = gif::Frame::from_rgb(mean_stack.width as u16, mean_stack.height as u16, &mut *first_pixels);

    let mut image = File::create(output).unwrap();
    let mut encoder = gif::Encoder::new(&mut image, mean_stack.width as u16, mean_stack.height as u16, &[]).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();
    
    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);

            let raw = rgbimage::RgbImage::open(&String::from(*in_file)).unwrap();

            let b0 = raw.get_band(0);
            let m0 = mean_stack.get_band(0);
            
            let mut d = b0.subtract(m0).unwrap();
            let mm = d.get_min_max().unwrap();
            let norm_min = (255.0 * black_level) + mm.min;
            let norm_max = (255.0 * white_level) + mm.min;

            d.clip_mut(norm_min, norm_max);
            d.power_mut(gamma);
            let n = d.normalize(mm.min, mm.max).unwrap();

            // TODO:
            // _ Absolute difference
            // _ Add back to the mean
            // _ Multiband (RGB)
            // _ Refactor for cleanliness

            let mut pixels = imagebuffer_to_vec_v8(&n);
            let frame = gif::Frame::from_rgb_speed(raw.width as u16, raw.height as u16, &mut *pixels, 10);
            encoder.write_frame(&frame).unwrap();
            //process_file(in_file, black_level, white_level, gamma);
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

    
}