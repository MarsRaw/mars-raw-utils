use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;

#[derive(Parser)]
#[command(author, version, about = "Get current Mars2020 mission date information", long_about = None)]
pub struct M20Date {}

impl RunnableSubcommand for M20Date {
    async fn run(&self) -> Result<()> {
        match time::get_lmst(Mission::Mars2020) {
            Ok(mtime) => {
                println!(
                    "Earth Time (UTC):       {}",
                    mtime.earth_time_utc.format("%a, %e %b %Y %T %Z")
                );
                println!(
                    "Earth DOY (UTC):        {}",
                    mtime.earth_time_utc.format("%Y-%jT%T%.3f")
                );
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
        Ok(())
    }
}
