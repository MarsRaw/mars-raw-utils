use crate::{
    calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument,
    marsimage::MarsImage, util,
};

use sciimg::path;

use anyhow::Result;

#[derive(Copy, Clone)]
pub struct MslMardi {}

impl Calibration for MslMardi {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::MslMARDI)
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

        let mut raw = MarsImage::open(input_file, enums::Instrument::MslMARDI);

        let data_max = if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            let lut = decompanding::get_ilt_for_instrument(enums::Instrument::MslMARDI).unwrap();
            raw.decompand(&lut);
            lut.max() as f32
        } else {
            255.0
        };

        vprintln!("Flatfielding...");
        raw.flatfield();

        vprintln!("Applying color weights...");
        raw.apply_weight(
            cal_context.red_scalar,
            cal_context.green_scalar,
            cal_context.blue_scalar,
        );

        vprintln!("Cropping...");
        raw.image.crop(24, 6, 1599, 1188);

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

        vprintln!("Writing to disk...");
        raw.update_history();
        match raw.save(&out_file) {
            Ok(_) => cal_ok(cal_context, &out_file),
            Err(why) => {
                veprintln!("Error saving file: {}", why);
                cal_fail(cal_context, &out_file)
            }
        }
    }
}
