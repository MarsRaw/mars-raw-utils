#![allow(clippy::too_many_arguments)]

use sciimg::{enums::ImageMode, image, imagebuffer, lowpass, path};

use gif;

use std::fs::File;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProductType {
    STANDARD,
    DIFFERENTIAL,
    STACKED,
}

impl FromStr for ProductType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match ProductType::_from_str(s) {
            None => Err("Invalid product type"),
            Some(t) => Ok(t),
        }
    }
}

impl ProductType {
    pub fn _from_str(s: &str) -> Option<ProductType> {
        match s {
            "std" => Some(ProductType::STANDARD),
            "diff" => Some(ProductType::DIFFERENTIAL),
            "stacked" => Some(ProductType::STACKED),
            _ => None,
        }
    }
}

fn imagebuffer_to_vec_v8(
    buff_0: &imagebuffer::ImageBuffer,
    buff_1: &imagebuffer::ImageBuffer,
    buff_2: &imagebuffer::ImageBuffer,
) -> Vec<u8> {
    let mut f: Vec<u8> = vec![0; buff_0.width * buff_0.height * 3];
    for y in 0..buff_0.height {
        for x in 0..buff_0.width {
            let idx = (y * buff_0.width + x) * 3;
            f[idx] = buff_0.get(x, y).round() as u8;
            f[idx + 1] = buff_1.get(x, y).round() as u8;
            f[idx + 2] = buff_2.get(x, y).round() as u8;
        }
    }

    f
}

fn rgbimage_to_vec_v8(img3band: &image::Image) -> Vec<u8> {
    let b0 = img3band.get_band(0);
    let b1 = img3band.get_band(1);
    let b2 = img3band.get_band(2);
    imagebuffer_to_vec_v8(b0, b1, b2)
}

