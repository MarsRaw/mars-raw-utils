use crate::{
    calibrate::*, calprofile::CalProfile, enums, enums::Instrument, marsimage::MarsImage, path,
    util, vprintln,
};

use sciimg::error;

#[derive(Copy, Clone)]
pub struct M20SherlocAci {}

impl Calibration for M20SherlocAci {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::M20SherlocAci)
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

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20SherlocAci);

        vprintln!("Flatfielding...");
        raw.flatfield();

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(255.0);

        if raw.image.width == 1648 && raw.image.height == 1200 {
            vprintln!("Cropping...");
            raw.image.crop(23, 2, 1607, 1198);
        } else if raw.image.width == 1600 && raw.image.height == 1200 {
            vprintln!("Cropping...");
            raw.image.crop(23, 2, 1577, 1198);
        }

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context)
    }
}
