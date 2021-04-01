

use opencv::{
    core,
    imgcodecs, 
    photo
};

use crate::{
    constants, 
    path, 
    error, 
    enums, 
    imagebuffer::ImageBuffer, 
    vprintln,
    opencvutils
};


fn determine_mask_file(instrument:enums::Instrument) -> error::Result<&'static str> {
    match instrument {
        enums::Instrument::MslMAHLI => 
                    Ok(constants::cal::MSL_MAHLI_INPAINT_MASK_PATH),
        enums::Instrument::M20MastcamZLeft => 
                    Ok(constants::cal::M20_INPAINT_MASK_LEFT_PATH),
        enums::Instrument::M20MastcamZRight =>
                    Ok(constants::cal::M20_INPAINT_MASK_RIGHT_PATH),
        _ => Err(constants::status::UNSUPPORTED_INSTRUMENT)
    }
}

fn load_mask_file(filename:&str) -> error::Result<core::Mat> {
    if ! path::file_exists(filename) {
        return Err(constants::status::FILE_NOT_FOUND);
    }

    vprintln!("Loading inpaint mask file {}", filename);

    let mask = imgcodecs::imread(filename, imgcodecs::IMREAD_GRAYSCALE).unwrap();
    let subframe = opencvutils::crop(&mask, 32, 16, 1584, 1184).unwrap();
    Ok(subframe)
}

fn load_mask(instrument:enums::Instrument) -> error::Result<core::Mat> {
    let mask_file = determine_mask_file(instrument).unwrap();
    load_mask_file(mask_file)
}

pub fn apply_inpaint_to_buffer(buffer:&ImageBuffer, instrument:enums::Instrument) -> error::Result<ImageBuffer> {

    let mask = load_mask(instrument).unwrap();
    let buffer_as_mat = opencvutils::buffer_to_cv2_mat(&buffer).unwrap();

    unsafe {
        let mut dest = core::Mat::new_rows_cols(buffer.height as i32, buffer.width as i32, core::CV_8U).unwrap();
        photo::inpaint(&buffer_as_mat, &mask, &mut dest, 3.0, photo::INPAINT_TELEA).unwrap();
        let b = opencvutils::cv2_mat_to_buffer(&dest, buffer.width, buffer.height).unwrap();
        Ok(b)
    }
}
