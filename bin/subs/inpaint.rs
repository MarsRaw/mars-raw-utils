use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use sciimg::{inpaint, prelude::*};
use std::process;

pb_create!();

#[derive(Parser)]
#[command(author, version, about = "Perform an image inpaint repair", long_about = None)]
pub struct Inpaint {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,
}

#[async_trait::async_trait]
impl RunnableSubcommand for Inpaint {
    async fn run(&self) -> Result<()> {
        pb_set_print_and_length!(self.input_files.len());

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);

                let raw =
                    Image::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                vprintln!("Generating mask from red pixels...");
                let mask = inpaint::make_mask_from_red(&raw).unwrap();
                //mask.save("/data/MSL/inpaint_test/test-mask.png", enums::ImageMode::U8BIT).unwrap();

                vprintln!("Inpainting based on generated mask...");
                let filled = match inpaint::apply_inpaint_to_buffer_with_mask(&raw, &mask) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Error in inpainting process: {}", e);
                        process::exit(1);
                    }
                };

                let out_file =
                    util::append_file_name(in_file.as_os_str().to_str().unwrap(), "inpaint");

                vprintln!("Saving output to {}", out_file);

                filled.save(&out_file).expect("Failed to save image");
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
            pb_inc!();
        }
        Ok(())
    }
}
