use crate::{
    time,
    constants,
    error
};

const TAI_OFFSET: f64 = 37.0;
const M2020_LONGITUDE: f64 = 77.43;
const M2020_SOL_OFFSET: f64 = -52303.0;

fn within_24(n:f64) -> f64 {
    let mut _n = n;
    if _n < 0.0 {
        _n += 24.0;
    } else if _n >= 24.0 {
        _n -= 24.0;
    }
    _n
}

// Based on m2020-bitbar which in turn is based on James Tauber's Mars Clock
pub fn get_lmst() -> error::Result<time::MissionTime> {

    let seconds_since_epoch = time::get_seconds_since_epoch();
    let millis = seconds_since_epoch * 1000.0;

    let jd_ut = 2440587.5 + (millis / 8.64E7);
    let jd_tt = jd_ut + (TAI_OFFSET + 32.184) / 86400.0;
    let j2000 = jd_tt - 2451545.0;
    let msd = ((j2000 - 4.5) / 1.027491252) + 44796.0 - 0.00096;
    let mtc = (24.0 * msd) % 24.0;

    let lambda = 360.0 - M2020_LONGITUDE;
    let sol = ((msd - lambda / 360.0) + M2020_SOL_OFFSET).floor();
    let lmst = within_24(mtc - lambda * 24.0 / 360.0);

    let hours = lmst.floor();
    let minutes_f = 60.0 * (lmst - hours);
    let minutes = minutes_f.floor();
    let seconds = (60.0 * (minutes_f - minutes)).floor();

    // VALIDATE THIS SECTION. I'M JUST GUESSING
    let unix_count = seconds_since_epoch - constants::time::M20_UNIX_COUNT_OFFSET;
    let display_sclk = constants::time::M20_SURFACE_SCLK + unix_count + 2.0;
    let lmst_string = format!("Sol {} {:02}:{:02}:{:02.3} (SCLK = {:.0})", sol, hours, minutes, seconds, display_sclk);

    Ok(time::MissionTime{
        display: lmst_string,
        sols: sol as i32,
        hours: hours as i32,
        minutes: minutes as i32,
        seconds: seconds as f32,
        sclk: display_sclk as i32
    })
}