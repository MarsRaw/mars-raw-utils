use mars_raw_utils::{
    constants, 
    print, 
    vprintln, 
    path,
    util
};

use sciimg::{
    rgbimage,
    imagebuffer,
    blur
};

use gif;

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

fn generate_mean_stack(input_files:&Vec<&str>) -> rgbimage::RgbImage {

    let mut mean : rgbimage::RgbImage = rgbimage::RgbImage::new_empty().unwrap();
    let mut count : imagebuffer::ImageBuffer = imagebuffer::ImageBuffer::new_empty().unwrap();
    let mut ones : imagebuffer::ImageBuffer = imagebuffer::ImageBuffer::new_empty().unwrap();

    vprintln!("Creating mean stack of all input frames...");

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Adding file to stack: {}", in_file);
            
            let raw = rgbimage::RgbImage::open16(&String::from(*in_file)).unwrap();

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
                        .help("Interframe delay in increments of 10ms")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_GAMMA)
                        .short(constants::param::PARAM_GAMMA_SHORT)
                        .long(constants::param::PARAM_GAMMA)
                        .value_name("PARAM_GAMMA")
                        .help("Gamma")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_BLUR)
                        .short(constants::param::PARAM_BLUR_SHORT)
                        .long(constants::param::PARAM_BLUR)
                        .value_name("PARAM_BLUR")
                        .help("Gaussian blur kernel size on differential output")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
                        .short(constants::param::PARAM_OUTPUT_SHORT)
                        .long(constants::param::PARAM_OUTPUT)
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
                s.parse::<f32>().unwrap() / 100.0
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
                s.parse::<f32>().unwrap() / 100.0
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

    let delay : u16 = match matches.is_present(constants::param::PARAM_DELAY) {
        true => {
            let s = matches.value_of(constants::param::PARAM_DELAY).unwrap();
            if util::string_is_valid_u16(&s) {
                s.parse::<u16>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for interframe delay");
                process::exit(1);
            }
        },
        false => {
            10
        }
    };

    let blur_kernel_size : f32 = match matches.is_present(constants::param::PARAM_BLUR) {
        true => {
            let s = matches.value_of(constants::param::PARAM_BLUR).unwrap();
            if util::string_is_valid_f32(&s) {
                s.parse::<f32>().unwrap()
            } else {
                eprintln!("Error: Invalid number specified for blur kernel size");
                process::exit(1);
            }
        },
        false => {
            0.0
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

    // if white_level > 1.0 || black_level > 1.0 {
    //     eprintln!("Levels cannot exceed 1.0");
    //     process::exit(1);
    // }

    if gamma <= 0.0 {
        eprintln!("Gamma cannot be zero or negative");
        process::exit(1);
    }

    if blur_kernel_size < 0.0 {
        eprintln!("Blur kernel size cannot be negative");
        process::exit(1);
    }

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();

    let mean_stack = generate_mean_stack(&input_files);

    let mut image = File::create(output).unwrap();
    let mut encoder = gif::Encoder::new(&mut image, mean_stack.width as u16, mean_stack.height as u16, &[]).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();
    
    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing frame differential on file: {}", in_file);

            let raw = rgbimage::RgbImage::open16(&String::from(*in_file)).unwrap();

            let b0 = raw.get_band(0);
            let m0 = mean_stack.get_band(0);
            
            let diff = b0.subtract(m0).unwrap();
            let mut d = diff.clone();

            // Convert for absolute value difference
            for y in 0..d.height {
                for x in 0..d.width {
                    let v = d.get(x, y).unwrap();
                    d.put(x, y, v.abs());
                }
            }

            let mm = d.get_min_max().unwrap();
            let rng = 65535.0;
            let norm_min = (rng * black_level) + mm.min;
            let norm_max = (rng * white_level) + mm.min;
            
            d.clip_mut(norm_min, norm_max);
            d.power_mut(gamma);

            let mut n = d.normalize(0.0, 65535.0).unwrap();

            for y in 0..d.height {
                for x in 0..d.width {
                    let mult = match diff.get(x, y).unwrap() >= 0.0 {
                        true => 1.0,
                        false => -1.0
                    };
                    n.put(x, y, n.get(x, y).unwrap() * mult);
                }
            }

            let blurred = match blur_kernel_size == 0.0 {
                true => n.clone(),
                false => {
                    // This method is lossy. Get over it.
                    // So if we're dealing with negative numbers here, we
                    // will need to scale them to within range of a u16.
                    // To do that, we will scale all values by half, then
                    // add the absolute value of the lowest value. 
                    // Then do the blur
                    // Then undo that offset and scaling. 
                    // We lose precision by about half
                    
                    let mnmx = n.get_min_max().unwrap();
                    let init_mn = mnmx.min;
                    if init_mn < 0.0 {
                        n.scale_mut(0.5);
                        n.add_across_mut(init_mn.abs() * 0.5);
                    }
                    
                    let mut b = blur::blur_imagebuffer(&n, blur_kernel_size);

                    if init_mn < 0.0 {
                        b.subtract_across_mut(init_mn.abs() * 0.5);
                        b.scale_mut(2.0);
                    }
                    
                    b
                }
            };


            

            let mut merged = m0.add(&blurred).unwrap();
            merged.clip_mut(0.0, 65355.0);

            // TODO:
            // _ Absolute difference
            // _ Add back to the mean
            // _ Multiband (RGB)
            // _ Refactor for cleanliness

            merged.normalize_mut(0.0, 255.0);
            let mut pixels = imagebuffer_to_vec_v8(&merged);
            let mut frame = gif::Frame::from_rgb(raw.width as u16, raw.height as u16, &mut *pixels);
            frame.delay = delay;
            encoder.write_frame(&frame).unwrap();
            
        } else {
            eprintln!("File not found: {}", in_file);
        }
    }

    
}