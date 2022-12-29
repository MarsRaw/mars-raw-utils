use mars_raw_utils::prelude::*;
use rayon::prelude::*;
use sciimg::prelude::*;

use crate::subs::runnable::RunnableSubcommand;

#[derive(clap::Args)]
#[clap(author, version, about = "Decorrelation stretching", long_about = None)]
pub struct DecorrelationStretch {
    #[clap(
        long,
        short,
        parse(from_os_str),
        help = "Input images",
        multiple_values(true)
    )]
    input_files: Vec<std::path::PathBuf>,
}

trait NormalizeRgbImageSingleChannels {
    fn normalize_to(&mut self, to_min: f32, to_max: f32);
}

impl NormalizeRgbImageSingleChannels for RgbImage {
    fn normalize_to(&mut self, to_min: f32, to_max: f32) {
        for b in 0..self.num_bands() {
            let mm = self.get_band(b).get_min_max();
            self.set_band(
                &self
                    .get_band(b)
                    .normalize_force_minmax(to_min, to_max, mm.min, mm.max)
                    .unwrap(),
                b,
            );
        }
    }
}

#[async_trait::async_trait]
impl RunnableSubcommand for DecorrelationStretch {
    async fn run(&self) {
        self.input_files.par_iter().for_each(|in_file| {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);

                let mut image =
                    RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                image.normalize_to(0.0, 65535.0);
                image.set_mode(ImageMode::U16BIT);

                vprintln!("Writing to disk...");
                image.save(&util::append_file_name(
                    in_file.as_os_str().to_str().unwrap(),
                    "decorr",
                ));
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        });
    }
}
