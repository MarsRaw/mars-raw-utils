use crate::{
    calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument, flatfield,
    image::MarsImage, inpaintmask, path, util, vprintln,
};

use sciimg::error;

#[derive(Copy, Clone)]
pub struct M20Watson {}

impl Calibration for M20Watson {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        match instrument {
            Instrument::M20Watson => true,
            _ => false,
        }
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

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20Watson);

        let mut data_max = 255.0;

        if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            raw.decompand(&decompanding::ILT);
            data_max = decompanding::get_max_for_instrument(enums::Instrument::M20Watson) as f32;
        }

        if input_file.contains("ECM") && raw.image.is_grayscale() {
            vprintln!("Image appears to be grayscale, applying debayering...");
            raw.debayer();
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

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

        if raw.image.width == 1648 {
            vprintln!("Cropping...");
            raw.image.crop(24, 4, 1600, 1192);
        }

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context)
    }
}
