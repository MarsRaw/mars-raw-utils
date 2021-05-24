/*
    Attempt at hot pixel detection and removal. 

    Method:
        For each pixel (excluding image border pixels):
            1. Compute the standard deviation of a window of pixels (3x3, say)
            2. Compute the z-score for the target pixel
            3. If the z-score exceeds a threshold variance from the mean
               we replace the pixel value with a median filter
*/

use crate::{
    error, 
    imagebuffer::ImageBuffer,
    stats
};

#[allow(dead_code)]
pub struct ReplacedPixel {
    x : usize,
    y : usize,
    pixel_value : f32,
    z_score : f32
}

pub struct HpcResults {
    pub buffer: ImageBuffer,
    pub replaced_pixels : Vec<ReplacedPixel>
}






fn isolate_window(buffer:&ImageBuffer, window_size:i32, x:usize, y:usize) -> Vec<f32> {
    let mut v:Vec<f32> = Vec::with_capacity(36);
    let start = window_size / 2 * -1;
    let end = window_size / 2 + 1;
    for _y in start..end as i32 {
        for _x in start..end as i32 {
            let get_x = x as i32 + _x;
            let get_y = y as i32 + _y;
            if get_x >= 0 && get_x < buffer.width as i32 && get_y >= 0 && get_y < buffer.height as i32 {
                v.push(buffer.get(get_x as usize, get_y as usize).unwrap());
            }
        }
    }
    v
}

pub fn hot_pixel_detection(buffer:&ImageBuffer, window_size:i32, threshold:f32) -> error::Result<HpcResults> {

    let mut map = ImageBuffer::new(buffer.width, buffer.height).unwrap();
    let mut replaced_pixels:Vec<ReplacedPixel> = Vec::new();

    for y in 1..buffer.height - 1 {
        for x in 1..buffer.width -1 {
            let pixel_value = buffer.get(x, y).unwrap();
            let window = isolate_window(buffer, window_size, x, y);
            let z_score = stats::z_score(pixel_value, &window[0..]).unwrap();
            if z_score > threshold {
                let m = stats::mean(&window[0..]).unwrap();
                map.put(x, y, m).unwrap();

                replaced_pixels.push(ReplacedPixel{
                    x,
                    y,
                    pixel_value,
                    z_score
                });

            } else {
                map.put(x, y, buffer.get(x, y).unwrap()).unwrap();
            }
        }
    }
    Ok(HpcResults{buffer:map, replaced_pixels})
}