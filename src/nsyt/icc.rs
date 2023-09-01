use crate::{
    calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument,
    marsimage::MarsImage, util,
};

use sciimg::prelude::*;

use anyhow::Result;

#[derive(Copy, Clone)]
pub struct NsytIcc {}
impl Calibration for NsytIcc {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::NsytICC)
    }

    fn process_file(
        &self,
        input_file: &str,
        cal_context: &CalProfile,
        only_new: bool,
    ) -> Result<CompleteContext> {
        let out_file = util::append_file_name(input_file, cal_context.filename_suffix.as_str());
        if path::file_exists(&out_file) && only_new {
            warn!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context, &out_file);
        }

        let mut raw = MarsImage::open(input_file, enums::Instrument::NsytICC);

        let data_max = if cal_context.apply_ilt {
            info!("Decompanding...");

            match decompanding::get_ilt_for_instrument(raw.instrument) {
                Ok(lut) => {
                    raw.decompand(&lut);
                    lut.max() as f32
                }
                Err(why) => {
                    error!("Failed to load LUT file for InSight IDC: {}", why);
                    return cal_fail(cal_context, &out_file);
                }
            }
        } else {
            255.0
        };

        info!("Flatfielding...");
        raw.flatfield();

        info!("Applying color weights...");
        raw.apply_weight(
            cal_context.red_scalar,
            cal_context.green_scalar,
            cal_context.blue_scalar,
        );

        info!("Cropping...");
        raw.image.crop(3, 3, 1018, 1018);

        if cal_context.srgb_color_correction {
            info!("Applying sRGB color conversion");
            raw.image
                .convert_colorspace(color::ColorSpaceType::RGB, color::ColorSpaceType::sRGB)?;
        }

        if cal_context.decorrelate_color {
            info!("Normalizing with decorrelated colors...");
            raw.image.normalize_to_16bit_decorrelated();
        } else {
            info!("Normalizing with correlated colors...");
            raw.image.normalize_to_16bit_with_max(data_max);
        }

        vprintln!("Writing to disk...");
        raw.update_history();
        match raw.save(&out_file) {
            Ok(_) => cal_ok(cal_context, &out_file),
            Err(why) => {
                error!("Error saving file: {}", why);
                cal_fail(cal_context, &out_file)
            }
        }
    }
}
