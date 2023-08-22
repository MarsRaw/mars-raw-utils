use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use sciimg::prelude::*;
use std::process;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Red/Blue colorizer for SHERLOC", long_about = None)]
pub struct M20SherlocColorizer {
    #[arg(long, short, help = "Image for red channel")]
    red: std::path::PathBuf,

    #[arg(long, short, help = "Image for blue channel")]
    blue: std::path::PathBuf,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,
}

#[async_trait::async_trait]
impl RunnableSubcommand for M20SherlocColorizer {
    async fn run(&self) -> Result<()> {
        pb_set_print!();

        if !self.red.exists() {
            eprintln!("Error: Image for red channel not found: {:?}", self.red);
            process::exit(1);
        }

        if !self.blue.exists() {
            eprintln!("Error: Image for blue channel not found: {:?}", self.blue);
            process::exit(1);
        }

        let red = MarsImage::open(
            self.red.as_os_str().to_string_lossy().as_ref(),
            Instrument::M20SherlocAci,
        );

        let blue = MarsImage::open(
            self.blue.as_os_str().to_string_lossy().as_ref(),
            Instrument::M20SherlocAci,
        );

        if red.image.width != blue.image.width || red.image.height != blue.image.height {
            eprintln!("Error: Input image dimension mismatch.");
            process::exit(2);
        }

        if red.image.get_mode() != blue.image.get_mode() {
            eprintln!("Error: Input images are of differing color modes.");
            process::exit(3);
        }

        // Add red and blue together, divide that by 2.0 (mean of the two)
        let green = red
            .image
            .get_band(0)
            .add(blue.image.get_band(0))?
            .divide_into(2.0)?;

        let colorized = Image::new_from_buffers_rgb(
            red.image.get_band(0),
            &green,
            blue.image.get_band(0),
            red.image.get_mode(),
        )?;

        colorized.save(self.output.to_string_lossy().as_ref())?;

        Ok(())
    }
}
