
// https://www.researchgate.net/publication/238183352_An_Image_Inpainting_Technique_Based_on_the_Fast_Marching_Method


use crate::{
    constants, 
    path, 
    enums, 
    vprintln,
    calibfile
};

use sciimg::{
    imagebuffer::ImageBuffer,
    error
};

fn determine_mask_file(instrument:enums::Instrument) -> error::Result<String> {
    calibfile::get_calibration_file_for_instrument(instrument, enums::CalFileType::InpaintMask)
}

pub fn inpaint_supported_for_instrument(instrument:enums::Instrument) -> bool {
    let r = determine_mask_file(instrument);
    r.is_ok()
}

fn load_mask_file(filename:&str, instrument:enums::Instrument) -> error::Result<ImageBuffer> {
    vprintln!("Loading inpaint mask file {}", filename);

    if ! path::file_exists(filename) {
        return Err(constants::status::FILE_NOT_FOUND);
    }
    let mask = match ImageBuffer::from_file(filename) {
        Ok(m) => m,
        Err(e) => return Err(e)
    };
    
    match instrument {
        enums::Instrument::MslMAHLI => mask.get_subframe(32, 16, 1584, 1184),
        _ => Ok(mask)
    }
}

pub fn load_mask(instrument:enums::Instrument) -> error::Result<ImageBuffer> {
    let mask_file = match determine_mask_file(instrument) {
        Ok(m) => m,
        Err(e) => return Err(e)
    };

    load_mask_file(mask_file.as_str(), instrument)
}


