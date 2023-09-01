use mars_raw_utils::prelude::*;

use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use std::process;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about = "Report sols with new images", long_about = None)]
pub struct NsytLatest {
    #[arg(long, short, help = "List sols with new images only")]
    list: bool,
}

use async_trait::async_trait;
#[async_trait]
impl RunnableSubcommand for NsytLatest {
    async fn run(&self) -> Result<()> {
        if let Ok(latest) = remotequery::get_latest(Mission::InSight).await {
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
            error!("Error fetching latest data from InSight remote server");
            process::exit(1);
        }
        Ok(())
    }
}
