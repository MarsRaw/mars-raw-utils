use crate::{
    calibfile,
    calibrate::*,
    calprofile::CalProfile,
    enums::{self, Instrument},
    inpaintmask,
    marsimage::MarsImage,
    metadata::MissionMetadata,
    util,
};

use sciimg::camera::cahvore::linearity;
use sciimg::path;

use anyhow::{anyhow, Result};

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
    ) -> Result<CompleteContext> {
        let out_file = util::append_file_name(input_file, cal_context.filename_suffix.as_str());
        if path::file_exists(&out_file) && only_new {
            vprintln!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context, &out_file);
        }

        let mut raw = MarsImage::open(input_file)?;
        let instrument = match raw.metadata.instrument() {
            Ok(instrument) => instrument,
            Err(why) => {
                return Err(anyhow!(
                    "Unable to determine camera instrument from metadata: {:?}",
                    why
                ))
            }
        };

        // Override default linearity
        match &mut raw.metadata.mission {
            MissionMetadata::Surface(md) => {
                md.camera_model_component_list.linearity = linearity::LINEARITY_PERSPECTIVE
            }
            _ => {
                return Err(anyhow!(
                    "Received nonsurface metadata when surface (rover) metadata was expected"
                ))
            }
        };

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
        )?;
        vprintln!("Using flat file: {}", flat_file_path);

        if path::file_exists(&flat_file_path) {
            let mut flat = MarsImage::open(&flat_file_path)?;

            if let Some(md) = raw.metadata.as_rover() {
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
        if cal_context.auto_subframing {
            let crop_to_width = raw.image.width - 2;
            let crop_to_height = raw.image.height - 2;
            raw.image.crop(1, 1, crop_to_width, crop_to_height);
        }

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
