
use crate::{
    time,
    constants,
    error
};


pub fn get_lmst() -> error::Result<time::MissionTime> {
    let unix_sec = time::get_seconds_since_epoch();
    let unix_count = unix_sec - constants::time::MSL_UNIX_COUNT_OFFSET;

    let rate_adjusted = unix_count / constants::time::MSL_RATE_ADJUSTMENT;
    let leap_second = 2.0;
    let display_sclk = constants::time::MSL_SURFACE_SCLK + rate_adjusted + leap_second;

    let surface_sec = unix_sec - constants::time::MSL_SURFACE_SEC_OFFSET + leap_second;
    let surface_mars_sec = surface_sec / constants::time::MARS_SEC_ADJUSTMENT;

    let sols = (surface_mars_sec / (24.0 * 60.0 * 60.0)).floor();
    let mars_hours = ((surface_mars_sec - (sols * 24.0 * 60.0 * 60.0)) / (60.0 * 60.0)).floor();
    let mars_minutes = ((surface_mars_sec - (sols * 24.0 * 60.0 * 60.0) - (mars_hours * 60.0 * 60.0)) / 60.0).floor();
    let mars_seconds = surface_mars_sec - (sols * 24.0 * 60.0 * 60.0) - (mars_hours * 60.0 * 60.0) - (mars_minutes * 60.0);

    let lmst_string = format!("Sol {} {:02}:{:02}:{:02.3} (SCLK = {:.0})", sols, mars_hours, mars_minutes, mars_seconds, display_sclk);

    Ok(time::MissionTime{
        display: lmst_string,
        sols: sols as i32,
        hours: mars_hours as i32,
        minutes: mars_minutes as i32,
        seconds: mars_seconds as f32,
        sclk: display_sclk as i32
    })
}