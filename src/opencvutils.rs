

use opencv::{core, prelude::*, imgcodecs, photo};
use crate::{constants, path, error, enums, imagebuffer::ImageBuffer, vprintln};
use opencv::prelude::MatTrait;


pub fn crop(m:&core::Mat, x:i32, y:i32, width:i32, height:i32) -> error::Result<core::Mat> {
    let rect = core::Rect::new(x, y, width, height);
    let subframe = core::Mat::roi(&m, rect).unwrap();
    Ok(subframe)
}


pub fn buffer_to_cv2_mat(buffer:&ImageBuffer) -> error::Result<core::Mat> {
    let f = core::Mat::from_slice(&buffer.buffer[0..]).unwrap();
    let b = f.reshape(0, buffer.height as i32).unwrap();
    Ok(b)
}

pub fn cv2_mat_to_buffer(m:&core::Mat, width:usize, height:usize) -> error::Result<ImageBuffer> {
    let mut b = ImageBuffer::new(width, height).unwrap();
    let v = m.data_typed::<f32>().unwrap().to_vec();
    
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            b.put(x, y, v[idx]).unwrap();
        }
    }
    
    Ok(b)
}