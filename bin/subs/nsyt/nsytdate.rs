use crate::subs::runnable::RunnableSubcommand;
use mars_raw_utils::prelude::*;

use async_trait::async_trait;

#[derive(clap::Args)]
#[clap(author, version, about = "Get current InSight mission date information", long_about = None)]
pub struct NsytDate {}

#[async_trait]
impl RunnableSubcommand for NsytDate {
    async fn run(&self) {
        match nsyt::missiontime::get_lmst() {
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
