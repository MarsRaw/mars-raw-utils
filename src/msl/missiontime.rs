use crate::{constants, time};

use anyhow::Result;

pub fn get_lmst() -> Result<time::MissionTime> {
    time::get_time(
        constants::time::MSL_SOL_OFFSET,
        constants::time::MSL_LONGITUDE,
        time::TimeSystem::LMST,
    )
}
