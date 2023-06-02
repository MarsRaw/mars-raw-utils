use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use mars_raw_utils::prelude::*;
use std::process;

#[derive(Parser)]
#[command(author, version, about = "Report sols with new images", long_about = None)]
pub struct MslLatest {
    #[arg(long, short, help = "List sols with new images only")]
    list: bool,
}

#[async_trait]
impl RunnableSubcommand for MslLatest {
    async fn run(&self) -> Result<()> {
        if let Ok(latest) = remotequery::get_latest(Mission::MSL).await {
            if self.list {
                latest.latest_sols().iter().for_each(|s| {
                    println!("{}", s);
                });
            } else {
                println!("Latest data: {}", latest.latest());
                println!("Latest sol: {}", latest.latest_sol());
                println!("Latest sols: {:?}", latest.latest_sols());
                println!("New Count: {}", latest.new_count());
                println!("Sol Count: {}", latest.sol_count());
                println!("Total: {}", latest.total());
            }
        } else {
            eprintln!("Error fetching latest data from InSight remote server");
            process::exit(1);
        }
        Ok(())
    }
}
