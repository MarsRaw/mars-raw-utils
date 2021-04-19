
extern crate bayer;
use std::io::Cursor;

use crate::{
    error, 
    imagebuffer::ImageBuffer, 
    rgbimage::RgbImage,
    enums
};

pub fn debayer(buffer:&ImageBuffer) -> error::Result<RgbImage> {
    let img_w = buffer.width;
    let img_h = buffer.height;
    let depth = bayer::RasterDepth::Depth8;  // Limiting to 8bit debayer for now
    let bytes_per_pixel = 3;
    let mut buf = vec![0; bytes_per_pixel * img_w * img_h];
    
    let mut dst = bayer::RasterMut::new(img_w, img_h, depth, &mut buf);

    let mut in_buf = vec![0; 3 * buffer.width * buffer.height];
    for i in 0..buffer.buffer.len() {
        in_buf[i] = buffer.buffer[i] as u8;
    }

    bayer::run_demosaic(&mut Cursor::new(&in_buf[..]), 
                        bayer::BayerDepth::Depth8,
                        bayer::CFA::RGGB,
                        bayer::Demosaic::Cubic,
                        &mut dst).unwrap();
                  
    let mut red = ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.mask).unwrap();
    let mut green = ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.mask).unwrap();
    let mut blue = ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.mask).unwrap();

    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let r = buf[y * (buffer.width * 3) + x * 3 + 0];
            let g = buf[y * (buffer.width * 3) + x * 3 + 1];
            let b = buf[y * (buffer.width * 3) + x * 3 + 2];
            red.put(x, y, r as f32).unwrap();
            green.put(x, y, g as f32).unwrap();
            blue.put(x, y, b as f32).unwrap();
        }
    }

    let newimage = RgbImage::new_from_buffers_rgb(&red, &green, &blue, enums::Instrument::None, enums::ImageMode::U8BIT).unwrap();

    Ok(newimage)
}
