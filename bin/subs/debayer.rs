use mars_raw_utils::{
    prelude::*
};
use sciimg::{
    prelude::*
};

use crate::subs::runnable::RunnableSubcommand;

#[derive(clap::Args)]
#[clap(author, version, about = "Batch image debayering", long_about = None)]
pub struct Debayer {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,
}   

impl RunnableSubcommand for Debayer {
    fn run(&self) {
        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);

                let mut raw = RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                let out_file = util::append_file_name(in_file.as_os_str().to_str().unwrap(), "debayer");

                if !raw.is_grayscale() {
                    vprintln!("WARNING: Image doesn't appear to be grayscale as would be expected.");
                    vprintln!("Results may be inaccurate");
                }

                vprintln!("Debayering image...");
                raw.debayer();

                vprintln!("Writing to disk...");
                raw.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }
    }
}