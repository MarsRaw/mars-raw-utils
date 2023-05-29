use crate::{
    calibrate::*, calprofile::CalProfile, enums, enums::Instrument, marsimage::MarsImage, util,
    vprintln,
};

use sciimg::path;

use anyhow::Result;

#[derive(Copy, Clone)]
pub struct M20HeliNav {}

impl Calibration for M20HeliNav {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::M20HeliNav)
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

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20HeliNav);

        let data_max = 255.0;

        //if ! no_ilt {
        //    vprintln!("Decompanding...");
        //    raw.decompand().unwrap();
        //    data_max = decompanding::get_max_for_instrument(enums::Instrument::M20Watson) as f32;
        //}

        vprintln!("Flatfielding...");
        raw.flatfield();

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

        vprintln!("Writing to disk...");
        raw.save(&out_file).expect("Failed to save image");

        cal_ok(cal_context, &out_file)
    }
}
