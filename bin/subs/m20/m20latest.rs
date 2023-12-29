use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::m20::fetch::M20Fetch as M20FetchClient;
use mars_raw_utils::prelude::*;
use std::process;

#[derive(Parser)]
#[command(author, version, about = "Report sols with new images", long_about = None)]
pub struct M20Latest {
    #[arg(long, short, help = "List sols with new images only")]
    list: bool,
}

impl RunnableSubcommand for M20Latest {
    async fn run(&self) -> Result<()> {
        let client = M20FetchClient::new();
        if let Ok(latest) = remotequery::get_latest(&client).await {
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
