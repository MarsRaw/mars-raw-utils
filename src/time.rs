use crate::{
    constants,
    error
};

use std::time::{
    SystemTime, 
    UNIX_EPOCH
};

fn within_24(n:f64) -> f64 {
    let mut _n = n;
    if _n < 0.0 {
        _n += 24.0;
    } else if _n >= 24.0 {
        _n -= 24.0;
    }
    _n
}

pub struct MissionTime {
    pub display: String,
    pub sols: i32,
    pub hours: i32,
    pub minutes: i32,
    pub seconds: f32,
    pub sclk: i32,
    pub msd: f64,
    pub mtc: f64
}


pub fn get_seconds_since_epoch() -> f64 {
    let now = SystemTime::now();

    let unix_time = now.duration_since(UNIX_EPOCH).unwrap();
    let unix_secs =  unix_time.as_secs() as f64;
    let unix_millis = ((unix_time.as_nanos() % 1_000_000_000) as f64) / 1_000_000_000.0;
    let unix_sec = unix_secs as f64 + unix_millis as f64;

    unix_sec
}


// NOTE: This isn't accurate.
pub fn get_lmst_from_epoch_secs(epoch:f64, longitude:f64) -> error::Result<MissionTime> {
    let jd_land = 2440587.5 + (epoch* 1000.0 / 8.64E7);
    let jd_tt_land = jd_land + (constants::time::TAI_OFFSET) / 86400.0;
    let j2000_land = jd_tt_land - 2451545.0 + 0.00014;
    let sol_offset = ((j2000_land - 4.5) / 1.027491252) + 44796.0 - 0.00096;
    
    get_lmst(-1.0 * sol_offset, longitude)
}


// Based on m2020-bitbar which in turn is based on James Tauber's Mars Clock
pub fn get_lmst(sol_offset:f64, longitude:f64) -> error::Result<MissionTime> {
    let seconds_since_epoch = get_seconds_since_epoch();
    let millis = seconds_since_epoch * 1000.0;

    let jd_ut = 2440587.5 + (millis / 8.64E7);
    let jd_tt = jd_ut + (constants::time::TAI_OFFSET + 32.184) / 86400.0;
    let j2000 = jd_tt - 2451545.0;
    let msd = ((j2000 - 4.5) / constants::time::MARS_SEC_ADJUSTMENT) + 44796.0 - 0.00096;
    let mtc = (24.0 * msd) % 24.0;

    let lambda = 360.0 - longitude;
    let sol = ((msd - lambda / 360.0) + sol_offset).round();
    let lmst = within_24(mtc - lambda * 24.0 / 360.0);

    let hours = lmst.floor();
    let minutes_f = 60.0 * (lmst - hours);
    let minutes = minutes_f.floor();
    let seconds = 60.0 * (minutes_f - minutes);

    // VALIDATE THIS SECTION. I'M JUST GUESSING
    // let unix_count = seconds_since_epoch - constants::time::M20_UNIX_COUNT_OFFSET;
    // let display_sclk = constants::time::M20_SURFACE_SCLK + unix_count + 2.0;
    
    let lmst_string = format!("Sol {} {:02}:{:02}:{:02.3} LMST", sol, hours, minutes, seconds);

    Ok(MissionTime{
        display: lmst_string,
        sols: sol as i32,
        hours: hours as i32,
        minutes: minutes as i32,
        seconds: seconds as f32,
        sclk: 0 as i32,
        msd: msd as f64,
        mtc: mtc as f64
    })
}