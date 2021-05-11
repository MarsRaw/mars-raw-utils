use mars_raw_utils::{
    constants,
    time
};

fn main() {
    match time::calc_mission_time(constants::time::MSL_SURFACE_SCLK, 
                                            constants::time::MSL_UNIX_COUNT_OFFSET,
                                            constants::time::MSL_SURFACE_SEC_OFFSET,
                                            constants::time::MSL_RATE_ADJUSTMENT) {
        Ok(mtime) => {
            println!("{}", mtime.display);
        },
        Err(_e) => {
            eprintln!("Error calculating mission time");
        }
    }
}