use std::process;

use crate::{path, util, vprintln};

use sciimg::{imagebuffer, lowpass, quality, rgbimage, stats};

use colored::*;

struct Diff {
    band_0: imagebuffer::ImageBuffer,
    band_1: imagebuffer::ImageBuffer,
    band_2: imagebuffer::ImageBuffer,
    image: rgbimage::RgbImage,
}

fn make_diff_for_band(
    buffer: &imagebuffer::ImageBuffer,
    amount: usize,
) -> imagebuffer::ImageBuffer {
    let blurred = lowpass::lowpass_imagebuffer(buffer, amount);
    blurred.subtract(buffer).unwrap()
}

fn make_diff_container(image: &rgbimage::RgbImage, blur_amount: usize) -> Diff {
    Diff {
        band_0: make_diff_for_band(image.get_band(0), blur_amount),
        band_1: make_diff_for_band(image.get_band(1), blur_amount),
        band_2: make_diff_for_band(image.get_band(2), blur_amount),
        image: image.clone(),
    }
}

pub fn focusmerge(
    input_files: &Vec<String>,
    quality_window_size: usize,
    depth_map: bool,
    output_file: &str,
) {
    let mut images: Vec<Diff> = vec![];

    let mut out_width = 0;
    let mut out_height = 0;

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);

            let image = rgbimage::RgbImage::open16(in_file).unwrap();

            if out_width == 0 {
                out_width = image.width;
                out_height = image.height;
            } else if out_width != image.width || out_height != image.height {
                eprintln!(
                    "{} Input images have differing dimensions. Cannot continue",
                    "Error".red()
                );
                process::exit(1);
            }

            images.push(make_diff_container(&image, 5));
        } else {
            eprintln!("File not found: {}", in_file);
            panic!("File not found");
        }
    }

    let mut depth_map_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(
        images[0].band_0.width,
        images[0].band_0.height,
        0.0,
        images[0].band_0.mode,
    )
    .unwrap();

    let mut b0_merge_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(
        images[0].band_0.width,
        images[0].band_0.height,
        0.0,
        images[0].band_0.mode,
    )
    .unwrap();
    let mut b1_merge_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(
        images[0].band_0.width,
        images[0].band_0.height,
        0.0,
        images[0].band_0.mode,
    )
    .unwrap();
    let mut b2_merge_buffer = imagebuffer::ImageBuffer::new_with_fill_as_mode(
        images[0].band_0.width,
        images[0].band_0.height,
        0.0,
        images[0].band_0.mode,
    )
    .unwrap();

    // Super mega inefficient. This'll take a few minutes to run.
    for y in 0..b0_merge_buffer.height {
        vprintln!(
            "Row {} of {}. {}%",
            y,
            b0_merge_buffer.height,
            (y as f32 / b0_merge_buffer.height as f32 * 100.0)
        );
        for x in 0..b0_merge_buffer.width {
            let mut b0_value = 0.0_f32;
            let mut b1_value = 0.0_f32;
            let mut b2_value = 0.0_f32;
            let mut max_quality = 0.0_f32;
            let mut depth_value = 0;

            for image_num in 0..images.len() {
                let image: &Diff = &images[image_num];

                let q0 = quality::get_point_quality_estimation_on_diff_buffer(
                    &image.band_0,
                    quality_window_size,
                    x,
                    y,
                );
                let q1 = quality::get_point_quality_estimation_on_diff_buffer(
                    &image.band_1,
                    quality_window_size,
                    x,
                    y,
                );
                let q2 = quality::get_point_quality_estimation_on_diff_buffer(
                    &image.band_2,
                    quality_window_size,
                    x,
                    y,
                );
                let q = stats::mean(&[q0, q1, q2]).unwrap_or(0.0);

                if q > max_quality {
                    depth_value = image_num;
                    max_quality = q;
                    b0_value = image.image.get_band(0).get(x, y).unwrap();
                    b1_value = image.image.get_band(1).get(x, y).unwrap();
                    b2_value = image.image.get_band(2).get(x, y).unwrap();
                }
            }

            depth_map_buffer.put(x, y, depth_value as f32);
            b0_merge_buffer.put(x, y, b0_value);
            b1_merge_buffer.put(x, y, b1_value);
            b2_merge_buffer.put(x, y, b2_value);
        }
    }

    let merge_buffer = rgbimage::RgbImage::new_from_buffers_rgb(
        &b0_merge_buffer,
        &b1_merge_buffer,
        &b2_merge_buffer,
        b0_merge_buffer.mode,
    )
    .unwrap();

    merge_buffer.save(output_file);

    if depth_map {
        depth_map_buffer = depth_map_buffer.normalize(0.0, 65535.0).unwrap();
        let depth_map_out_file = util::append_file_name(output_file, "depth");
        depth_map_buffer.save_16bit(&depth_map_out_file);
    }
}
