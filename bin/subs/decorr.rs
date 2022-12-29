use crate::subs::runnable::RunnableSubcommand;
use mars_raw_utils::prelude::*;
use rayon::prelude::*;
use sciimg::prelude::*;
use sciimg::MinMax;
use std::path::PathBuf;
use std::process;

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
    #[clap(
        long,
        short,
        help = "Cross-File decorrelation (value ranges determined across all files rather than individually)"
    )]
    cross_file: bool,
}

trait NormalizeRgbImageSingleChannels {
    fn normalize_to(&mut self, to_min: f32, to_max: f32);
    fn normalize_band_to_with_min_max(
        &mut self,
        band: usize,
        to_min: f32,
        to_max: f32,
        from_min: f32,
        from_max: f32,
    );
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

    fn normalize_band_to_with_min_max(
        &mut self,
        band: usize,
        to_min: f32,
        to_max: f32,
        from_min: f32,
        from_max: f32,
    ) {
        self.set_band(
            &self
                .get_band(band)
                .normalize_force_minmax(to_min, to_max, from_min, from_max)
                .unwrap(),
            band,
        );
    }
}

fn cross_file_decorrelation(input_files: &Vec<PathBuf>) {
    let mut ranges = vec![
        MinMax {
            min: std::f32::MAX,
            max: std::f32::MIN,
        },
        MinMax {
            min: std::f32::MAX,
            max: std::f32::MIN,
        },
        MinMax {
            min: std::f32::MAX,
            max: std::f32::MIN,
        },
    ];

    vprintln!("Computing value ranges...");
    input_files.iter().for_each(|in_file| {
        if in_file.exists() {
            let image =
                RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

            for b in 0..3 {
                let mm = image.get_band(b).get_min_max();
                ranges[b].min = min!(mm.min, ranges[b].min);
                ranges[b].max = max!(mm.max, ranges[b].max);
            }
        } else {
            eprintln!("File not found: {:?}", in_file);
            process::exit(1);
        }
    });

    input_files.par_iter().for_each(|in_file| {
        if in_file.exists() {
            vprintln!("Processing File: {:?}", in_file);

            let mut image =
                RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

            for b in 0..3 {
                image.normalize_band_to_with_min_max(b, 0.0, 65535.0, ranges[b].min, ranges[b].max);
            }

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

fn individual_file_decorrelation(input_files: &Vec<PathBuf>) {
    input_files.par_iter().for_each(|in_file| {
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

#[async_trait::async_trait]
impl RunnableSubcommand for DecorrelationStretch {
    async fn run(&self) {
        match self.cross_file {
            true => cross_file_decorrelation(&self.input_files),
            false => individual_file_decorrelation(&self.input_files),
        };
    }
}
