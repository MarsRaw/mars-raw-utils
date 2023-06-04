use crate::{
    calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument,
    marsimage::MarsImage, util,
};

use anyhow::Result;
use sciimg::path;

#[derive(Copy, Clone)]
pub struct M20CacheCam {}

impl Calibration for M20CacheCam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::M20CacheCam)
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

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20CacheCam);

        let data_max = if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            let lut = decompanding::get_ilt_for_instrument(enums::Instrument::M20CacheCam).unwrap();
            raw.decompand(&lut);
            lut.max() as f32
        } else {
            255.0
        };

        if input_file.contains("ECM") && raw.image.is_grayscale() {
            vprintln!("Image appears to be grayscale, applying debayering...");
            raw.debayer_with_method(cal_context.debayer_method);
        }

        vprintln!("Applying color weights...");
        raw.apply_weight(
            cal_context.red_scalar,
            cal_context.green_scalar,
            cal_context.blue_scalar,
        );

        if cal_context.decorrelate_color {
            vprintln!("Normalizing with decorrelated colors...");
            raw.image.normalize_to_16bit_decorrelated();
        } else {
            vprintln!("Normalizing with correlated colors...");
            raw.image.normalize_to_16bit_with_max(data_max);
        }

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
