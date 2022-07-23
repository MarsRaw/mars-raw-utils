

pub use crate::constants;
pub use crate::vprintln;
pub use crate::print;
pub use crate::path;
pub use crate::m20;
pub use crate::msl;
pub use crate::nsyt;
pub use crate::mer;
pub use crate::util;
pub use crate::min;
pub use crate::max;
pub use crate::image::MarsImage;
pub use crate::metadata;
pub use crate::calprofile::CalProfile;
pub use crate::calibrate::*;
pub use crate::enums::*;
pub use crate::drawable::*;
pub use crate::anaglyph;

pub use crate::print::{
    print_complete,
    print_done,
    print_warn,
    print_fail
};

use lazy_static;



lazy_static! {
    static ref CALIBRATORS:Vec<CalContainer> = vec![
        CalContainer{calibrator:Box::new(msl::mcam::MslMastcam{})},
        CalContainer{calibrator:Box::new(msl::ccam::MslChemCam{})},
        CalContainer{calibrator:Box::new(msl::ecam::MslEcam{})},
        CalContainer{calibrator:Box::new(msl::mahli::MslMahli{})},
        CalContainer{calibrator:Box::new(msl::mardi::MslMardi{})},

        CalContainer{calibrator:Box::new(m20::ecam::M20EECam{})},
        CalContainer{calibrator:Box::new(m20::zcam::M20MastcamZ{})},
        CalContainer{calibrator:Box::new(m20::helinav::M20HeliNav{})},
        CalContainer{calibrator:Box::new(m20::helirte::M20HeliRte{})},
        CalContainer{calibrator:Box::new(m20::pixlmcc::M20Pixl{})},
        CalContainer{calibrator:Box::new(m20::scam::M20SuperCam{})},
        CalContainer{calibrator:Box::new(m20::skycam::M20SkyCam{})},
        CalContainer{calibrator:Box::new(m20::watson::M20Watson{})},

        CalContainer{calibrator:Box::new(nsyt::icc::NsytIcc{})},
        CalContainer{calibrator:Box::new(nsyt::idc::NsytIdc{})}
    ];
}

pub fn calibrator_for_instrument(instrument:Instrument) -> Option<&'static CalContainer> {
    for calibrator in CALIBRATORS.iter() {
        if calibrator.calibrator.accepts_instrument(instrument) {
            return Some(calibrator);
        }
    }
    None
}

pub fn calibrator_for_instrument_from_str(instrument:&String) -> Option<&'static CalContainer> {
    calibrator_for_instrument(Instrument::from_str(&instrument.as_str()))
}



use std::panic;
use backtrace::Backtrace;

pub fn init_panic_handler() {
    panic::set_hook(Box::new(|_info| {
        if print::is_verbose() {
            println!("{:?}", Backtrace::new());  
        }
    }));
}