use std::str::FromStr;

use mars_raw_utils::prelude::*;
use sciimg::prelude::*;

use crate::subs::runnable::RunnableSubcommand;

use clap::Parser;

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
    async fn run(&self) {
        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);

                let mut raw =
                    Image::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                let out_file =
                    util::append_file_name(in_file.as_os_str().to_str().unwrap(), "debayer");

                if !raw.is_grayscale() {
                    vprintln!(
                        "WARNING: Image doesn't appear to be grayscale as would be expected."
                    );
                    vprintln!("Results may be inaccurate");
                }

                let debayer_method = if let Some(debayer) = &self.debayer {
                    DebayerMethod::from_str(debayer.as_str()).unwrap_or(DebayerMethod::Malvar)
                } else {
                    DebayerMethod::Malvar
                };

                vprintln!("Debayering image...");
                raw.debayer_with_method(debayer_method);

                vprintln!("Writing to disk...");
                raw.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }
    }
}
