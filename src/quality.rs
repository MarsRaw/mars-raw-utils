
use sciimg::{
    stats,
    imagebuffer,
    rgbimage,
    lowpass
};

fn apply_blur(image:&imagebuffer::ImageBuffer, amount:usize) -> imagebuffer::ImageBuffer {
    lowpass::lowpass_imagebuffer(&image, amount)
}

fn isolate_window(buffer:&imagebuffer::ImageBuffer, window_size:usize, x:usize, y:usize) -> Vec<f32> {
    let mut v:Vec<f32> = Vec::with_capacity(window_size * window_size);
    let start = window_size as i32 / 2 * -1;
    let end = window_size as i32 / 2 + 1;
    for _y in start..end as i32 {
        for _x in start..end as i32 {
            let get_x = x as i32 + _x;
            let get_y = y as i32 + _y;
            if get_x >= 0 
                && get_x < buffer.width as i32 
                && get_y >= 0 
                && get_y < buffer.height as i32
                && buffer.get_mask_at_point(get_x as usize, get_y as usize).unwrap()
                {
                v.push(buffer.get(get_x as usize, get_y as usize).unwrap());
            }
        }
    }
    v
}

pub fn get_point_quality_estimation_on_diff_buffer(diff:&imagebuffer::ImageBuffer, window_size:usize, x: usize, y: usize) -> f32 {
    let window = isolate_window(&diff, window_size, x, y);
    match stats::std_deviation(&window) {
        Some(sd) => sd,
        None => 0.0
    }
}


pub fn get_point_quality_estimation_on_buffer(image:&imagebuffer::ImageBuffer, window_size:usize, x: usize, y: usize) -> f32 {
    let blurred = apply_blur(&image, 5);
    let diff = blurred.subtract(&image).unwrap();
    get_point_quality_estimation_on_diff_buffer(&diff, window_size, x, y)
}

pub fn get_point_quality_estimation(image:&rgbimage::RgbImage, window_size:usize, x: usize, y: usize) -> f32 {
    let mut q:Vec<f32> = vec!();
    for b in 0..image.num_bands() {
        let band = image.get_band(b);
        q.push(get_point_quality_estimation_on_buffer(&band, window_size, x, y));
    }
    match stats::mean(&q) {
        Some(m) => m,
        None => 0.0
    }
}

pub fn get_quality_estimation_on_buffer(image:&imagebuffer::ImageBuffer) -> f32 {
    let blurred = apply_blur(&image, 5);
    let diff = blurred.subtract(&image).unwrap();
    match stats::std_deviation(&diff.buffer[..]) {
        Some(sd) => sd,
        None => 0.0
    }
}

// A very simple image sharpness quantifier that computes the standard deviation of the difference between
// an image and a blurred copy.
pub fn get_quality_estimation(image:&rgbimage::RgbImage) -> f32 {
    let mut q:Vec<f32> = vec!();
    for b in 0..image.num_bands() {
        let band = image.get_band(b);
        q.push(get_quality_estimation_on_buffer(&band));
    }
    match stats::mean(&q) {
        Some(m) => m,
        None => 0.0
    }
}