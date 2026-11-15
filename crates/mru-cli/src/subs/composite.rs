use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mru::{composite::create_composite, enums::Projection};
use stump;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Create composite mosaic", long_about = None)]
pub struct Composite {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,

    #[arg(long, short, help = "Anaglyph mode")]
    anaglyph: bool,

    #[arg(
        long,
        short = 'r',
        help = "Azimuth rotation",
        allow_negative_numbers = true
    )]
    azimuth: Option<f64>,
}

impl RunnableSubcommand for Composite {
    async fn run(&self) -> Result<()> {
        pb_set_print!();

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        let output = self.output.as_os_str().to_str().unwrap();

        let azimuth_rotation: f64 = self.azimuth.unwrap_or(0.0);

        let canvas = create_composite(
            &in_files,
            self.anaglyph,
            azimuth_rotation,
            Projection::Cylindrical,
        )?;
        canvas.save(output).expect("Failed to save image");

        pb_done!();
        Ok(())
    }
}
