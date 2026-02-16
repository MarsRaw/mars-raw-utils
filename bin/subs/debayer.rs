use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use sciimg::prelude::*;
use std::path::Path;
use std::str::FromStr;

pb_create!();

#[derive(Parser)]
#[command(author, version, about = "Batch image debayering", long_about = None)]
pub struct Debayer {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short = 'D', help = "Debayer method (malvar, amaze)")]
    debayer: Option<String>,

    #[arg(
        long,
        help = "For JPEG inputs, zero AC[63] in each 8x8 DCT block in memory before decode"
    )]
    zero_final_ac: bool,
}

#[async_trait::async_trait]
impl RunnableSubcommand for Debayer {
    async fn run(&self) -> Result<()> {
        pb_set_print_and_length!(self.input_files.len());

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                info!("Processing File: {:?}", in_file);

                let in_file_str = String::from(in_file.as_os_str().to_str().unwrap());

                let mut raw = if self.zero_final_ac {
                    if is_jpeg_path(in_file.as_path()) {
                        super::jpeg_final_ac::load_jpeg_with_zeroed_final_ac(in_file.as_path())?
                    } else {
                        warn!(
                            "--zero-final-ac requested for non-JPEG input, falling back to default decoder: {:?}",
                            in_file
                        );
                        Image::open(&in_file_str)?
                    }
                } else {
                    Image::open(&in_file_str)?
                };

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
                raw.save(&out_file)?;
            } else {
                error!("File not found: {:?}", in_file);
            }
            pb_inc!();
        }
        Ok(())
    }
}

fn is_jpeg_path(path: &Path) -> bool {
    path.extension()
        .and_then(|v| v.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg"))
        .unwrap_or(false)
}
