use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use cli_table::{Cell, Style, Table};
use mars_raw_utils::dsnevents::fetch_dsn_events;

#[derive(Parser)]
#[command(author, version, about = "Retrieve DSN event information", long_about = None)]
pub struct DsnEvents {
    #[arg(long, short, help = "Limit to orbiter(s)", num_args = 1..)]
    orbiter: Vec<String>,

    #[arg(long, short, help = "Limit to dss(s)", num_args = 1..)]
    dss: Vec<String>,

    #[arg(long, short, help = "Limit to future events")]
    future: bool,

    #[arg(long, short, help = "Maximum result rows")]
    max_results: Option<usize>,
}

impl RunnableSubcommand for DsnEvents {
    async fn run(&self) -> Result<()> {
        let now = Utc::now();
        let dsnevents = fetch_dsn_events().await?;

        let table = dsnevents
            .iter()
            .filter(|event| {
                (self.orbiter.is_empty() || self.orbiter.iter().any(|o| event.orbiter.contains(o)))
                    && (self.dss.is_empty() || self.dss.iter().any(|l| event.dss.contains(l)))
                    && (!self.future || now < event.start_time)
            })
            .take(match &self.max_results {
                Some(max_results) => max_results.to_owned(),
                None => 100000, // Kinda arbitrary...
            })
            .map(|event| {
                vec![
                    event.orbiter.clone().cell(),
                    event.event_type.clone().cell(),
                    format!("{:.19}", event.start_time.to_string()).cell(),
                    format!("{:.19}", event.end_time.to_string()).cell(),
                    if event.dss.eq("NULL") {
                        "".cell()
                    } else {
                        event.dss.clone().cell()
                    },
                ]
            })
            .collect::<Vec<_>>()
            .table()
            .title(vec![
                "Orbiter".cell().bold(true),
                "Event Type".cell().bold(true),
                "Start Time".cell().bold(true),
                "End Time".cell().bold(true),
                "DSS".cell().bold(true),
            ]);

        println!("{}", table.display().unwrap());

        Ok(())
    }
}
