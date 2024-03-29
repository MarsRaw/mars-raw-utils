use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use std::process;
pb_create!();

#[derive(Parser)]
#[command(author, version, about = "Batch image crop", long_about = None)]
pub struct Crop {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short, help = "Crop as x,y,width,height")]
    crop: String,
}

#[async_trait::async_trait]
impl RunnableSubcommand for Crop {
    async fn run(&self) -> Result<()> {
        pb_set_print_and_length!(self.input_files.len());

        //https://stackoverflow.com/questions/26536871/how-can-i-convert-a-string-of-numbers-to-an-array-or-vector-of-integers-in-rust
        let crop_numbers: Vec<usize> = self
            .crop
            .split(',')
            .map(|s| s.parse().expect("parse error"))
            .collect();

        if crop_numbers.len() != 4 {
            eprintln!("Invalid number of crop parameters specified.");
            process::exit(1);
        }

        let x = crop_numbers[0];
        let y = crop_numbers[1];
        let width = crop_numbers[2];
        let height = crop_numbers[3];

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                info!("Processing File: {:?}", in_file);

                let mut raw =
                    MarsImage::open(in_file.as_os_str().to_str().unwrap(), Instrument::None);

                if x >= raw.image.width {
                    error!(
                        "X parameter is out of bounds: {}. Must be between 0 and {}",
                        x,
                        raw.image.width - 1
                    );
                    process::exit(2);
                }

                if y >= raw.image.height {
                    error!(
                        "Y parameter is out of bounds: {}. Must be between 0 and {}",
                        x,
                        raw.image.height - 1
                    );
                    process::exit(2);
                }

                if width > raw.image.width - x {
                    error!("Specified width exceeds maximum allowable value");
                    process::exit(2);
                }

                if height > raw.image.height - y {
                    eprintln!("Specified height exceeds maximum allowable value");
                    process::exit(2);
                }

                let out_file =
                    util::append_file_name(in_file.as_os_str().to_str().unwrap(), "crop");

                info!(
                    "Cropping with x={}, y={}, width={}, height={}",
                    x, y, width, height
                );
                raw.crop(x, y, width, height);

                info!("Saving output to {}", out_file);

                raw.update_history();
                raw.save(&out_file).expect("Failed to save image");
            } else {
                error!("File not found: {:?}", in_file);
            }
            pb_inc!();
        }
        Ok(())
    }
}
