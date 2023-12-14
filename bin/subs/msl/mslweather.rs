use std::fmt::Display;

use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use cli_table::{Cell, Style, Table};
use mars_raw_utils::msl::weather::{self, RemsSol};

#[derive(Parser)]
#[command(author, version, about = "Return weather data from previous sols", long_about = None)]
pub struct MslWeather {
    #[arg(long, short = 'a', help = "Print all results")]
    all: bool,

    #[arg(long, short, help = "Print CSV format")]
    csv: bool,
}

fn format_if_some<T: Display>(v: Option<T>) -> String {
    if v.is_some() {
        format!("{}", v.unwrap())
    } else {
        "--".to_string()
    }
}

fn print_csv(rems_list: &[&RemsSol]) {
    println!("Date,Sol,Max C,Min C,Pressure,Sunrise,Sunset,Opacity,Season");
    rems_list.iter().for_each(|w| {
        println!(
            "{},{},{},{},{},{},{},{},{}",
            w.terrestrial_date,
            w.sol,
            format_if_some(w.max_temp),
            format_if_some(w.min_temp),
            format_if_some(w.pressure),
            w.sunrise,
            w.sunset,
            w.atmo_opacity,
            w.season
        );
    });
}

fn print_table(rems_list: &[&RemsSol]) {
    let table = rems_list
        .iter()
        .map(|w| {
            vec![
                w.terrestrial_date.cell(),
                w.sol.cell(),
                format_if_some(w.max_temp).cell(),
                format_if_some(w.min_temp).cell(),
                format_if_some(w.pressure).cell(),
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

        if self.csv {
            print_csv(&limited_rems_list);
        } else {
            print_table(&limited_rems_list);
        }

        println!("Source: Rover Environmental Monitoring Station (REMS)");
        println!("Credit: NASA/JPL-Caltech/CAB(CSIC-INTA)");
        Ok(())
    }
}
