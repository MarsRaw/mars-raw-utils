use mars_raw_utils::prelude::*;
use sciimg::prelude::*;

use crate::subs::runnable::RunnableSubcommand;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Adjust image levels", long_about = None)]
pub struct Levels {
    #[clap(
        long,
        short,
        parse(from_os_str),
        help = "Input images",
        multiple_values(true)
    )]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, help = "Black level")]
    black: Option<f32>,

    #[clap(long, short, help = "White level")]
    white: Option<f32>,

    #[clap(long, short, help = "Gamma level")]
    gamma: Option<f32>,
}

impl RunnableSubcommand for Levels {
    fn run(&self) {
        let white_level = self.white.unwrap_or(1.0);

        let black_level = self.black.unwrap_or(0.0);

        let gamma = self.gamma.unwrap_or(1.0);

        // Some rules on the parameters
        // TODO: Keep an eye on floating point errors
        if white_level < 0.0 || black_level < 0.0 {
            eprintln!("Levels cannot be negative");
            process::exit(1);
        }

        if white_level < black_level {
            eprintln!("White level cannot be less than black level");
            process::exit(1);
        }

        if white_level > 1.0 || black_level > 1.0 {
            eprintln!("Levels cannot exceed 1.0");
            process::exit(1);
        }

        if gamma <= 0.0 {
            eprintln!("Gamma cannot be zero or negative");
            process::exit(1);
        }

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);

                let mut raw =
                    RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                vprintln!(
                    "Black: {}, White: {}, Gamma: {}, {:?}",
                    black_level,
                    white_level,
                    gamma,
                    in_file
                );
                raw.levels(black_level, white_level, gamma);

                let out_file =
                    util::append_file_name(in_file.as_os_str().to_str().unwrap(), "lvls");
                raw.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }
    }
}
