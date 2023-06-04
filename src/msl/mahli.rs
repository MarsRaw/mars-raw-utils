use crate::{
    calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument, flatfield,
    marsimage::MarsImage, util,
};

use sciimg::prelude::*;

use anyhow::Result;

#[derive(Copy, Clone)]
pub struct MslMahli {}

impl Calibration for MslMahli {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::MslMAHLI)
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

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::MslMAHLI);

        if raw.image.width == 1632 && raw.image.height == 1200 {
            vprintln!("Cropping...");
            raw.image.crop(32, 16, 1584, 1184);
        } else if raw.image.width == 1648 && raw.image.height == 1200 {
            vprintln!("Cropping...");
            raw.image.crop(48, 16, 1584, 1184);
        }
        vprintln!(
            "Image width/height after cropping: {}x{}",
            raw.image.width,
            raw.image.height
        );

        //1648, 1200
        vprintln!("Inpainting...");
        raw.apply_inpaint_fix();

        let data_max = if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            let lut = decompanding::get_ilt_for_instrument(enums::Instrument::MslMAHLI).unwrap();
            raw.decompand(&lut);
            lut.max() as f32
        } else {
            255.0
        };

        vprintln!("Flatfielding...");
        let mut flat = flatfield::load_flat(enums::Instrument::MslMAHLI).unwrap();
        if flat.image.width == 1632 && flat.image.height == 1200 {
            flat.image.crop(32, 16, 1584, 1184);
        }
        flat.apply_inpaint_fix();

        if flat.image.width > raw.image.width {
            let x = (flat.image.width - raw.image.width) / 2;
            let y = (flat.image.height - raw.image.height) / 2;
            vprintln!(
                "Cropping flat with x/y/width/height: {},{} {}x{}",
                x,
                y,
                raw.image.width,
                raw.image.height
            );
            flat.image.crop(x, y, raw.image.width, raw.image.height);
        }

        raw.flatfield_with_flat(&flat);

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

        vprintln!("Cropping...");
        raw.image.crop(2, 3, 1580, 1180);

        vprintln!("Applying color weights...");
        raw.apply_weight(
            cal_context.red_scalar,
            cal_context.green_scalar,
            cal_context.blue_scalar,
        );

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
        match raw.save(&out_file) {
            Ok(_) => cal_ok(cal_context, &out_file),
            Err(why) => {
                veprintln!("Error saving file: {}", why);
                cal_fail(cal_context, &out_file)
            }
        }
    }
}
