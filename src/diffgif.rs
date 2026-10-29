#![allow(clippy::too_many_arguments)]

use anyhow::{anyhow, bail, Result};
use gif;
use sciimg::{enums::ImageMode, image, imagebuffer, lowpass, path};
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

fn image_mono_to_rgb(img: &image::Image) -> Result<image::Image> {
    let mut rgb_image = match image::Image::new_with_bands(img.width, img.height, 3, img.get_mode())
    {
        Ok(rgb_image) => rgb_image,
        Err(why) => return Err(anyhow!("Failed to create RGB image: {:?}", why)),
    };

    rgb_image.set_band(img.get_band(0), 0);
    rgb_image.set_band(img.get_band(0), 1);
    rgb_image.set_band(img.get_band(0), 2);

    Ok(rgb_image)
}

fn image_to_mono(img: &image::Image) -> Result<image::Image> {
    if img.num_bands() == 1 {
        return Err(anyhow!("Cannot convert image to mono: Already mono"));
    } else if img.num_bands() != 3 {
        return Err(anyhow!(
            "Cannot convert image to mono: Needs a three band RGB image"
        ));
    }

    let mut band_0_scaled = match img.get_band(0).scale(0.2125) {
        Ok(b) => b,
        Err(why) => return Err(anyhow!("Failed to scale band values: {:?}", why)),
    };

    let band_1_scaled = match img.get_band(1).scale(0.7154) {
        Ok(b) => b,
        Err(why) => return Err(anyhow!("Failed to scale band values: {:?}", why)),
    };

    let band_2_scaled = match img.get_band(2).scale(0.0721) {
        Ok(b) => b,
        Err(why) => return Err(anyhow!("Failed to scale band values: {:?}", why)),
    };

    band_0_scaled.add_mut(&band_1_scaled);
    band_0_scaled.add_mut(&band_2_scaled);

    match image::Image::new_from_buffer_mono(&band_0_scaled) {
        Ok(mono) => Ok(mono),
        Err(why) => Err(anyhow!(
            "Failed to create single band mono image: {:?}",
            why
        )),
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

fn generate_mean_stack(input_files: &[String]) -> Result<image::Image> {
    let mut mean: Option<image::Image> = None;
    let mut count: f32 = 0.0;
    info!("Creating mean stack of all input frames...");

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            info!("Adding file to stack: {}", in_file);

            let mut raw = image::Image::open(in_file)?;
            raw.normalize_between(0.0, 65535.0);

            if mean.is_none() {
                mean = Some(raw);
            } else if let Some(mean) = &mut mean {
                if raw.width != mean.width || raw.height != mean.height {
                    error!("Input image has differing dimensions, cannot continue");
                    return Err(anyhow!(
                        "Input image has differing dimensions, cannot continue"
                    ));
                }

                mean.add(&raw);
            }
            count += 1.0;
        } else {
            error!("File not found: {}", in_file);
            return Err(anyhow!("File not found: {}", in_file));
        }
    }

    if let Some(mean) = &mut mean {
        if count > 0.0 {
            mean.divide_into_each(count)
        } else {
            warn!("Encountering zero entries to mean stack!");
        }
    }

    if let Some(mean) = mean {
        Ok(mean)
    } else {
        Err(anyhow!("Did not generate image"))
    }
}

fn process_band(
    band: &imagebuffer::ImageBuffer,
    mean_band: &imagebuffer::ImageBuffer,
    black_level: f32,
    white_level: f32,
    gamma: f32,
    lowpass_window_size: u8,
    add_back_to_mean: bool,
    light_only: bool,
) -> Result<imagebuffer::ImageBuffer> {
    let diff = band.subtract(mean_band)?;

    let mut d = diff.clone();

    // Convert for absolute value difference
    for y in 0..d.height {
        for x in 0..d.width {
            let v = d.get(x, y);
            d.put(x, y, v.abs());
        }
    }

    let mm = mean_band.get_min_max();
    d.levels_with_gamma_mut(black_level, white_level, gamma);
    let mut n = d.normalize(mm.min, mm.max)?;

    for y in 0..d.height {
        for x in 0..d.width {
            let mult = match diff.get(x, y) >= 0.0 {
                true => 1.0,
                false => {
                    if light_only {
                        0.0
                    } else {
                        -1.0
                    }
                }
            };
            n.put(x, y, n.get(x, y) * mult);
        }
    }

    let blurred = match lowpass_window_size == 0 {
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
            let mut merged = mean_band.add(&blurred)?;
            merged.clip_mut(0.0, 65355.0);
            Ok(merged)
        }
        false => blurred.normalize(0.0, 65535.0),
    }
}

fn process_frame_mono(
    raw: &image::Image,
    mean_stack: &image::Image,
    black_level: f32,
    white_level: f32,
    gamma: f32,
    lowpass_window_size: u8,
    product_type: ProductType,
    light_only: bool,
) -> Result<image::Image> {
    let mono_raw = image_to_mono(raw)?;
    let mono_mean = image_to_mono(mean_stack)?;

    let mut processed_band = process_band(
        mono_raw.get_band(0),
        mono_mean.get_band(0),
        black_level,
        white_level,
        gamma,
        lowpass_window_size,
        product_type == ProductType::STANDARD,
        light_only,
    )?;

    processed_band.normalize_force_minmax_mut(0.0, 255.0, 0.0, 65535.0);

    image::Image::new_from_buffer_mono(&processed_band)
}

