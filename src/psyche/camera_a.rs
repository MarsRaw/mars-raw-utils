use crate::{
    calibrate::*,
    calprofile::CalProfile,
    decompanding,
    enums::{self, Instrument},
    inpaintmask,
    marsimage::MarsImage,
    util,
};

use sciimg::prelude::*;

use anyhow::Result;

#[derive(Copy, Clone)]
pub struct PsycheCameraA {}
impl Calibration for PsycheCameraA {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::PsycheCameraA)
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
            cal_warn(cal_context, &out_file)
        } else {
            let mut raw = MarsImage::open(input_file, enums::Instrument::PsycheCameraA);

            let data_max = if cal_context.apply_ilt {
                info!("Decompanding...");
                let lut =
                    decompanding::get_ilt_for_instrument(enums::Instrument::PsycheCameraA).unwrap();
                raw.decompand(&lut);
                lut.max() as f32
            } else {
                255.0
            };

            if raw.image.width == 1648 {
                info!("Applying Dark Signal Correction. Reference column is 15");
                raw.dark_signal_correction_with_ref_cols(15);
            } else {
                info!("Reference columns may have been cropped due to subframing. Skipping dark signal correction.");
            }

            if (cal_context.desmear_epsilon - 0.0).abs() > f32::EPSILON {
                info!(
                    "Applying CCD frame-transfer smear correction with epsilon of {}",
                    cal_context.desmear_epsilon
                );
                raw.desmear_ccd_image(cal_context.desmear_epsilon);
            } // else, don't bother

            info!("Applying inpainting repair of blemishes...");
            let inpaint_mask = inpaintmask::load_mask(enums::Instrument::PsycheCameraA).unwrap();
            raw.apply_inpaint_fix_with_mask(&inpaint_mask);

            if raw.image.width == 1648 && raw.image.height == 1200 {
                vprintln!("Cropping out dark reference pixels...");
                raw.image.crop(48, 16, 1584, 1184);
            }

            info!("Normalizing for 16bit output");
            raw.image.normalize_to_16bit_with_max(data_max);

            info!("Writing to disk...");
            raw.update_history();
            match raw.save(&out_file) {
                Ok(_) => cal_ok(cal_context, &out_file),
                Err(why) => {
                    error!("Error saving file: {}", why);
                    cal_fail(cal_context, &out_file)
                }
            }
        }
        // Do stuff
    }
}
