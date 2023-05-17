use crate::{constants, time};

use anyhow::Result;

pub fn get_lmst() -> Result<time::MissionTime> {
    time::get_time(
        constants::time::NSYT_SOL_OFFSET,
        constants::time::NSYT_LONGITUDE,
        time::TimeSystem::LMST,
    )
}
