use crate::{
    calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument, flatfield,
    inpaintmask, marsimage::MarsImage, util,
};

use sciimg::prelude::*;

use anyhow::Result;

#[derive(Copy, Clone)]
pub struct M20Watson {}

impl Calibration for M20Watson {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::M20Watson)
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

        let mut raw = MarsImage::open(input_file, enums::Instrument::M20Watson);

        let data_max = if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            let lut = decompanding::get_ilt_for_instrument(enums::Instrument::M20Watson).unwrap();
            raw.decompand(&lut);
            lut.max() as f32
        } else {
            255.0
        };

        if input_file.contains("ECM") && raw.image.is_grayscale() {
            vprintln!("Image appears to be grayscale, applying debayering...");
            raw.debayer_with_method(cal_context.debayer_method);
        }

        vprintln!("Flatfielding...");
        let mut flat = flatfield::load_flat(enums::Instrument::M20Watson).unwrap();
        if raw.image.width == 1584 && raw.image.height == 1184 {
            flat.image.crop(32, 16, 1584, 1184);
        }
        raw.flatfield_with_flat(&flat);

        vprintln!("Inpainting...");
        let mut inpaint_mask = inpaintmask::load_mask(enums::Instrument::M20Watson).unwrap();
        if raw.image.width == 1584 && raw.image.height == 1184 {
            inpaint_mask = inpaint_mask.get_subframe(32, 16, 1584, 1184).unwrap();
        }
        raw.apply_inpaint_fix_with_mask(&inpaint_mask);

        vprintln!("Applying color weights...");
        raw.apply_weight(
            cal_context.red_scalar,
            cal_context.green_scalar,
            cal_context.blue_scalar,
        );

        if raw.image.width == 1648 {
            vprintln!("Cropping...");
            raw.image.crop(24, 4, 1600, 1192);
        }

        if cal_context.srgb_color_correction {
            vprintln!("Applying sRGB color conversion");
            raw.image
                .convert_colorspace(color::ColorSpaceType::RGB, color::ColorSpaceType::sRGB)?;
        }

        if cal_context.decorrelate_color {
            vprintln!("Normalizing with decorrelated colors...");
            raw.image.normalize_to_16bit_decorrelated();
        } else {
            vprintln!("Normalizing with correlated colors...");
            raw.image.normalize_to_16bit_with_max(data_max);
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
