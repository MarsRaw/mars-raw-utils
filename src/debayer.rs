

use opencv::{
    core, 
    prelude::*, 
    imgproc
};

use crate::{
    constants, 
    error, 
    imagebuffer::ImageBuffer, 
    rgbimage::RgbImage,
    vprintln,
    opencvutils,
    not_implemented
};


pub fn debayer(buffer:ImageBuffer) -> error::Result<RgbImage> {
    unsafe {
        let buffer_as_mat = opencvutils::buffer_to_cv2_mat(&buffer).unwrap();
        let mut dst_mat = Mat::new_rows_cols(buffer.height as i32, buffer.width as i32, core::CV_8UC3).unwrap();
        imgproc::cvt_color(&buffer_as_mat, &mut dst_mat, imgproc::COLOR_BayerBG2BGR, 0).unwrap();
        
        // TODO: Convert to RgbImage
        not_implemented!()
    }
}
