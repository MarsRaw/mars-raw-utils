use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use rayon::prelude::*;
use sciimg::prelude::*;
use std::process;

pb_create!();

#[derive(Parser)]
#[command(author, version, about = "Perform hot pixel detection and correction", long_about = None)]
pub struct HpcFilter {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short = 't', help = "HPC threshold")]
    threshold: Option<f32>,

    #[arg(long, short = 'w', help = "HPC window size")]
    window: Option<i32>,
}

#[async_trait::async_trait]
impl RunnableSubcommand for HpcFilter {
    async fn run(&self) -> Result<()> {
        pb_set_print_and_length!(self.input_files.len());

        let window_size = self.window.unwrap_or(3);

        let threshold = self.threshold.unwrap_or(0.0);

        if threshold < 0.0 {
            error!("Threshold cannot be less than zero!");
            process::exit(1);
        }

        self.input_files.par_iter().for_each(|in_file| {
            if in_file.exists() {
                info!("Processing File: {:?}", in_file);
                let mut raw =
                    Image::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                debug!(
                    "Hot pixel correction with variance threshold {}...",
                    threshold
                );
                raw.hot_pixel_correction(window_size, threshold);

                info!("Writing to disk...");

                let out_file = util::append_file_name(in_file.as_os_str().to_str().unwrap(), "hpc");
                raw.save(&out_file).expect("Failed to save image");
            } else {
                error!("File not found: {:?}", in_file);
            }
            pb_inc!();
        });

        Ok(())
    }
}
