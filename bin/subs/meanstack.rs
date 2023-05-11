use mars_raw_utils::prelude::*;
use sciimg::prelude::*;

use crate::subs::runnable::RunnableSubcommand;

use std::process;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about = "Compute the mean of a series of images", long_about = None)]
pub struct MeanStack {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,
}

#[async_trait::async_trait]
impl RunnableSubcommand for MeanStack {
    async fn run(&self) {
        let output = self.output.as_os_str().to_str().unwrap();

        let mut mean: Image = Image::new_empty().unwrap();
        let mut count: ImageBuffer = ImageBuffer::new_empty().unwrap();
        let mut ones: ImageBuffer = ImageBuffer::new_empty().unwrap();

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);

                let raw =
                    Image::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                if mean.is_empty() {
                    mean = raw;
                    count = ImageBuffer::new(mean.width, mean.height).unwrap();
                    ones = ImageBuffer::new_with_fill(mean.width, mean.height, 1.0).unwrap();
                } else {
                    if raw.width != mean.width || raw.height != mean.height {
                        eprintln!("Input image has differing dimensions, cannot continue");
                        process::exit(1);
                    }

                    mean.add(&raw);
                }

                count = count.add(&ones).unwrap();
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }

        if !mean.is_empty() {
            mean.divide_from_each(&count);

            if path::parent_exists_and_writable(output) {
                vprintln!("Writing image to {}", output);
                mean.save(output);
            } else {
                eprintln!("Unable to write output image, parent doesn't exist or is not writable");
            }
        } else {
            println!("No images processed, cannot create output");
        }
    }
}
