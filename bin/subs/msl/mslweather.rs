use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use cli_table::{Cell, Style, Table};
use mars_raw_utils::msl::weather;

#[derive(Parser)]
#[command(author, version, about = "Return weather data from previous sols", long_about = None)]
pub struct MslWeather {
    #[arg(long, short = 'a', help = "Print all results")]
    all: bool,
}

#[async_trait::async_trait]
impl RunnableSubcommand for MslWeather {
    async fn run(&self) -> Result<()> {
        let rems_list = weather::fetch_weather().await?;
        let limited_rems_list: Vec<_> = if self.all {
            rems_list.iter().rev().collect()
        } else {
            rems_list.iter().take(10).rev().collect()
        };

        let table = limited_rems_list
            .into_iter()
            .map(|w| {
                vec![
                    w.terrestrial_date.cell(),
                    w.sol.cell(),
                    w.max_temp.cell(),
                    w.min_temp.cell(),
                    w.pressure.cell(),
                    w.sunrise.clone().cell(),
                    w.sunset.clone().cell(),
                    w.atmo_opacity.clone().cell(),
                    w.season.clone().cell(),
                ]
            })
            .collect::<Vec<_>>()
            .table()
            .title(vec![
                "Date".cell().bold(true),
                "Sol".cell().bold(true),
                "Max (˚C)".cell().bold(true),
                "Min (˚C)".cell().bold(true),
                "Pressure (Pa)".cell().bold(true),
                "Sunrise".cell().bold(true),
                "Sunset".cell().bold(true),
                "Opacity".cell().bold(true),
                "Season".cell().bold(true),
            ]);

        println!("{}", &table.display().unwrap());
        println!("Source: Rover Environmental Monitoring Station (REMS)");
        println!("Credit: NASA/JPL-Caltech/CAB(CSIC-INTA)");
        Ok(())
    }
}
