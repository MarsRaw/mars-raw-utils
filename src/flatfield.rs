
use crate::{
        rgbimage::RgbImage, 
        error, 
        enums,
        calibfile
};

pub fn load_flat(instrument:enums::Instrument) -> error::Result<RgbImage> {
    match calibfile::get_calibration_file_for_instrument(instrument, enums::CalFileType::FlatField) {
        Ok(cal_file) => RgbImage::open(cal_file, instrument),
        Err(e) => return Err(e)
    }
}
