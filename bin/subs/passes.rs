use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use cli_table::{Cell, Style, Table};
use mars_raw_utils::enums::Mission;
use mars_raw_utils::passes::fetch_passes;
use mars_raw_utils::time::get_lmst;

#[derive(Parser)]
#[command(author, version, about = "Retrieve overflight information", long_about = None)]
pub struct Passes {
    #[arg(long, short, help = "Limit to orbiter(s)", num_args = 1..)]
    orbiter: Vec<String>,

    #[arg(long, short, help = "Limit to lander(s)", num_args = 1..)]
    lander: Vec<String>,

    #[arg(long, short, help = "Limit to future overflights")]
    future: bool,

    #[arg(long, short, help = "Has predicted data")]
    predict: bool,

    #[arg(long, short, help = "Has actual data")]
    actual: bool,

    #[arg(long, short, help = "Maximum result rows")]
    max_results: Option<usize>,
}

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
                    && (!self.future || now < overflight.start_time)
                    && (!self.predict || self.predict && overflight.predict.is_some())
                    && (!self.actual || self.actual && overflight.actual.is_some())
            })
            .take(match &self.max_results {
                Some(max_results) => max_results.to_owned(),
                None => 100000, // Kinda arbitrary...
            })
            .map(|overflight| {
                vec![
                    overflight.overflight_id.clone().cell(),
                    overflight.spacecraft_orbiter.clone().cell(),
                    overflight.spacecraft_lander.clone().cell(),
                    format!("{:.19}", overflight.start_time.to_string()).cell(),
                    match overflight.spacecraft_lander.as_str() {
                        "MSL" => {
                            if let Ok(lmst) = get_lmst(Mission::MSL, Some(overflight.start_time)) {
                                format!("SOL-{}T{:.12}", lmst.sol, lmst.ltst_display).cell()
                            } else {
                                "".cell()
                            }
                        }
                        "M20" => {
                            if let Ok(lmst) =
                                get_lmst(Mission::Mars2020, Some(overflight.start_time))
                            {
                                format!("SOL-{}T{:.12}", lmst.sol, lmst.ltst_display).cell()
                            } else {
                                "".cell()
                            }
                        }
                        _ => "".cell(),
                    },
                    format!("{:.19}", overflight.end_time.to_string()).cell(),
                    match overflight.spacecraft_lander.as_str() {
                        "MSL" => {
                            if let Ok(lmst) = get_lmst(Mission::MSL, Some(overflight.end_time)) {
                                format!("SOL-{}T{:.12}", lmst.sol, lmst.ltst_display).cell()
                            } else {
                                "".cell()
                            }
                        }
                        "M20" => {
                            if let Ok(lmst) = get_lmst(Mission::Mars2020, Some(overflight.end_time))
                            {
                                format!("SOL-{}T{:.12}", lmst.sol, lmst.ltst_display).cell()
                            } else {
                                "".cell()
                            }
                        }
                        _ => "".cell(),
                    },
                    overflight.maximum_elevation.cell(),
                    if let Some(pre) = &overflight.predict {
                        format!("{:.3}", pre.return_volume).cell()
                    } else {
                        "".cell()
                    },
                    if let Some(act) = &overflight.actual {
                        format!("{:.3}", act.return_volume).cell()
                    } else {
                        "".cell()
                    },
                ]
            })
            .collect::<Vec<_>>()
            .table()
            .title(vec![
                "ID".cell().bold(true),
                "Orbiter".cell().bold(true),
                "Lander".cell().bold(true),
                "Start Time".cell().bold(true),
                "Start LTST".cell().bold(true),
                "End Time".cell().bold(true),
                "End LTST".cell().bold(true),
                "Max El (deg)".cell().bold(true),
                "Pre Vol (MB)".cell().bold(true),
                "Act Vol (MB)".cell().bold(true),
                //"Duration".cell().bold(true),
                //"Range".cell().bold(true),
                //"Data Vol".cell().bold(true),
            ]);
        println!("{}", table.display().unwrap());

        Ok(())
    }
}
