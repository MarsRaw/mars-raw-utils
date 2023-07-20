use crate::{
    calibrate::*, calprofile::CalProfile, enums, enums::Instrument, marsimage::MarsImage, util,
};

use sciimg::path;

use anyhow::Result;

#[derive(Copy, Clone)]
pub struct M20Pixl {}

impl Calibration for M20Pixl {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::M20Pixl)
    }

    fn process_file(
        &self,
        input_file: &str,
        cal_context: &CalProfile,
        only_new: bool,
    ) -> Result<CompleteContext> {
        let out_file = util::append_file_name(input_file, cal_context.filename_suffix.as_str());
        if path::file_exists(&out_file) && only_new {
            vprintln!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context, &out_file);
        }

        let mut raw = MarsImage::open(input_file, enums::Instrument::M20Pixl);

        vprintln!("Flatfielding...");
        raw.flatfield();

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(255.0);

        vprintln!("Writing to disk...");
        match raw.save(&out_file) {
            Ok(_) => cal_ok(cal_context, &out_file),
            Err(why) => {
                veprintln!("Error saving file: {}", why);
                cal_fail(cal_context, &out_file)
            }
        }
    }
}
