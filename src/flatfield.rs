
use crate::{
        constants, 
        rgbimage::RgbImage, 
        error, 
        enums,
        calibfile
};

pub fn load_flat(instrument:enums::Instrument) -> error::Result<RgbImage> {
    match instrument {
        enums::Instrument::MslMAHLI => 
                Ok(RgbImage::open(calibfile::calibration_file(constants::cal::MSL_MAHLI_FLAT_PATH).unwrap(), instrument).unwrap()),
        enums::Instrument::MslNavCamRight => 
                Ok(RgbImage::open(calibfile::calibration_file(constants::cal::MSL_NCAM_RIGHT_FLAT_PATH).unwrap(), instrument).unwrap()), 
        enums::Instrument::MslNavCamLeft => 
                Ok(RgbImage::open(calibfile::calibration_file(constants::cal::MSL_NCAM_LEFT_FLAT_PATH).unwrap(), instrument).unwrap()),
        enums::Instrument::MslFrontHazLeft => 
                Ok(RgbImage::open(calibfile::calibration_file(constants::cal::MSL_FHAZ_LEFT_FLAT_PATH).unwrap(), instrument).unwrap()),
        enums::Instrument::MslFrontHazRight => 
                Ok(RgbImage::open(calibfile::calibration_file(constants::cal::MSL_FHAZ_RIGHT_FLAT_PATH).unwrap(), instrument).unwrap()),
        enums::Instrument::MslRearHazLeft => 
                Ok(RgbImage::open(calibfile::calibration_file(constants::cal::MSL_RHAZ_LEFT_FLAT_PATH).unwrap(), instrument).unwrap()),
        enums::Instrument::MslRearHazRight => 
                Ok(RgbImage::open(calibfile::calibration_file(constants::cal::MSL_RHAZ_RIGHT_FLAT_PATH).unwrap(), instrument).unwrap()),
        enums::Instrument::M20Watson =>
                Ok(RgbImage::open(calibfile::calibration_file(constants::cal::M20_WATSON_FLAT_PATH).unwrap(), instrument).unwrap()),
        enums::Instrument::M20SuperCam =>
                Ok(RgbImage::open(calibfile::calibration_file(constants::cal::M20_SCAM_FLAT_RGB_PATH).unwrap(), instrument).unwrap()),
        _ => Err(constants::status::UNSUPPORTED_INSTRUMENT)
    }
}
