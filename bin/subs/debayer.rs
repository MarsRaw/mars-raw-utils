use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use sciimg::prelude::*;
use std::str::FromStr;

pb_create!();

#[derive(Parser)]
#[command(author, version, about = "Batch image debayering", long_about = None)]
pub struct Debayer {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short = 'D', help = "Debayer method (malvar, amaze)")]
    debayer: Option<String>,
}

#[async_trait::async_trait]
impl RunnableSubcommand for Debayer {
    async fn run(&self) -> Result<()> {
        pb_set_print_and_length!(self.input_files.len());

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                info!("Processing File: {:?}", in_file);

                let mut raw =
                    Image::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                let out_file =
                    util::append_file_name(in_file.as_os_str().to_str().unwrap(), "debayer");

                if !raw.is_grayscale() {
                    warn!("WARNING: Image doesn't appear to be grayscale as would be expected.");
                    warn!("Results may be inaccurate");
                }

                let debayer_method = if let Some(debayer) = &self.debayer {
                    DebayerMethod::from_str(debayer.as_str()).unwrap_or(DebayerMethod::Malvar)
                } else {
                    DebayerMethod::Malvar
                };

                info!("Debayering image...");
                raw.debayer_with_method(debayer_method);

                info!("Writing to disk...");
                raw.save(&out_file).expect("Failed to save image");
            } else {
                error!("File not found: {:?}", in_file);
            }
            pb_inc!();
        }
        Ok(())
    }
}
