use crate::{constants, time};
use anyhow::Result;
use chrono::{DateTime, FixedOffset};

pub fn get_lmst(on_date: Option<DateTime<FixedOffset>>) -> Result<time::MissionTime> {
    time::get_time(
        constants::time::MSL_SOL_OFFSET,
        constants::time::MSL_LONGITUDE,
        time::TimeSystem::LMST,
        on_date,
    )
}
