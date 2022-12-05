use crate::{
    calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument, image::MarsImage,
    path, util, vprintln,
};

use sciimg::error;

#[derive(Copy, Clone)]
pub struct M20EECam {}

impl Calibration for M20EECam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(
            instrument,
            Instrument::M20FrontHazLeft
                | Instrument::M20FrontHazRight
                | Instrument::M20NavcamLeft
                | Instrument::M20NavcamRight
                | Instrument::M20RearHazLeft
                | Instrument::M20RearHazRight
        )
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

        let mut instrument = enums::Instrument::M20NavcamRight;

        // Attempt to figure out camera from file name
        if util::filename_char_at_pos(input_file, 0) == 'N' {
            // NAVCAMS
            if util::filename_char_at_pos(input_file, 1) == 'L' {
                // Left
                instrument = enums::Instrument::M20NavcamLeft;
            } else {
                // Assume Right
                instrument = enums::Instrument::M20NavcamRight;
            }
        } else if util::filename_char_at_pos(input_file, 0) == 'F' {
            // FHAZ
            if util::filename_char_at_pos(input_file, 1) == 'L' {
                // Left
                instrument = enums::Instrument::M20FrontHazLeft;
            } else {
                // Assume Right
                instrument = enums::Instrument::M20FrontHazRight;
            }
        } else if util::filename_char_at_pos(input_file, 0) == 'R' {
            // RHAZ
            if util::filename_char_at_pos(input_file, 1) == 'L' {
                // Left
                instrument = enums::Instrument::M20RearHazLeft;
            } else {
                // Assume Right
                instrument = enums::Instrument::M20RearHazRight;
            }
        }

        let mut raw = MarsImage::open(String::from(input_file), instrument);

        let data_max = if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            raw.decompand(&decompanding::get_ilt_for_instrument(instrument));
            decompanding::get_max_for_instrument(instrument) as f32
        } else {
            255.0
        };

        // Looks like 'ECM' in the name seems to indicate that it still have the bayer pattern
        if raw.image.is_grayscale() {
            vprintln!("Debayering...");
            raw.debayer();
        }

        vprintln!("Flatfielding...");
        raw.flatfield();

        // We're going to need a reliable way of figuring out what part of the sensor
        // is represented before we can flatfield or apply an inpainting mask
        //vprintln!("Inpainting...");
        //raw.apply_inpaint_fix().unwrap();

        if !raw.image.is_grayscale() {
            vprintln!("Applying color weights...");
            raw.apply_weight(
                cal_context.red_scalar,
                cal_context.green_scalar,
                cal_context.blue_scalar,
            );
        }

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

        // Trim off border pixels
        //let crop_to_width = raw.image.width - 4;
        //let crop_to_height = raw.image.height - 4;
        //raw.crop(2, 2, crop_to_width, crop_to_height).unwrap();

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context)
    }
}
