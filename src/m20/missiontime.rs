use crate::{
    time,
    constants
};

use sciimg::error;

pub fn get_lmst() -> error::Result<time::MissionTime> {
    time::get_time(constants::time::M2020_SOL_OFFSET, constants::time::M2020_LONGITUDE, time::TimeSystem::LMST)
}