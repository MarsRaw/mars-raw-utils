use crate::{constants, time};

use anyhow::Result;

pub fn get_lmst() -> Result<time::MissionTime> {
    time::get_time(
        constants::time::M2020_SOL_OFFSET,
        constants::time::M2020_LONGITUDE,
        time::TimeSystem::LMST,
    )
}
