use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use sciimg::enums::ImageMode;
use sciimg::image::Image;

use anyhow::anyhow;

use crate::marsimage::MarsImage;
use crate::subs::runnable::RunnableSubcommand;
use crate::{caldata, pb_create, pb_inc, pb_set_length, pb_set_print};

pb_create!();

#[derive(Parser)]
#[command(author, version, about = "Compose RGB image from monochrome frames", long_about = None)]
pub struct Compose {
    #[arg(long, short, help = "Red band input image")]
    red: PathBuf,

    #[arg(long, short, help = "Green band input image")]
    green: PathBuf,

    #[arg(long, short, help = "Blue band input image")]
    blue: PathBuf,

    #[arg(long, short, help = "Output image")]
    output: PathBuf,

    #[arg(long, short = 'R', help = "Red weight")]
    red_weight: Option<f32>,

    #[arg(long, short = 'G', help = "Green weight")]
    green_weight: Option<f32>,

    #[arg(long, short = 'B', help = "Blue weight")]
    blue_weight: Option<f32>,
}

impl RunnableSubcommand for Compose {
    async fn run(&self) -> Result<()> {
        pb_set_print!();

        let red = if let Ok(img) = Image::open(self.red.as_os_str().to_str().unwrap()) {
            img
        } else {
            return Err(anyhow!("Failed to load red band image: {:?}", self.red));
        };

        let green = if let Ok(img) = Image::open(self.green.as_os_str().to_str().unwrap()) {
            img
        } else {
            return Err(anyhow!("Failed to load green band image: {:?}", self.green));
        };

        let blue = if let Ok(img) = Image::open(self.blue.as_os_str().to_str().unwrap()) {
            img
        } else {
            return Err(anyhow!("Failed to load blue band image: {:?}", self.blue));
        };

        let red_weight = self.red_weight.unwrap_or(1.0);
        let green_weight = self.green_weight.unwrap_or(1.0);
        let blue_weight = self.blue_weight.unwrap_or(1.0);

        let mut rgb = if let Ok(img) = Image::new_from_buffers_rgb(
            red.get_band(0),
            green.get_band(0),
            blue.get_band(0),
            red.get_mode(),
        ) {
            img
        } else {
            return Err(anyhow!("Failed to compose RGB image from bands"));
        };

        vprintln!("Applying red channel weight of {}", red_weight);
        rgb.apply_weight_on_band(red_weight, 0);

        vprintln!("Applying green channel weight of {}", green_weight);
        rgb.apply_weight_on_band(green_weight, 1);

        vprintln!("Applying blue channel weight of {}", blue_weight);
        rgb.apply_weight_on_band(blue_weight, 2);

        if let Some(output_path) = self.output.as_os_str().to_str() {
            vprintln!("Saving output file to {}", output_path);
            rgb.save(output_path)
        } else {
            Err(anyhow!("Failed to parse output path"))
        }
    }
}
