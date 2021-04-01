

use opencv::{
    core, 
    prelude::*
};

use crate::{
    error, 
    enums, 
    imagebuffer::ImageBuffer,
    rgbimage::RgbImage
};



pub fn crop(m:&core::Mat, x:i32, y:i32, width:i32, height:i32) -> error::Result<core::Mat> {
    let rect = core::Rect::new(x, y, width, height);
    let subframe = core::Mat::roi(&m, rect).unwrap();
    Ok(subframe)
}

fn convert_f32_to_u16(src:&[f32]) -> error::Result<Vec<u16>> {
    
    let mut dest:Vec<u16> = Vec::with_capacity(src.len());
    dest.resize(src.len(), 0);

    for i in 0..src.len() {
        dest[i] = src[i].round() as u16;
    }

    Ok(dest)
}

pub fn rgbimage_to_cv2_mat_u8(image:&RgbImage) -> error::Result<core::Mat>  {
    let need_len = image.width * image.height * 3;
    let mut dest:Vec<u8> = Vec::with_capacity(need_len);
    dest.resize(need_len, 0);
    for y in 0..image.height {
        for x in 0..image.width {
            let idx = y * (image.width * 3) + (x * 3);
            let r = image.red().get(x, y).unwrap() as u8;
            let g = image.green().get(x, y).unwrap() as u8;
            let b = image.blue().get(x, y).unwrap() as u8;
            dest[idx + 0] = r;
            dest[idx + 1] = g;
            dest[idx + 2] = b;
        }
    }

    let f = core::Mat::from_slice(&dest[0..]).unwrap();
    let b = f.reshape(3, image.height as i32).unwrap();

    Ok(b)
}

pub fn rgbimage_to_cv2_mat_u16(image:&RgbImage) -> error::Result<core::Mat>  {
    let need_len = image.width * image.height * 3;
    let mut dest:Vec<u16> = Vec::with_capacity(need_len);
    dest.resize(need_len, 0);
    for y in 0..image.height {
        for x in 0..image.width {
            let idx = y * (image.width * 3) + (x * 3);
            let r = image.red().get(x, y).unwrap() as u16;
            let g = image.green().get(x, y).unwrap() as u16;
            let b = image.blue().get(x, y).unwrap() as u16;
            dest[idx + 0] = r;
            dest[idx + 1] = g;
            dest[idx + 2] = b;
        }
    }

    let f = core::Mat::from_slice(&dest[0..]).unwrap();
    let b = f.reshape(3, image.height as i32).unwrap();

    Ok(b)
}

pub fn cv2_mat_to_rgbimage_u8(m:&core::Mat, width:usize, height:usize) -> error::Result<RgbImage> {
    let red = cv2_mat_to_buffer_2d_u8(&m, 0, width, height).unwrap();
    let green = cv2_mat_to_buffer_2d_u8(&m, 1, width, height).unwrap();
    let blue = cv2_mat_to_buffer_2d_u8(&m, 2, width, height).unwrap();
    
    let newimage = RgbImage::new_from_buffers_rgb(&red, &green, &blue, enums::Instrument::None).unwrap();
    Ok(newimage)
}

pub fn cv2_mat_to_rgbimage_u16(m:&core::Mat, width:usize, height:usize) -> error::Result<RgbImage> {
    let red = cv2_mat_to_buffer_2d_u16(&m, 0, width, height).unwrap();
    let green = cv2_mat_to_buffer_2d_u16(&m, 1, width, height).unwrap();
    let blue = cv2_mat_to_buffer_2d_u16(&m, 2, width, height).unwrap();
    
    let newimage = RgbImage::new_from_buffers_rgb(&red, &green, &blue, enums::Instrument::None).unwrap();
    Ok(newimage)
}

pub fn buffer_to_cv2_mat(buffer:&ImageBuffer) -> error::Result<core::Mat> {
    let f = core::Mat::from_slice(&convert_f32_to_u16(&buffer.buffer[0..]).unwrap()[0..]).unwrap();
    let b = f.reshape(0, buffer.height as i32).unwrap();
    Ok(b)
}


pub fn cv2_mat_to_buffer_2d_u8(m:&core::Mat, channel:usize, width:usize, height:usize) -> error::Result<ImageBuffer> {
    let mut b = ImageBuffer::new(width, height).unwrap();
    let v = m.data_typed::<core::Vec3<u8>>().unwrap().to_vec();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let pxl = v[idx];
            b.put(x, y, pxl[channel] as f32).unwrap();
        }
    }
    
    Ok(b)
}

pub fn cv2_mat_to_buffer_2d_u16(m:&core::Mat, channel:usize, width:usize, height:usize) -> error::Result<ImageBuffer> {
    let mut b = ImageBuffer::new(width, height).unwrap();
    let v = m.data_typed::<core::Vec3<u16>>().unwrap().to_vec();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let pxl = v[idx];
            b.put(x, y, pxl[channel] as f32).unwrap();
        }
    }
    
    Ok(b)
}

pub fn cv2_mat_to_buffer(m:&core::Mat, width:usize, height:usize) -> error::Result<ImageBuffer> {
    let mut b = ImageBuffer::new(width, height).unwrap();
    let v = m.data_typed::<u16>().unwrap().to_vec();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            b.put(x, y, v[idx] as f32).unwrap();
        }
    }
    
    Ok(b)
}