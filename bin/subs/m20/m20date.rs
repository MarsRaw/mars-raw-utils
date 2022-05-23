use mars_raw_utils::{
    prelude::*
};

use crate::subs::runnable::RunnableSubcommand;

#[derive(clap::Args)]
#[clap(author, version, about = "Get current Mars2020 mission date information", long_about = None)]
pub struct M20Date {}

impl RunnableSubcommand for M20Date {
    fn run(&self) {
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
}