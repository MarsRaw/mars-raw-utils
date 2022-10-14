use crate::{constants, time};

use sciimg::error;

pub fn get_lmst_mer_b() -> error::Result<time::MissionTime> {
    time::get_time(
        constants::time::MER_MERB_SOL_OFFSET,
        constants::time::MER_MERB_LONGITUDE,
        time::TimeSystem::HLST,
    )
}

pub fn get_lmst_mer_a() -> error::Result<time::MissionTime> {
    time::get_time(
        constants::time::MER_MERA_SOL_OFFSET,
        constants::time::MER_MERA_LONGITUDE,
        time::TimeSystem::HLST,
    )
}
