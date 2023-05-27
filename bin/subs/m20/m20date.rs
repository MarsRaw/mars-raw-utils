use mars_raw_utils::prelude::*;

use crate::subs::runnable::RunnableSubcommand;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about = "Get current Mars2020 mission date information", long_about = None)]
pub struct M20Date {}

#[async_trait::async_trait]
impl RunnableSubcommand for M20Date {
    async fn run(&self) {
        match time::get_lmst(Mission::Mars2020) {
            Ok(mtime) => {
                println!("Mars Sol Date:          {}", mtime.msd);
                println!("Coordinated Mars Time:  {}", mtime.mtc_display);
                println!("Mission Sol:            {}", mtime.sol);
                println!("Mission Time:           {}", mtime.mission_time_display);
                println!("Local True Solar Time:  {}", mtime.ltst_display);
                println!("Solar Longitude:        {}", mtime.l_s);
            }
            Err(_e) => {
                eprintln!("Error calculating mission time");
            }
        }
    }
}
