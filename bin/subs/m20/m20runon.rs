use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use itertools::Itertools;
use mars_raw_utils::metadata::Metadata;
use mars_raw_utils::prelude::*;
use std::collections::HashMap;
use std::process;

#[derive(Parser)]
#[command(author, version, about = "List sequences run on a specific sol", long_about = None)]
pub struct M20RunOn {
    #[arg(long, short, help = "Mars2020 Camera Instrument(s)", num_args = 1..)]
    camera: Vec<String>,

    #[arg(long, short = 's', help = "Mission Sol")]
    sol: i32,

    #[arg(long, short = 't', help = "Allow thumbnails in the results")]
    thumbnails: bool,

    #[arg(long, short = 'C', help = "Show sequence and eye counts")]
    counts: bool,
}

struct SequenceStats {
    seqid: String,
    count: u32,
    left: u32,
    right: u32,
}

impl SequenceStats {
    pub fn new(seqid: &str, md: &Metadata) -> SequenceStats {
        let mut ss = SequenceStats {
            seqid: seqid.to_owned(),
            count: 0,
            left: 0,
            right: 0,
        };
        ss.add(md);
        ss
    }

    pub fn add(&mut self, md: &Metadata) {
        self.count += 1;
        if md.instrument.to_uppercase().contains("LEFT") {
            self.left += 1;
        } else if md.instrument.to_uppercase().contains("RIGHT") {
            // Some cameras are mono. They won't use left/right counts
            self.right += 1;
        }
    }
}

#[async_trait::async_trait]
impl RunnableSubcommand for M20RunOn {
    async fn run(&self) -> Result<()> {
        let instruments = remotequery::get_instrument_map(Mission::Mars2020).unwrap();

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

        let mut sequences: HashMap<String, SequenceStats> = HashMap::new();
        let available = remotequery::fetch_available(Mission::Mars2020, &query).await?;

        // sequences.insert(md.imageid[35..44].to_string());

        available
            .into_iter()
            .filter(|md| md.imageid.len() >= 36)
            .for_each(|md| {
                let seqid = md.imageid[35..44].to_string();

                if let Some(ss) = sequences.get_mut(&seqid) {
                    ss.add(&md)
                } else {
                    sequences.insert(seqid.clone(), SequenceStats::new(&seqid, &md));
                }
            });

        let mut seq_vec = sequences.values().collect_vec();
        seq_vec.sort_by_key(|ss| &ss.seqid);

        if self.counts {
            println!("{:10} {:>6} {:>6} {:>6}", "SeqID", "Left", "Right", "Total");
            seq_vec.into_iter().for_each(|ss| {
                println!(
                    "{:10} {:6} {:6} {:6}",
                    ss.seqid, ss.left, ss.right, ss.count
                );
            });
        } else {
            seq_vec.into_iter().for_each(|ss| {
                println!("{}", ss.seqid);
            });
        }

        Ok(())
    }
}
