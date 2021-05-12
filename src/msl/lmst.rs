
use crate::{
    time,
    constants,
    error
};


pub fn get_lmst() -> error::Result<time::MissionTime> {
    time::get_lmst(constants::time::MSL_SOL_OFFSET, constants::time::MSL_LONGITUDE)
}