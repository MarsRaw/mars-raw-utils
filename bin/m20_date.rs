use mars_raw_utils::{
    constants,
    time
};

fn main() {
    match time::calc_mission_time(constants::time::M20_SURFACE_SCLK, 
                                constants::time::M20_UNIX_COUNT_OFFSET,
                                constants::time::M20_SURFACE_SEC_OFFSET,
                                constants::time::M20_RATE_ADJUSTMENT) {
        Ok(mtime) => {
            println!("{}", mtime.display);
        },
        Err(_e) => {
            eprintln!("Error calculating mission time");
        }
    }
}