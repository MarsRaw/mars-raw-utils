pub use crate::anaglyph;
pub use crate::calibrate::*;
pub use crate::calprofile::CalProfile;
pub use crate::constants;
pub use crate::decorr;
pub use crate::enums::*;
pub use crate::m20;
pub use crate::m20::fetch::M20Fetch;
pub use crate::marsimage::MarsImage;
pub use crate::max;
pub use crate::mer;
pub use crate::metadata;
pub use crate::min;
pub use crate::msl;
pub use crate::nsyt;
pub use crate::remotequery;
pub use crate::remotequery::FetchError;
pub use crate::time;
pub use crate::util;
use std::str::FromStr;

pub use stump::{
    format_complete, format_done, format_fail, format_warn, print_complete, print_done, print_fail,
    print_warn,
};

use lazy_static;

lazy_static! {
    static ref CALIBRATORS: Vec<CalContainer> = vec![
        CalContainer {
            calibrator: Box::new(msl::mcam::MslMastcam {})
        },
        CalContainer {
            calibrator: Box::new(msl::ccam::MslChemCam {})
        },
        CalContainer {
            calibrator: Box::new(msl::ecam::MslEcam {})
        },
        CalContainer {
            calibrator: Box::new(msl::mahli::MslMahli {})
        },
        CalContainer {
            calibrator: Box::new(msl::mardi::MslMardi {})
        },
        CalContainer {
            calibrator: Box::new(m20::ecam::M20EECam {})
        },
        CalContainer {
            calibrator: Box::new(m20::zcam::M20MastcamZ {})
        },
        CalContainer {
            calibrator: Box::new(m20::helinav::M20HeliNav {})
        },
        CalContainer {
            calibrator: Box::new(m20::helirte::M20HeliRte {})
        },
        CalContainer {
            calibrator: Box::new(m20::pixlmcc::M20Pixl {})
        },
        CalContainer {
            calibrator: Box::new(m20::scam::M20SuperCam {})
        },
        CalContainer {
            calibrator: Box::new(m20::skycam::M20SkyCam {})
        },
        CalContainer {
            calibrator: Box::new(m20::watson::M20Watson {})
        },
        CalContainer {
            calibrator: Box::new(m20::sherlocaci::M20SherlocAci {})
        },
        CalContainer {
            calibrator: Box::new(m20::cachecam::M20CacheCam {})
        },
        CalContainer {
            calibrator: Box::new(m20::edlrdcam::M20EdlRdcam {})
        },
        CalContainer {
            calibrator: Box::new(nsyt::icc::NsytIcc {})
        },
        CalContainer {
            calibrator: Box::new(nsyt::idc::NsytIdc {})
        },
    ];
}

pub fn calibrator_for_instrument(instrument: Instrument) -> Option<&'static CalContainer> {
    CALIBRATORS
        .iter()
        .find(|&calibrator| calibrator.calibrator.accepts_instrument(instrument))
}

pub fn calibrator_for_instrument_from_str(instrument: &str) -> Option<&'static CalContainer> {
    calibrator_for_instrument(Instrument::from_str(instrument).unwrap())
}

use backtrace::Backtrace;
use std::panic;

pub fn init_panic_handler() {
    panic::set_hook(Box::new(|_info| {
        if stump::is_verbose() {
            println!("{:?}", Backtrace::new());
        }
    }));
}
