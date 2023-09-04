use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use std::collections::HashSet;
use std::process;

#[derive(Parser)]
#[command(author, version, about = "List sequences run on a specific sol", long_about = None)]
pub struct MslRunOn {
    #[arg(long, short, help = "MSL Camera Instrument(s)", num_args = 1..)]
    camera: Vec<String>,

    #[arg(long, short = 's', help = "Mission Sol")]
    sol: i32,
}

#[async_trait::async_trait]
impl RunnableSubcommand for MslRunOn {
    async fn run(&self) -> Result<()> {
        let instruments = remotequery::get_instrument_map(Mission::MSL).unwrap();

        let camera_ids_res = instruments.find_remote_instrument_names_fromlist(&self.camera);
        let cameras = match camera_ids_res {
            Err(_e) => {
                error!("Invalid camera instrument(s) specified");
                process::exit(1);
            }
            Ok(v) => v,
        };

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

        // let mut sequences: HashSet<String> = HashSet::new();

        let available = remotequery::fetch_available(Mission::MSL, &query).await?;
        println!("Number of images available: {}", available.len());

        available.into_iter().for_each(|md| {
            println!("Sequence: {}", md.imageid);
        });

        Ok(())
    }
}
