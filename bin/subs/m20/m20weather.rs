use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use cli_table::{Cell, Style, Table};
use mars_raw_utils::m20::weather::{self, MedaSol};

#[derive(Parser)]
#[command(author, version, about = "Return weather data from previous sols", long_about = None)]
pub struct M20Weather {
    #[arg(long, short, help = "Print CSV format")]
    csv: bool,
}

fn print_csv(meda_list: &[MedaSol]) {
    println!("Date,Sol,Max C,Min C,Pressure,Sunrise,Sunset,Season");
    meda_list.into_iter().for_each(|w| {
        println!(
            "{},{},{},{},{},{},{},{}",
            w.terrestrial_date,
            w.sol,
            w.max_temp,
            w.min_temp,
            w.pressure,
            w.sunrise,
            w.sunset,
            w.season
        );
    });
}

fn print_table(meda_list: &[MedaSol]) {
    let table = meda_list
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
            "Season".cell().bold(true),
        ]);

    println!("{}", &table.display().unwrap());
}

#[async_trait::async_trait]
impl RunnableSubcommand for M20Weather {
    async fn run(&self) -> Result<()> {
        let meda_list = weather::fetch_weather().await?;

        if self.csv {
            print_csv(&meda_list);
        } else {
            print_table(&meda_list);
        }

        println!("Source: Mars Environmental Dynamics Analyzer (MEDA)");
        println!("Credit: NASA/JPL-Caltech/CAB(CSIC-INTA)");
        Ok(())
    }
}
