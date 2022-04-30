use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    enums::Instrument,
    path, 
    calibfile,
    util,
    calprofile::CalProfile,
    calibrate::*
};

use sciimg::{
    imagebuffer,
    error
};


#[derive(Copy, Clone)]
pub struct M20SuperCam {}

impl Calibration for M20SuperCam {

    fn accepts_instrument(&self, instrument:Instrument) -> bool {
        match instrument {
            Instrument::M20SuperCam => true,
            _ => false
        }
    }

    fn process_file(&self, input_file:&str, cal_context:&CalProfile, only_new:bool)  -> error::Result<CompleteContext> {

        let out_file = util::append_file_name(input_file, &cal_context.filename_suffix.as_str());
        if path::file_exists(&out_file) && only_new {
            vprintln!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context);
        }
        
        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20SuperCam);
        
        vprintln!("Loading image mask");
        let mask = imagebuffer::ImageBuffer::from_file(calibfile::get_calibration_file_for_instrument(enums::Instrument::M20SuperCam, enums::CalFileType::Mask).unwrap().as_str()).unwrap();
        raw.apply_mask(&mask);

        let data_max = 255.0;

        
        if input_file.find("ECM") != None && raw.image.is_grayscale() {
            vprintln!("Image appears to be grayscale, applying debayering...");
            raw.debayer();
        }

        // Gonna start with standard rectangular flat field, but should really
        // mask it to just the round light-collecting area of the image.
        vprintln!("Flatfielding...");
        raw.flatfield();

        vprintln!("Applying color weights...");
        raw.apply_weight(cal_context.red_scalar, cal_context.green_scalar, cal_context.blue_scalar);

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context)
    }
}

