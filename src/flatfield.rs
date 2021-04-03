
use crate::{constants, rgbimage::RgbImage, error, enums};




pub fn load_flat(instrument:enums::Instrument) -> error::Result<RgbImage> {
    match instrument {
        enums::Instrument::MslMAHLI => Ok(RgbImage::open(constants::cal::MSL_MAHLI_FLAT_PATH, instrument).unwrap()),
        enums::Instrument::MslNavCamRight => Ok(RgbImage::open(constants::cal::MSL_NCAM_RIGHT_FLAT_PATH, instrument).unwrap()), 
        enums::Instrument::MslNavCamLeft => Ok(RgbImage::open(constants::cal::MSL_NCAM_LEFT_FLAT_PATH, instrument).unwrap()),
        enums::Instrument::MslFrontHazLeft => Ok(RgbImage::open(constants::cal::MSL_FHAZ_LEFT_FLAT_PATH, instrument).unwrap()),
        enums::Instrument::MslFrontHazRight => Ok(RgbImage::open(constants::cal::MSL_FHAZ_RIGHT_FLAT_PATH, instrument).unwrap()),
        enums::Instrument::MslRearHazLeft => Ok(RgbImage::open(constants::cal::MSL_RHAZ_LEFT_FLAT_PATH, instrument).unwrap()),
        enums::Instrument::MslRearHazRight => Ok(RgbImage::open(constants::cal::MSL_RHAZ_RIGHT_FLAT_PATH, instrument).unwrap()),
        _ => Err(constants::status::UNSUPPORTED_INSTRUMENT)
    }
}
