use crate::{constants, time};
use anyhow::Result;
use chrono::{DateTime, FixedOffset};

pub fn get_lmst_mer_b(on_date: Option<DateTime<FixedOffset>>) -> Result<time::MissionTime> {
    time::get_time(
        constants::time::MER_MERB_SOL_OFFSET,
        constants::time::MER_MERB_LONGITUDE,
        time::TimeSystem::HLST,
        on_date,
    )
}

pub fn get_lmst_mer_a(on_date: Option<DateTime<FixedOffset>>) -> Result<time::MissionTime> {
    time::get_time(
        constants::time::MER_MERA_SOL_OFFSET,
        constants::time::MER_MERA_LONGITUDE,
        time::TimeSystem::HLST,
        on_date,
    )
}
