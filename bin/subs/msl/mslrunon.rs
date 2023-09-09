use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use std::collections::HashSet;
use std::process;

#[derive(Parser)]
#[command(author, version, about = "List sequences run on a specific sol", long_about = None)]
pub struct MslRunOn {
    #[arg(long, short, help = "MSL Camera Instrument(s)", num_args = 0..)]
    camera: Vec<String>,

    #[arg(long, short = 's', help = "Mission Sol")]
    sol: i32,

    #[arg(long, short = 't', help = "Allow thumbnails in the results")]
    thumbnails: bool,
}

#[async_trait::async_trait]
impl RunnableSubcommand for MslRunOn {
    async fn run(&self) -> Result<()> {
        let instruments = remotequery::get_instrument_map(Mission::MSL).unwrap();

        let cameras = if self.camera.is_empty() {
            instruments.remote_instrument_names()
        } else {
            let camera_ids_res = instruments.find_remote_instrument_names_fromlist(&self.camera);
            match camera_ids_res {
                Err(_e) => {
                    error!("Invalid camera instrument(s) specified");
                    process::exit(1);
                }
                Ok(v) => v,
            }
        };

        let query = remotequery::RemoteQuery {
            cameras,
            num_per_page: 100,
            page: None,
            minsol: self.sol,
            maxsol: self.sol,
            movie_only: false,
            thumbnails: self.thumbnails,
            list_only: true,
            search: vec![],
            only_new: false,
            product_types: vec![],
            output_path: "".to_string(),
        };

        let mut sequences: HashSet<String> = HashSet::new();

        let available = remotequery::fetch_available(Mission::MSL, &query).await?;

        available
            .into_iter()
            .filter(|md| md.imageid.len() >= 36)
            .for_each(|md| {
                sequences.insert(md.imageid[25..34].to_string());
            });

        let mut seq_vec = sequences.into_iter().collect::<Vec<_>>();
        seq_vec.sort();
        seq_vec.into_iter().for_each(|seqid| {
            println!("{}", seqid);
        });
        Ok(())
    }
}
