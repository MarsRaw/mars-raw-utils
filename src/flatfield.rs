
use crate::{constants, rgbimage::RgbImage, error, enums};




pub fn load_flat(instrument:enums::Instrument) -> error::Result<RgbImage> {
    match instrument {
        enums::Instrument::MslMAHLI => Ok(RgbImage::open(constants::cal::MSL_MAHLI_FLAT_PATH, instrument).unwrap()),
        _ => Err(constants::status::UNSUPPORTED_INSTRUMENT)
    }
}
