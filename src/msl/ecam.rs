use crate::{
    calibfile, calibrate::*, calprofile::CalProfile, enums, enums::Instrument, image::MarsImage,
    inpaintmask, path, util, vprintln,
};

use sciimg::error;

// Doesn't support subframed images yet since we won't know what part of the sensor was
// used from the raws alone. If it's in the JSON response from the raw image site, then
// maybe I can embed that data into the jpegs (EXIF) when downloaded using msl_fetch_raws
// and trigger off of that. Still need to think of times when someone downloads the image
// directly from the webpage using their browser as the website often prepends a wonky
// prefix to the image filename.
//
// Also leaving in the ILT parameter until I iron out the cases in which it's needed
// for ECAM.
#[derive(Copy, Clone)]
pub struct MslEcam {}

impl Calibration for MslEcam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(
            instrument,
            Instrument::MslNavCamLeft
                | Instrument::MslNavCamRight
                | Instrument::MslFrontHazLeft
                | Instrument::MslFrontHazRight
                | Instrument::MslRearHazLeft
                | Instrument::MslRearHazRight
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

        let instrument;
        match util::filename_char_at_pos(input_file, 0) {
            'N' => {
                // NAVCAMS
                match util::filename_char_at_pos(input_file, 1) == 'L' {
                    // Left
                    true => instrument = enums::Instrument::MslNavCamLeft,
                    // Assume Right
                    false => instrument = enums::Instrument::MslNavCamRight,
                }
            }
            'F' => {
                // FHAZ
                match util::filename_char_at_pos(input_file, 1) == 'L' {
                    // Left
                    true => instrument = enums::Instrument::MslFrontHazLeft,
                    // Assume Right
                    false => instrument = enums::Instrument::MslFrontHazRight,
                }
            }
            'R' => {
                // RHAZ
                match util::filename_char_at_pos(input_file, 1) == 'L' {
                    // Left
                    true => instrument = enums::Instrument::MslRearHazLeft,
                    // Assume Right
                    false => instrument = enums::Instrument::MslRearHazRight,
                }
            }
            // TODO should never panic?
            _ => panic!("Unrecognized camera option"),
        }

        // // Attempt to figure out camera from file name
        // if util::filename_char_at_pos(input_file, 0) == 'N' {
        //     // NAVCAMS
        //     if util::filename_char_at_pos(input_file, 1) == 'L' {
        //         // Left
        //         instrument = enums::Instrument::MslNavCamLeft;
        //     } else {
        //         // Assume Right
        //         instrument = enums::Instrument::MslNavCamRight;
        //     }
        // } else if util::filename_char_at_pos(input_file, 0) == 'F' {
        //     // FHAZ
        //     if util::filename_char_at_pos(input_file, 1) == 'L' {
        //         // Left
        //         instrument = enums::Instrument::MslFrontHazLeft;
        //     } else {
        //         // Assume Right
        //         instrument = enums::Instrument::MslFrontHazRight;
        //     }
        // } else if util::filename_char_at_pos(input_file, 0) == 'R' {
        //     // RHAZ
        //     if util::filename_char_at_pos(input_file, 1) == 'L' {
        //         // Left
        //         instrument = enums::Instrument::MslRearHazLeft;
        //     } else {
        //         // Assume Right
        //         instrument = enums::Instrument::MslRearHazRight;
        //     }
        // }

        let mut raw = MarsImage::open(String::from(input_file), instrument);

        // Exclude subframed images for now...
        if inpaintmask::inpaint_supported_for_instrument(instrument) && raw.image.height >= 1022 {
            vprintln!("Inpainting...");
            raw.apply_inpaint_fix();
        } else {
            vprintln!("Inpainting not supported for instrument {:?}", instrument);
        }

        if cal_context.hot_pixel_detection_threshold > 0.0 {
            vprintln!(
                "Hot pixel correction with variance threshold {}...",
                cal_context.hot_pixel_detection_threshold
            );
            raw.hot_pixel_correction(
                cal_context.hot_pixel_window_size,
                cal_context.hot_pixel_detection_threshold,
            );
        }

        let data_max = 255.0;

        let flat_file_path = calibfile::get_calibration_file_for_instrument(
            instrument,
            enums::CalFileType::FlatField,
        )
        .unwrap();
        vprintln!("Using flat file: {}", flat_file_path);

        if path::file_exists(&flat_file_path) {
            let mut flat = MarsImage::open(flat_file_path, instrument);

            if let Some(md) = &raw.metadata {
                if let Some(rect) = &md.subframe_rect {
                    flat.crop(
                        rect[0] as usize - 1,
                        rect[1] as usize - 1,
                        rect[2] as usize,
                        rect[3] as usize,
                    );
                }
            }

            raw.flatfield_with_flat(&flat);
        } else {
            eprintln!("Flat file not found: {}", flat_file_path);
            panic!("Flat file not found!");
        }

        vprintln!("Applying color weights...");
        raw.apply_weight(
            cal_context.red_scalar,
            cal_context.green_scalar,
            cal_context.blue_scalar,
        );

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

        // Trim off border pixels
        let crop_to_width = raw.image.width - 2;
        let crop_to_height = raw.image.height - 2;
        raw.image.crop(1, 1, crop_to_width, crop_to_height);

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context)
    }
}
