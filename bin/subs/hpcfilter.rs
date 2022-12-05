use crate::subs::runnable::RunnableSubcommand;
use mars_raw_utils::prelude::*;
use rayon::prelude::*;
use sciimg::prelude::*;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Perform hot pixel detection and correction", long_about = None)]
pub struct HpcFilter {
    #[clap(
        long,
        short,
        parse(from_os_str),
        help = "Input images",
        multiple_values(true)
    )]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short = 't', help = "HPC threshold")]
    threshold: Option<f32>,

    #[clap(long, short = 'w', help = "HPC window size")]
    window: Option<i32>,
}

#[async_trait::async_trait]
impl RunnableSubcommand for HpcFilter {
    async fn run(&self) {
        let window_size = self.window.unwrap_or(3);

        let threshold = self.threshold.unwrap_or(0.0);

        if threshold < 0.0 {
            eprintln!("Threshold cannot be less than zero!");
            process::exit(1);
        }

        self.input_files.par_iter().for_each(|in_file| {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);
                let mut raw =
                    RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                vprintln!(
                    "Hot pixel correction with variance threshold {}...",
                    threshold
                );
                raw.hot_pixel_correction(window_size, threshold);

                // DON'T ASSUME THIS!
                let data_max = 255.0;

                vprintln!("Normalizing...");
                raw.normalize_to_16bit_with_max(data_max);

                vprintln!("Writing to disk...");

                let out_file = util::append_file_name(in_file.as_os_str().to_str().unwrap(), "hpc");
                raw.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        });
    }
}
