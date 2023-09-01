use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use async_trait::async_trait;
use mars_raw_utils::prelude::*;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about = "Get current InSight mission date information", long_about = None)]
pub struct NsytDate {}

#[async_trait]
impl RunnableSubcommand for NsytDate {
    async fn run(&self) -> Result<()> {
        match time::get_lmst(Mission::InSight) {
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
        Ok(())
    }
}
