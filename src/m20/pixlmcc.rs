use crate::{
    calibrate::*, calprofile::CalProfile, enums, enums::Instrument, image::MarsImage, path, util,
    vprintln,
};

use sciimg::error;

#[derive(Copy, Clone)]
pub struct M20Pixl {}

impl Calibration for M20Pixl {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        match instrument {
            Instrument::M20Pixl => true,
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

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20Pixl);

        vprintln!("Flatfielding...");
        raw.flatfield();

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(255.0);

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context)
    }
}
