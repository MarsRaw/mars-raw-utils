use crate::{
    calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument, flatfield,
    image::MarsImage, inpaintmask, path, util, vprintln,
};

use sciimg::{enums::ImageMode, error};

#[derive(Copy, Clone)]
pub struct MslMastcam {}

impl Calibration for MslMastcam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        match instrument {
            Instrument::MslMastcamLeft | Instrument::MslMastcamRight => true,
            _ => false,
        }
    }

    fn process_file(
        &self,
        input_file: &str,
        cal_context: &CalProfile,
        only_new: bool,
    ) -> error::Result<CompleteContext> {
        let out_file = util::append_file_name(input_file, &cal_context.filename_suffix.as_str());
        if path::file_exists(&out_file) && only_new {
            vprintln!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context);
        }

        let mut instrument = enums::Instrument::MslMastcamLeft;

        if util::filename_char_at_pos(&input_file, 5) == 'R' {
            instrument = enums::Instrument::MslMastcamRight;
            vprintln!("Processing for Mastcam Right");
        } else {
            vprintln!("Processing for Mastcam Left");
        }

        let mut raw = MarsImage::open(String::from(input_file), instrument);

        let mut data_max = 255.0;

        if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            raw.decompand(&decompanding::get_ilt_for_instrument(instrument));
            data_max = decompanding::get_max_for_instrument(instrument) as f32;
        }

        if
        /*util::filename_char_at_pos(&input_file, 22) == 'E' &&*/
        raw.image.is_grayscale() {
            vprintln!("Image appears to be grayscale, applying debayering...");
            raw.debayer();
        }

        let mut inpaint_mask = inpaintmask::load_mask(instrument).unwrap();
        let mut flat = flatfield::load_flat(instrument).unwrap();

        if raw.image.width == 1536 {
            raw.image.crop(161, 0, 1328, raw.image.height);
        }

        if raw.image.height == 1600 && raw.image.height == 1200 {
            raw.image.crop(125, 13, 1328, 1184);
        }

        vprintln!("Flatfielding...");

        if instrument == enums::Instrument::MslMastcamRight {
            if raw.image.width == 1328 && raw.image.height == 1184 {
                //x160, y16
                flat.image.crop(160, 16, 1328, 1184);
                inpaint_mask = inpaint_mask.get_subframe(160, 16, 1328, 1184).unwrap();
            } else if raw.image.width == 848 && raw.image.height == 848 {
                //x400, y192
                flat.image.crop(400, 192, 848, 848);
                inpaint_mask = inpaint_mask.get_subframe(400, 192, 848, 848).unwrap();
            } else if raw.image.width == 1344 && raw.image.height == 1200 {
                //x400, y192
                flat.image.crop(160, 0, 1344, 1200);
                inpaint_mask = inpaint_mask.get_subframe(160, 0, 1344, 1200).unwrap();
            }

            if raw.image.get_mode() == ImageMode::U8BIT {
                flat.image.normalize_to_12bit_with_max(
                    decompanding::get_max_for_instrument(instrument) as f32,
                    255.0,
                );
                flat.compand(&decompanding::get_ilt_for_instrument(instrument));
            }
        }

        if instrument == enums::Instrument::MslMastcamLeft {
            if raw.image.width == 1328 && raw.image.height == 1184 {
                //9
                flat.image.crop(160, 16, 1328, 1184);
                inpaint_mask = inpaint_mask.get_subframe(160, 16, 1328, 1184).unwrap();
            } else if raw.image.width == 1152 && raw.image.height == 432 {
                flat.image.crop(305, 385, 1152, 432);
                inpaint_mask = inpaint_mask.get_subframe(305, 385, 1152, 432).unwrap();
            } else if raw.image.width == 1600 && raw.image.height == 1200 {
                flat.image.crop(33, 0, 1600, 1200);
                inpaint_mask = inpaint_mask.get_subframe(33, 0, 1600, 1200).unwrap();
            } else if raw.image.width == 1456 && raw.image.height == 640 {
                flat.image.crop(96, 280, 1456, 640);
                inpaint_mask = inpaint_mask.get_subframe(96, 280, 1456, 640).unwrap();
            }

            if raw.image.get_mode() == ImageMode::U8BIT {
                flat.image.normalize_to_12bit_with_max(
                    decompanding::get_max_for_instrument(instrument) as f32,
                    255.0,
                );
                flat.compand(&decompanding::get_ilt_for_instrument(instrument));
            }
        }

        vprintln!(
            "Raw: {}/{}, Flat: {}/{}",
            raw.image.width,
            raw.image.height,
            flat.image.width,
            flat.image.height
        );

        // Catch some subframing edge cases
        if flat.image.width > raw.image.width {
            let x = (flat.image.width - raw.image.width) / 2;
            let y = (flat.image.height - raw.image.height) / 2;
            vprintln!(
                "Cropping flat/inpaint mask with x/y/width/height: {},{} {}x{}",
                x,
                y,
                raw.image.width,
                raw.image.height
            );
            flat.image.crop(x, y, raw.image.width, raw.image.height);
            inpaint_mask = inpaint_mask
                .get_subframe(x, y, raw.image.width, raw.image.height)
                .unwrap();
        }

        flat.apply_inpaint_fix_with_mask(&inpaint_mask);

        vprintln!(
            "Raw: {}/{}, Flat: {}/{}",
            raw.image.width,
            raw.image.height,
            flat.image.width,
            flat.image.height
        );

        raw.flatfield_with_flat(&flat);

        // Only inpaint with the same size as the mask until we can reliably determine
        // subframing sensor location.
        //if raw.image.width == 1328 && raw.image.height == 1184 {
        vprintln!("Inpainting...");
        raw.apply_inpaint_fix_with_mask(&inpaint_mask);
        //}

        vprintln!("Applying color weights...");
        raw.apply_weight(
            cal_context.red_scalar,
            cal_context.green_scalar,
            cal_context.blue_scalar,
        );

        if cal_context.color_noise_reduction && cal_context.color_noise_reduction_amount > 0 {
            vprintln!("Color noise reduction...");
            raw.image
                .reduce_color_noise(cal_context.color_noise_reduction_amount);
        }

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

        vprintln!("Cropping...");
        raw.image
            .crop(3, 3, raw.image.width - 6, raw.image.height - 6);

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context)
    }
}
