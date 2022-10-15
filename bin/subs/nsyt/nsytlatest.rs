use mars_raw_utils::prelude::*;

use crate::subs::runnable::RunnableSubcommand;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Report sols with new images", long_about = None)]
pub struct NsytLatest {
    #[clap(long, short, help = "List sols with new images only")]
    list: bool,
}

use async_trait::async_trait;
#[async_trait]
impl RunnableSubcommand for NsytLatest {
    async fn run(&self) {
        let latest: nsyt::latest::LatestData = match nsyt::remote::fetch_latest().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "Error fetching latest data from InSight remote server: {}",
                    e
                );
                process::exit(1);
            }
        };

        if self.list {
            latest.latest_sols.iter().for_each(|s| {
                println!("{}", s);
            });
        } else {
            println!("Latest data: {}", latest.latest);
            println!("Latest sol: {}", latest.latest_sol);
            println!("Latest sols: {:?}", latest.latest_sols);
            println!("New Count: {}", latest.new_count);
            println!("Sol Count: {}", latest.sol_count);
            println!("Total: {}", latest.total);
        }
    }
}
