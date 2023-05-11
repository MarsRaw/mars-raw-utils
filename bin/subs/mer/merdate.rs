use mars_raw_utils::prelude::*;

use crate::subs::runnable::RunnableSubcommand;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about = "Get current MER mission date information", long_about = None)]
pub struct MerDate {}

#[async_trait::async_trait]
impl RunnableSubcommand for MerDate {
    async fn run(&self) {
        match mer::missiontime::get_lmst_mer_a() {
            Ok(mtime) => {
                println!("MER-A / Spirit:");
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
        };
        println!("-----------------------------------------------");
        match mer::missiontime::get_lmst_mer_b() {
            Ok(mtime) => {
                println!("MER-B / Opportunity:");
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
        };
    }
}
