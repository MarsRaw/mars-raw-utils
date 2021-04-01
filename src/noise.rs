
use opencv::{
    core,
    prelude::*,
    imgproc,
    types
};

use crate::{
    error, 
    rgbimage::RgbImage,
    opencvutils
};



pub fn color_noise_reduction(image:&mut RgbImage, amount:i32) -> error::Result<RgbImage> {
    unsafe {
        image.normalize_to_8bit_with_max(2033.0).unwrap();
        
        let m = opencvutils::rgbimage_to_cv2_mat_u8(image).unwrap();

        let mut lab = Mat::new_rows_cols(image.height as i32, image.width as i32, core::CV_8UC3).unwrap();
        
        imgproc::cvt_color(&m, &mut lab, imgproc::COLOR_RGB2Lab, 0).unwrap();
       
        let mut split = types::VectorOfMat::new();

        core::split(&lab, &mut split).unwrap();

        let l = split.get(0).unwrap();
        let a = split.get(1).unwrap();
        let b = split.get(2).unwrap();

        let mut a_out = Mat::default().unwrap();
        let mut b_out = Mat::default().unwrap();

        imgproc::gaussian_blur(&a, &mut a_out, core::Size::new(amount, amount), 0.0, 0.0, core::BORDER_DEFAULT).unwrap();
        imgproc::gaussian_blur(&b, &mut b_out, core::Size::new(amount, amount), 0.0, 0.0, core::BORDER_DEFAULT).unwrap();
        
        let mut to_merge = types::VectorOfMat::new();
        to_merge.push(l);
        to_merge.push(a_out);
        to_merge.push(b_out);

        core::merge(&to_merge, &mut lab).unwrap();

        let mut o = Mat::default().unwrap();
        imgproc::cvt_color(&lab, &mut o, imgproc::COLOR_Lab2RGB, 0).unwrap();

        let mut i = opencvutils::cv2_mat_to_rgbimage_u8(&o, image.width, image.height).unwrap();
        i.normalize_to_12bit_with_max(255.0).unwrap();
        Ok(i)
    }
}

