
use crate::{
        error, 
        enums,
        calibfile,
        image::MarsImage
};

pub fn load_flat(instrument:enums::Instrument) -> error::Result<MarsImage> {
    match calibfile::get_calibration_file_for_instrument(instrument, enums::CalFileType::FlatField) {
        Ok(cal_file) => Ok(MarsImage::open(cal_file, instrument)),
        Err(e) => Err(e)
    }
}
