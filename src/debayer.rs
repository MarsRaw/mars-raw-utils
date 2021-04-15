

use opencv::{
    core, 
    prelude::*, 
    imgproc
};

use crate::{
    error, 
    imagebuffer::ImageBuffer, 
    rgbimage::RgbImage,
    opencvutils
};


pub fn debayer(buffer:&ImageBuffer) -> error::Result<RgbImage> {
    unsafe {
        let buffer_as_mat = opencvutils::buffer_to_cv2_mat(&buffer).unwrap();
        let mut dst_mat = Mat::new_rows_cols(buffer.height as i32, buffer.width as i32, core::CV_8UC3).unwrap();
        imgproc::cvt_color(&buffer_as_mat, &mut dst_mat, imgproc::COLOR_BayerBG2RGB, 0).unwrap();
        let mut newimage = opencvutils::cv2_mat_to_rgbimage_u16(&dst_mat, buffer.width, buffer.height).unwrap();
        
        // Might not be the best place to do this...
        newimage.copy_mask_from(buffer);

        Ok(newimage)
    }
}
