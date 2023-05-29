use crate::{
    calibrate::*, calprofile::CalProfile, enums, enums::Instrument, marsimage::MarsImage, util,
    veprintln, vprintln,
};

use anyhow::Result;
use sciimg::path;

#[derive(Copy, Clone)]
pub struct M20EdlRdcam {}

impl Calibration for M20EdlRdcam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(instrument, Instrument::M20EdlRdcam)
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

        let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20EdlRdcam);

        let data_max = 255.0; /*if cal_context.apply_ilt {
                                  vprintln!("Decompanding...");
                                  raw.decompand(&decompanding::ILT);
                                  decompanding::get_max_for_instrument(enums::Instrument::M20EdlRdcam) as f32
                              } else {
                                  255.0
                              };*/

        vprintln!("Debayering...");
        raw.debayer();

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

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
