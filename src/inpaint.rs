
// https://www.researchgate.net/publication/238183352_An_Image_Inpainting_Technique_Based_on_the_Fast_Marching_Method


use crate::{
    constants, 
    path, 
    error, 
    enums, 
    imagebuffer::ImageBuffer, 
    vprintln,
    not_implemented
};


fn determine_mask_file(instrument:enums::Instrument) -> error::Result<&'static str> {
    match instrument {
        enums::Instrument::MslMAHLI => 
                    Ok(constants::cal::MSL_MAHLI_INPAINT_MASK_PATH),
        enums::Instrument::M20MastcamZLeft => 
                    Ok(constants::cal::M20_INPAINT_MASK_LEFT_PATH),
        enums::Instrument::M20MastcamZRight =>
                    Ok(constants::cal::M20_INPAINT_MASK_RIGHT_PATH),
        enums::Instrument::MslNavCamRight =>
                    Ok(constants::cal::MSL_NCAM_RIGHT_INPAINT_PATH),
        enums::Instrument::MslMastcamLeft =>
                    Ok(constants::cal::MSL_MCAM_LEFT_INPAINT_PATH),
        enums::Instrument::M20Watson =>
                    Ok(constants::cal::M20_WATSON_INPAINT_MASK_PATH),
        _ => Err(constants::status::UNSUPPORTED_INSTRUMENT)
    }
}

pub fn inpaint_supported_for_instrument(instrument:enums::Instrument) -> bool {
    let r = determine_mask_file(instrument);
    match r {
        Ok(_) => true,
        Err(_) => false
    }
}

fn load_mask_file(filename:&str, instrument:enums::Instrument) -> error::Result<ImageBuffer> {
    vprintln!("Loading inpaint mask file {}", filename);

    if ! path::file_exists(filename) {
        return Err(constants::status::FILE_NOT_FOUND);
    }
    let mask = ImageBuffer::from_file(filename).unwrap();
    
    match instrument {
        enums::Instrument::MslMAHLI => Ok(mask.get_subframe(32, 16, 1584, 1184).unwrap()),
        _ => Ok(mask)
    }
}

fn load_mask(instrument:enums::Instrument) -> error::Result<ImageBuffer> {
    let mask_file = determine_mask_file(instrument).unwrap();
    load_mask_file(mask_file, instrument)
}


pub fn apply_inpaint_to_buffer(buffer:&ImageBuffer, instrument:enums::Instrument) -> error::Result<ImageBuffer> {

    let mut mask = load_mask(instrument).unwrap();

    // Crop the mask image if it's larger than the input image. 
    // Sizes need to match
    if mask.width > buffer.width {
        let x = (mask.width - buffer.width) / 2;
        let y = (mask.height - buffer.width) / 2;
        vprintln!("Cropping inpaint mask with params {}, {}, {}, {}", x, y, buffer.width, buffer.height);
        mask = mask.get_subframe(x, y, buffer.width, buffer.height).unwrap();
    }



    not_implemented!()
    // unsafe {
    //     let mut dest = core::Mat::new_rows_cols(buffer.height as i32, buffer.width as i32, core::CV_8U).unwrap();
    //     photo::inpaint(&buffer_as_mat, &mask, &mut dest, 3.0, photo::INPAINT_TELEA).unwrap();
    //     let b = opencvutils::cv2_mat_to_buffer(&dest, buffer.width, buffer.height).unwrap();
    //     Ok(b)
    // }
}


