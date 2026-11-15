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

    #[arg(
        long,
        short = 's',
        help = "Image size used as length & height for polar projections (default: 2048)"
    )]
    size: Option<usize>,

    #[arg(
        long,
        short = 'd',
        help = "Maximum distance (meters) for polar projections (default: 25.0)"
    )]
    distance: Option<f64>,

    #[arg(long, short = 'p', help = "Mosaic projection (default: cylindrical)")]
    projection: Option<Projection>,

    #[arg(
        long,
        short = 'e',
        help = "Eye height (meters) for polar projections (default: 1.98)"
    )]
    eye_height: Option<f64>,
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
        let size = self.size.unwrap_or(2048);
        let max_radius = self.distance.unwrap_or(25.0);
        let projection = self.projection.unwrap_or(Projection::Cylindrical);
        let eye_height = self.eye_height.unwrap_or(1.98);

        let canvas = create_composite(
            &in_files,
            self.anaglyph,
            azimuth_rotation,
            size,
            max_radius,
            eye_height,
            projection,
        )?;
        canvas.save(output).expect("Failed to save image");

        pb_done!();
        Ok(())
    }
}
