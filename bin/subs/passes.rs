use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use clap::Parser;
use cli_table::{Cell, Style, Table};
use mars_raw_utils::passes::fetch_passes;

#[derive(Parser)]
#[command(author, version, about = "Retrieve overflight information", long_about = None)]
pub struct Passes {
    #[arg(long, short, help = "Limit to orbiter(s)", num_args = 1..)]
    orbiter: Vec<String>,

    #[arg(long, short, help = "Limit to lander(s)", num_args = 1..)]
    lander: Vec<String>,

    #[arg(long, short, help = "Limit to future overflights")]
    future: bool,
}

#[async_trait]
impl RunnableSubcommand for Passes {
    async fn run(&self) -> Result<()> {
        let now = Utc::now();

        let passes = fetch_passes().await?;
        let table = passes
            .iter()
            .filter(|overflight| {
                (self.orbiter.is_empty()
                    || self
                        .orbiter
                        .iter()
                        .any(|o| overflight.spacecraft_orbiter.contains(o)))
                    && (self.lander.is_empty()
                        || self
                            .lander
                            .iter()
                            .any(|l| overflight.spacecraft_lander.contains(l)))
                    && !overflight.request_type.is_empty()
                    && (!self.future || now < overflight.start_time)
            })
            .map(|overflight| {
                vec![
                    overflight.overflight_id.clone().cell(),
                    overflight.spacecraft_orbiter.clone().cell(),
                    overflight.spacecraft_lander.clone().cell(),
                    overflight.maximum_elevation_time.cell(),
                    overflight.maximum_elevation.cell(),
                    overflight.rise_set_duration.cell(),
                    overflight.maximum_elevation_range.cell(),
                    overflight.request_data_volume_returned.cell(),
                ]
            })
            .collect::<Vec<_>>()
            .table()
            .title(vec![
                "ID".cell().bold(true),
                "Orbiter".cell().bold(true),
                "Lander".cell().bold(true),
                "Max El Time".cell().bold(true),
                "Max El (deg)".cell().bold(true),
                "Duration".cell().bold(true),
                "Range".cell().bold(true),
                "Data Vol".cell().bold(true),
            ]);
        println!("{}", &table.display().unwrap());

        Ok(())
    }
}
