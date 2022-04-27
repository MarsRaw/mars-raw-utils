
pub use crate::constants;
pub use crate::vprintln;
pub use crate::print;
pub use crate::path;
pub use crate::m20;
pub use crate::msl;
pub use crate::nsyt;
pub use crate::util;
pub use crate::min;
pub use crate::max;
pub use crate::image::MarsImage;
pub use crate::metadata;
pub use crate::enums::Mission;
pub use crate::enums::Instrument;
pub use crate::enums::CalFileType;

use std::panic;
use backtrace::Backtrace;

pub fn init_panic_handler() {
    panic::set_hook(Box::new(|_info| {
        if print::is_verbose() {
            println!("{:?}", Backtrace::new());  
        }
    }));
}