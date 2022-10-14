use crate::{constants, time};

use sciimg::error;

pub fn get_lmst() -> error::Result<time::MissionTime> {
    time::get_time(
        constants::time::MSL_SOL_OFFSET,
        constants::time::MSL_LONGITUDE,
        time::TimeSystem::LMST,
    )
}
