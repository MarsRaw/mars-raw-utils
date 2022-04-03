use mars_raw_utils::prelude::*;

fn main() {
    match m20::lmst::get_lmst() {
        Ok(mtime) => {
            println!("Mars Sol Date:          {}", mtime.msd);
            println!("Coordinated Mars Time:  {}", mtime.mtc_display);
            println!("Mission Sol:            {}", mtime.sol);
            println!("Mission Time:           {}", mtime.lmst_display);
            println!("Local True Solar Time:  {}", mtime.ltst_display);
            println!("Solar Longitude:        {}", mtime.l_s);
        },
        Err(_e) => {
            eprintln!("Error calculating mission time");
        }
    }
}