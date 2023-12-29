use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::{constants::url, location};
use std::process;

#[derive(Parser)]
#[command(author, version, about = "Current reported Mars2020 location information", long_about = None)]
pub struct M20Location {
    #[arg(long, short, help = "Print all known waypoints")]
    all: bool,

    #[arg(long, short, help = "Print CSV format")]
    csv: bool,
}

impl RunnableSubcommand for M20Location {
    async fn run(&self) -> Result<()> {
        if !self.all && self.csv {
            eprintln!("Error: CSV can only be used with --all|-a");
            process::exit(1);
        }

        if self.all && self.csv {
            location::print_all_csv(url::M20_WAYPOINTS_URL).await?;
        } else if self.all {
            location::print_all(url::M20_WAYPOINTS_URL).await?;
        } else {
            location::print_single(url::M20_LOCATION_URL).await?;
        }
        Ok(())
    }
}
