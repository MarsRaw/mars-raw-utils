use crate::{
    constants,
    error
};

use std::time::{
    SystemTime, 
    UNIX_EPOCH
};

pub struct MissionTime {
    pub display: String,
    pub sols: i32,
    pub hours: i32,
    pub minutes: i32,
    pub seconds: f32,
    pub sclk: i32
}


pub fn get_seconds_since_epoch() -> f64 {
    let now = SystemTime::now();

    let unix_time = now.duration_since(UNIX_EPOCH).unwrap();
    let unix_secs =  unix_time.as_secs() as f64;
    let unix_millis = ((unix_time.as_nanos() % 1_000_000_000) as f64) / 1_000_000_000.0;
    let unix_sec = unix_secs as f64 + unix_millis as f64;

    unix_sec
}

pub fn calc_mission_time(surface_sclk:f64, unix_count_offset:f64, surface_sec_offset:f64, rate_adjustment:f64) -> error::Result<MissionTime> {
    // Needs testing in multiple time zones! (My system is set to UTC)
    let now = SystemTime::now();

    let unix_time = now.duration_since(UNIX_EPOCH).unwrap();
    let unix_secs =  unix_time.as_secs() as f64;
    let unix_millis = ((unix_time.as_nanos() % 1_000_000_000) as f64) / 1_000_000_000.0;
    let unix_sec = unix_secs as f64 + unix_millis as f64;
    let unix_count = unix_sec - unix_count_offset;

    let rate_adjusted = unix_count / rate_adjustment;
    let leap_second = 2.0;
    let display_sclk = surface_sclk + rate_adjusted + leap_second;

    let surface_sec = unix_sec - surface_sec_offset + leap_second;
    let surface_mars_sec = surface_sec / constants::time::MARS_SEC_ADJUSTMENT;

    let sols = (surface_mars_sec / (24.0 * 60.0 * 60.0)).floor();
    let mars_hours = ((surface_mars_sec - (sols * 24.0 * 60.0 * 60.0)) / (60.0 * 60.0)).floor();
    let mars_minutes = ((surface_mars_sec - (sols * 24.0 * 60.0 * 60.0) - (mars_hours * 60.0 * 60.0)) / 60.0).floor();
    let mars_seconds = surface_mars_sec - (sols * 24.0 * 60.0 * 60.0) - (mars_hours * 60.0 * 60.0) - (mars_minutes * 60.0);

    let lmst_string = format!("Sol {} {:02}:{:02}:{:02.3} (SCLK = {:.0})", sols, mars_hours, mars_minutes, mars_seconds, display_sclk);

    Ok(MissionTime{
        display: lmst_string,
        sols: sols as i32,
        hours: mars_hours as i32,
        minutes: mars_minutes as i32,
        seconds: mars_seconds as f32,
        sclk: display_sclk as i32
    })
}