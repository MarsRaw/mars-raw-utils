use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;

#[derive(Parser)]
#[command(author, version, about = "Get current MER mission date information", long_about = None)]
pub struct MerDate {}

#[async_trait::async_trait]
impl RunnableSubcommand for MerDate {
    async fn run(&self) -> Result<()> {
        match time::get_lmst(Mission::MerA) {
            Ok(mtime) => {
                println!("MER-A / Spirit:");
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
        };
        println!("-----------------------------------------------");
        match time::get_lmst(Mission::MerB) {
            Ok(mtime) => {
                println!("MER-B / Opportunity:");
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
        };
        Ok(())
    }
}
