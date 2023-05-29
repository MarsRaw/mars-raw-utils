use crate::{
    calibfile, calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument,
    flatfield, marsimage::MarsImage, util, veprintln, vprintln,
};

use anyhow::Result;
use sciimg::{imagebuffer, path};

#[derive(Copy, Clone)]
pub struct M20SuperCam {}

impl Calibration for M20SuperCam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::M20SuperCam)
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

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20SuperCam);
        vprintln!("Destretching...");
        raw.destretch_image();

        vprintln!("Loading image mask");
        let mask_file_path = calibfile::get_calibration_file_for_instrument(
            enums::Instrument::M20SuperCam,
            enums::CalFileType::Mask,
        )
        .unwrap();
        vprintln!("Loading supercam mask from {}", mask_file_path);
        let mut mask = imagebuffer::ImageBuffer::from_file(mask_file_path.as_str()).unwrap();
        mask = mask
            .get_subframe(1, 1, mask.width - 2, mask.height - 2)
            .expect("Failed to extract subframe of M20 SuperCam mask image");
        raw.apply_alpha(&mask);

        let data_max = if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            let lut = decompanding::get_ilt_for_instrument(enums::Instrument::M20SuperCam)
                .expect("Failed to determine ILT for M20 SuperCam");
            raw.decompand(&lut);
            lut.max() as f32
        } else {
            255.0
        };

        if input_file.contains("ECM") && raw.image.is_grayscale() {
            vprintln!("Image appears to be grayscale, applying debayering...");
            raw.debayer_with_method(cal_context.debayer_method);
        }

        // Gonna start with standard rectangular flat field, but should really
        // mask it to just the round light-collecting area of the image.
        raw.image
            .crop(1, 1, raw.image.width - 2, raw.image.height - 2);
        vprintln!("Flatfielding...");
        let mut flat = flatfield::load_flat(enums::Instrument::M20SuperCam)
            .expect("Failed to load flatfield image for M20 SuperCam");
        flat.image
            .crop(1, 1, flat.image.width - 2, flat.image.height - 2);
        raw.flatfield_with_flat(&flat);

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
        raw.image.set_using_alpha(true);
        match raw.save(&out_file) {
            Ok(_) => cal_ok(cal_context, &out_file),
            Err(why) => {
                veprintln!("Error saving file: {}", why);
                cal_fail(cal_context, &out_file)
            }
        }
    }
}
