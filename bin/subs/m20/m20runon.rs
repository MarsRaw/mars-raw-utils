use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use std::collections::HashSet;
use std::process;

#[derive(Parser)]
#[command(author, version, about = "List sequences run on a specific sol", long_about = None)]
pub struct M20RunOn {
    #[arg(long, short, help = "Mars2020 Camera Instrument(s)", num_args = 1..)]
    camera: Vec<String>,

    #[arg(long, short = 's', help = "Mission Sol")]
    sol: i32,
}

#[async_trait::async_trait]
impl RunnableSubcommand for M20RunOn {
    async fn run(&self) -> Result<()> {
        let instruments = remotequery::get_instrument_map(Mission::Mars2020).unwrap();

        let camera_ids_res = instruments.find_remote_instrument_names_fromlist(&self.camera);
        let cameras = match camera_ids_res {
            Err(_e) => {
                error!("Invalid camera instrument(s) specified");
                process::exit(1);
            }
            Ok(v) => v,
        };
        // ZR3_0901_0746923185_303ECM_T0440898ZCAM03014_048300J
        let query = remotequery::RemoteQuery {
            cameras,
            num_per_page: 100,
            page: None,
            minsol: self.sol,
            maxsol: self.sol,
            movie_only: false,
            thumbnails: true,
            list_only: true,
            search: vec![],
            only_new: false,
            product_types: vec![],
            output_path: "".to_string(),
        };

        let mut sequences: HashSet<String> = HashSet::new();

        let available = remotequery::fetch_available(Mission::Mars2020, &query).await?;

        available
            .into_iter()
            .filter(|md| md.imageid.len() >= 45)
            .for_each(|md| {
                sequences.insert(md.imageid[35..44].to_string());
            });

        let mut seq_vec = sequences.into_iter().collect::<Vec<_>>();
        seq_vec.sort();
        seq_vec.into_iter().for_each(|seqid| {
            println!("{}", seqid);
        });
        Ok(())
    }
}
