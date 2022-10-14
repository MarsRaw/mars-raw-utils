use crate::{
    calibfile, calibrate::*, calprofile::CalProfile, enums, enums::Instrument, image::MarsImage,
    path, util, vprintln,
};

use sciimg::{error, imagebuffer};

#[derive(Copy, Clone)]
pub struct MslChemCam {}

impl Calibration for MslChemCam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        match instrument {
            Instrument::MslChemCam => true,
            _ => false,
        }
    }

    fn process_file(
        &self,
        input_file: &str,
        cal_context: &CalProfile,
        only_new: bool,
    ) -> error::Result<CompleteContext> {
        let out_file = util::append_file_name(input_file, cal_context.filename_suffix.as_str());
        if path::file_exists(&out_file) && only_new {
            vprintln!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context);
        }

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::MslChemCam);

        vprintln!("Loading image mask");
        let mask = imagebuffer::ImageBuffer::from_file(
            calibfile::get_calibration_file_for_instrument(
                enums::Instrument::MslChemCam,
                enums::CalFileType::Mask,
            )
            .unwrap()
            .as_str(),
        )
        .unwrap();
        raw.apply_alpha(&mask);

        let data_max = 255.0;

        if input_file.contains("EDR") {
            vprintln!("Image appears to be in standard contrast");

            vprintln!("Flatfielding...");
            raw.flatfield();
        } else if input_file.contains("EDR") {
            vprintln!("Image appears to be in enhanced contrast");
            // ... Don't do flatfielding, these appear to already been applied.
            // ... Do something about that
        }

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context)
    }
}
