use crate::{
    time,
    constants,
    error
};

pub fn get_lmst() -> error::Result<time::MissionTime> {
    time::get_lmst(constants::time::NSYT_SOL_OFFSET, constants::time::NSYT_LONGITUDE)
}