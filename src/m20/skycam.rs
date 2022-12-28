use crate::{
    calibrate::*, calprofile::CalProfile, enums, enums::Instrument, flatfield, image::MarsImage,
    path, util, vprintln,
};

use sciimg::error;

#[derive(Copy, Clone)]
pub struct M20SkyCam {}

impl Calibration for M20SkyCam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::M20SkyCam)
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

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20SkyCam);

        vprintln!("Flatfielding...");
        let flat = flatfield::load_flat(enums::Instrument::M20SkyCam).unwrap();
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

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(255.0);

        // Trim off border pixels
        vprintln!("Cropping border pixels...");
        let crop_to_width = raw.image.width - 34;
        let crop_to_height = raw.image.height - 2;
        raw.image.crop(18, 1, crop_to_width, crop_to_height);

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context)
    }
}
