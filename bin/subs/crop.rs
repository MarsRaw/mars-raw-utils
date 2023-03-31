use mars_raw_utils::prelude::*;
use sciimg::prelude::*;

use crate::subs::runnable::RunnableSubcommand;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Batch image crop", long_about = None)]
pub struct Crop {
    #[clap(
        long,
        short,
        parse(from_os_str),
        help = "Input images",
        multiple_values(true)
    )]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, help = "Crop as x,y,width,height")]
    crop: String,
}
#[async_trait::async_trait]
impl RunnableSubcommand for Crop {
    async fn run(&self) {
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
                vprintln!("Processing File: {:?}", in_file);

                let mut raw =
                    Image::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                if x >= raw.width {
                    eprintln!(
                        "X parameter is out of bounds: {}. Must be between 0 and {}",
                        x,
                        raw.width - 1
                    );
                    process::exit(2);
                }

                if y >= raw.height {
                    eprintln!(
                        "Y parameter is out of bounds: {}. Must be between 0 and {}",
                        x,
                        raw.height - 1
                    );
                    process::exit(2);
                }

                if width > raw.width - x {
                    eprintln!("Specified width exceeds maximum allowable value");
                    process::exit(2);
                }

                if height > raw.height - y {
                    eprintln!("Specified height exceeds maximum allowable value");
                    process::exit(2);
                }

                let out_file =
                    util::append_file_name(in_file.as_os_str().to_str().unwrap(), "crop");

                vprintln!(
                    "Cropping with x={}, y={}, width={}, height={}",
                    x,
                    y,
                    width,
                    height
                );
                raw.crop(x, y, width, height);

                vprintln!("Saving output to {}", out_file);

                raw.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }
    }
}
