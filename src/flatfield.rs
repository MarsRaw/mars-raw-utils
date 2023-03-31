use crate::{calibfile, enums, marsimage::MarsImage, vprintln};

use sciimg::error;

pub fn load_flat(instrument: enums::Instrument) -> error::Result<MarsImage> {
    match calibfile::get_calibration_file_for_instrument(instrument, enums::CalFileType::FlatField)
    {
        Ok(cal_file) => {
            vprintln!("Loading calibration file from {}", cal_file);
            Ok(MarsImage::open(cal_file, instrument))
        }
        Err(e) => Err(e),
    }
}
