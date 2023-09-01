use crate::{calibfile, enums, marsimage::MarsImage, memcache::load_image};

use anyhow::Result;

pub fn load_flat(instrument: enums::Instrument) -> Result<MarsImage> {
    match calibfile::get_calibration_file_for_instrument(instrument, enums::CalFileType::FlatField)
    {
        Ok(cal_file) => {
            info!("Loading calibration file from {}", cal_file);
            Ok(MarsImage::from_image(
                &load_image(&cal_file).unwrap(),
                instrument,
            ))
        }
        Err(e) => Err(e),
    }
}