fn generate_mean_stack(input_files: &[String]) -> image::Image {
    let mut mean: image::Image = image::Image::new_empty().unwrap();
    let mut count: imagebuffer::ImageBuffer = imagebuffer::ImageBuffer::new_empty().unwrap();
    let mut ones: imagebuffer::ImageBuffer = imagebuffer::ImageBuffer::new_empty().unwrap();

    vprintln!("Creating mean stack of all input frames...");

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Adding file to stack: {}", in_file);

            let raw = image::Image::open(in_file).unwrap();

            if mean.is_empty() {
                mean = raw;
                count = imagebuffer::ImageBuffer::new(mean.width, mean.height).unwrap();
                ones =
                    imagebuffer::ImageBuffer::new_with_fill(mean.width, mean.height, 1.0).unwrap();
            } else {
                if raw.width != mean.width || raw.height != mean.height {
                    eprintln!("Input image has differing dimensions, cannot continue");
                    panic!("Input image has differing dimensions, cannot continue");
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

fn process_band(
    band: &imagebuffer::ImageBuffer,
    mean_band: &imagebuffer::ImageBuffer,
    black_level: f32,
    white_level: f32,
    gamma: f32,
    lowpass_window_size: u8,
    add_back_to_mean: bool,
) -> imagebuffer::ImageBuffer {
    let diff = band.subtract(mean_band).unwrap();
    let mut d = diff.clone();

    // Convert for absolute value difference
    for y in 0..d.height {
        for x in 0..d.width {
            let v = d.get(x, y);
            d.put(x, y, v.abs());
        }
    }

    let mm = d.get_min_max();
    let rng = 65535.0;
    let norm_min = (rng * black_level) + mm.min;
    let norm_max = (rng * white_level) + mm.min;

    d.clip_mut(norm_min, norm_max);
    d.power_mut(gamma);

    let mut n = d.normalize(0.0, 65535.0).unwrap();

    for y in 0..d.height {
        for x in 0..d.width {
            let mult = match diff.get(x, y) >= 0.0 {
                true => 1.0,
                false => -1.0,
            };
            n.put(x, y, n.get(x, y) * mult);
        }
    }

    let mut blurred = match lowpass_window_size == 0 {
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

            let mnmx = n.get_min_max();
            let init_mn = mnmx.min;
            if init_mn < 0.0 {
                n.scale_mut(0.5);
                n.add_across_mut(init_mn.abs() * 0.5);
            }

            //let mut b = blur::blur_imagebuffer(&n, blur_kernel_size);
            let mut b = lowpass::lowpass_imagebuffer(&n, lowpass_window_size as usize);

            if init_mn < 0.0 {
                b.subtract_across_mut(init_mn.abs() * 0.5);
                b.scale_mut(2.0);
            }

            b
        }
    };

    match add_back_to_mean {
        true => {
            let mut merged = mean_band.add(&blurred).unwrap();
            merged.clip_mut(0.0, 65355.0);
            merged
        }
        false => {
            let mnmx = blurred.get_min_max();

            if mnmx.min.abs() < mnmx.max.abs() {
                blurred.clip_mut(mnmx.min, mnmx.min.abs());
            } else {
                blurred.clip_mut(-1.0 * mnmx.max, mnmx.max);
            }

            blurred.add_across_mut(32767.0);
            blurred.clip(0.0, 65355.0).unwrap()
        }
    }
}

fn process_frame_3channel(
    raw: &image::Image,
    mean_stack: &image::Image,
    black_level: f32,
    white_level: f32,
    gamma: f32,
    lowpass_window_size: u8,
    product_type: ProductType,
    convert_to_mono: bool,
) -> image::Image {
    let mut processed_band_0 = process_band(
        raw.get_band(0),
        mean_stack.get_band(0),
        black_level,
        white_level,
        gamma,
        lowpass_window_size,
        product_type == ProductType::STANDARD,
    );
    let mut processed_band_1 = process_band(
        raw.get_band(1),
        mean_stack.get_band(1),
        black_level,
        white_level,
        gamma,
        lowpass_window_size,
        product_type == ProductType::STANDARD,
    );
    let mut processed_band_2 = process_band(
        raw.get_band(2),
        mean_stack.get_band(2),
        black_level,
        white_level,
        gamma,
        lowpass_window_size,
        product_type == ProductType::STANDARD,
    );

    processed_band_0.normalize_force_minmax_mut(0.0, 255.0, 0.0, 65535.0);
    processed_band_1.normalize_force_minmax_mut(0.0, 255.0, 0.0, 65535.0);
    processed_band_2.normalize_force_minmax_mut(0.0, 255.0, 0.0, 65535.0);

    if convert_to_mono {
        processed_band_0.scale_mut(0.2125);
        processed_band_1.scale_mut(0.7154);
        processed_band_2.scale_mut(0.0721);
        processed_band_0 = processed_band_0
            .add(&processed_band_1)
            .unwrap()
            .add(&processed_band_2)
            .unwrap();
        processed_band_1 = processed_band_0.clone();
        processed_band_2 = processed_band_0.clone();
    }

    image::Image::new_from_buffers_rgb(
        &processed_band_0,
        &processed_band_1,
        &processed_band_2,
        ImageMode::U16BIT,
    )
    .unwrap()
}

fn process_file(
    encoder: &mut gif::Encoder<&mut std::fs::File>,
    in_file: &String,
    mean_stack: &image::Image,
    black_level: f32,
    white_level: f32,
    gamma: f32,
    lowpass_window_size: u8,
    delay: u16,
    product_type: ProductType,
    convert_to_mono: bool,
) {
    vprintln!("Processing frame differential on file: {}", in_file);

    let raw = image::Image::open(in_file).unwrap();

    let (pixels, height) = match product_type {
        ProductType::STACKED => {
            let img_std = process_frame_3channel(
                &raw,
                mean_stack,
                black_level,
                white_level,
                gamma,
                lowpass_window_size,
                ProductType::STANDARD,
                convert_to_mono,
            );
            let img_diff = process_frame_3channel(
                &raw,
                mean_stack,
                black_level,
                white_level,
                gamma,
                lowpass_window_size,
                ProductType::DIFFERENTIAL,
                convert_to_mono,
            );
            let mut stacked = image::Image::new_with_bands(
                img_std.width,
                img_std.height * 2,
                3,
                ImageMode::U16BIT,
            )
            .unwrap();
            stacked.paste(&img_diff, 0, 0);
            stacked.paste(&img_std, 0, img_std.height);
            (rgbimage_to_vec_v8(&stacked), img_std.height * 2)
        }
        _ => {
            let img = process_frame_3channel(
                &raw,
                mean_stack,
                black_level,
                white_level,
                gamma,
                lowpass_window_size,
                product_type,
                convert_to_mono,
            );
            (rgbimage_to_vec_v8(&img), img.height)
        }
    };

    let mut frame = gif::Frame::from_rgb(raw.width as u16, height as u16, &pixels);

    frame.delay = delay;
    encoder.write_frame(&frame).unwrap();
}

pub struct DiffGif {
    pub input_files: Vec<String>,
    pub product_type: ProductType,
    pub output: String,
    pub black_level: f32,
    pub white_level: f32,
    pub gamma: f32,
    pub delay: u16,
    pub lowpass_window_size: u8,
    pub convert_to_mono: bool,
}

pub fn process(params: &DiffGif) {
    let mean_stack = generate_mean_stack(&params.input_files);
    let height = match params.product_type {
        ProductType::STACKED => mean_stack.height * 2,
        _ => mean_stack.height,
    };

    let mut image = File::create(&params.output).unwrap();
    let mut encoder =
        gif::Encoder::new(&mut image, mean_stack.width as u16, height as u16, &[]).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    for in_file in params.input_files.iter() {
        if path::file_exists(in_file) {
            process_file(
                &mut encoder,
                in_file,
                &mean_stack,
                params.black_level,
                params.white_level,
                params.gamma,
                params.lowpass_window_size,
                params.delay,
                params.product_type,
                params.convert_to_mono,
            );
        } else {
            eprintln!("File not found: {}", in_file);
            panic!("File not found");
        }
    }
}