fn process_frame_3channel(
    raw: &image::Image,
    mean_stack: &image::Image,
    black_level: f32,
    white_level: f32,
    gamma: f32,
    lowpass_window_size: u8,
    product_type: ProductType,
    light_only: bool,
) -> Result<image::Image> {
    let mut processed_band_0 = process_band(
        raw.get_band(0),
        mean_stack.get_band(0),
        black_level,
        white_level,
        gamma,
        lowpass_window_size,
        product_type == ProductType::STANDARD,
        light_only,
    )?;
    let mut processed_band_1 = process_band(
        raw.get_band(1),
        mean_stack.get_band(1),
        black_level,
        white_level,
        gamma,
        lowpass_window_size,
        product_type == ProductType::STANDARD,
        light_only,
    )?;
    let mut processed_band_2 = process_band(
        raw.get_band(2),
        mean_stack.get_band(2),
        black_level,
        white_level,
        gamma,
        lowpass_window_size,
        product_type == ProductType::STANDARD,
        light_only,
    )?;

    processed_band_0.normalize_force_minmax_mut(0.0, 255.0, 0.0, 65535.0);
    processed_band_1.normalize_force_minmax_mut(0.0, 255.0, 0.0, 65535.0);
    processed_band_2.normalize_force_minmax_mut(0.0, 255.0, 0.0, 65535.0);

    image::Image::new_from_buffers_rgb(
        &processed_band_0,
        &processed_band_1,
        &processed_band_2,
        ImageMode::U16BIT,
    )
}

fn process_frame(
    raw: &image::Image,
    mean_stack: &image::Image,
    black_level: f32,
    white_level: f32,
    gamma: f32,
    lowpass_window_size: u8,
    product_type: ProductType,
    convert_to_mono: bool,
    light_only: bool,
) -> Result<image::Image> {
    match convert_to_mono {
        true => process_frame_mono(
            raw,
            mean_stack,
            black_level,
            white_level,
            gamma,
            lowpass_window_size,
            product_type,
            light_only,
        ),
        false => process_frame_3channel(
            raw,
            mean_stack,
            black_level,
            white_level,
            gamma,
            lowpass_window_size,
            product_type,
            light_only,
        ),
    }
}

fn process_file(
    in_file: &String,
    mean_stack: &image::Image,
    black_level: f32,
    white_level: f32,
    gamma: f32,
    lowpass_window_size: u8,
    product_type: ProductType,
    convert_to_mono: bool,
    light_only: bool,
) -> Result<image::Image> {
    info!("Processing frame differential on file: {}", in_file);

    let mut raw = image::Image::open(in_file)?;
    raw.normalize_between(0.0, 65535.0);
    match product_type {
        ProductType::STACKED => {
            let img_std = process_frame(
                &raw,
                mean_stack,
                black_level,
                white_level,
                gamma,
                lowpass_window_size,
                ProductType::STANDARD,
                convert_to_mono,
                light_only,
            )?;
            let img_diff = process_frame(
                &raw,
                mean_stack,
                black_level,
                white_level,
                gamma,
                lowpass_window_size,
                ProductType::DIFFERENTIAL,
                convert_to_mono,
                light_only,
            )?;
            let mut stacked = image::Image::new_with_bands(
                img_std.width,
                img_std.height * 2,
                match convert_to_mono {
                    true => 1,
                    false => 3,
                },
                ImageMode::U16BIT,
            )?;
            stacked.paste(&img_diff, 0, 0);
            stacked.paste(&img_std, 0, img_std.height);

            Ok(stacked)
        }
        _ => process_frame(
            &raw,
            mean_stack,
            black_level,
            white_level,
            gamma,
            lowpass_window_size,
            product_type,
            convert_to_mono,
            light_only,
        ),
    }
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
    pub light_only: bool,
}

pub fn process(params: &DiffGif) -> Result<()> {
    let mean_stack = generate_mean_stack(&params.input_files)?;

    let gif_height = match params.product_type {
        ProductType::STACKED => mean_stack.height * 2,
        _ => mean_stack.height,
    };

    let gif_width = mean_stack.width;

    let mut image_output = match File::create(&params.output) {
        Ok(image_output) => image_output,
        Err(why) => return Err(anyhow!("{:?}", why)),
    };

    let mut encoder =
        match gif::Encoder::new(&mut image_output, gif_width as u16, gif_height as u16, &[]) {
            Ok(encoder) => encoder,
            Err(why) => return Err(anyhow!("Error creating gif encoder: {:?}", why)),
        };

    if let Err(why) = encoder.set_repeat(gif::Repeat::Infinite) {
        return Err(anyhow!("Failed to set gif repeat option: {:?}", why));
    }

    // Process each frame...
    params.input_files.iter().try_for_each(|file_path| {
        if !(path::file_exists(file_path) && path::file_readable(file_path)) {
            bail!("Input file not found: {:?}", file_path);
        }

        let processed_frame = match process_file(
            file_path,
            &mean_stack,
            params.black_level,
            params.white_level,
            params.gamma,
            params.lowpass_window_size,
            params.product_type,
            params.convert_to_mono,
            params.light_only,
        ) {
            Ok(frame) => frame,
            Err(why) => {
                bail!("Failed to process file {} with error: {:?}", file_path, why);
            }
        };

        let final_img = match params.convert_to_mono && processed_frame.num_bands() != 3 {
            true => image_mono_to_rgb(&processed_frame)?,
            false => processed_frame,
        };

        let pixels = rgbimage_to_vec_v8(&final_img);

        let mut frame = gif::Frame::from_rgb(gif_width as u16, gif_height as u16, &pixels);

        frame.delay = params.delay;
        if let Err(why) = encoder.write_frame(&frame) {
            bail!("Failed to write gif frame: {:?}", why);
        }
        Ok(())
    })?;

    Ok(())
}
