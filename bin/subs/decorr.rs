use crate::subs::runnable::RunnableSubcommand;
use mars_raw_utils::prelude::*;
use rayon::prelude::*;
use sciimg::lowpass;
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

    #[clap(long, short = 'b', help = "Ignore black values")]
    ignore_black: bool,
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

trait MinMaxIgnoreBlack {
    fn get_min_max_ignore_black(&self) -> MinMax;
}

impl MinMaxIgnoreBlack for ImageBuffer {
    fn get_min_max_ignore_black(&self) -> MinMax {
        let mut mm = MinMax {
            min: std::f32::MAX,
            max: std::f32::MIN,
        };
        (0..self.buffer.len()).into_iter().for_each(|i| {
            if self.buffer[i] != std::f32::INFINITY && self.buffer[i] > 0.0 {
                mm.min = min!(mm.min, self.buffer[i]);
                mm.max = max!(mm.max, self.buffer[i]);
            }
        });
        mm
    }
}

fn color_range_determine_prep(image: &RgbImage) -> RgbImage {
    let mut cloned = image.clone();

    // Here we need to correct for energetic particle hits, hot pixels, and outlier values.
    // To accomplish this, we perform a small-radius hot pixel correction and then a
    // low-pass blur. This is only for range determination, and not for the output image.
    // Testing will indicate whether this is more or less than we actually need to do to
    // accomplish this goal.
    cloned.hot_pixel_correction(4, 2.0);
    cloned = lowpass::lowpass(&cloned, 5);

    cloned
}

fn cross_file_decorrelation(input_files: &Vec<PathBuf>, ignore_black: bool) {
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

            let prepped = color_range_determine_prep(&image);

            for b in 0..3 {
                let mm = match ignore_black {
                    true => prepped.get_band(b).get_min_max_ignore_black(),
                    false => prepped.get_band(b).get_min_max(),
                };
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

fn individual_file_decorrelation(input_files: &Vec<PathBuf>, ignore_black: bool) {
    input_files.par_iter().for_each(|in_file| {
        if in_file.exists() {
            vprintln!("Processing File: {:?}", in_file);

            let mut image =
                RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

            let prepped = color_range_determine_prep(&image);
            for b in 0..3 {
                let mm = match ignore_black {
                    true => prepped.get_band(b).get_min_max_ignore_black(),
                    false => prepped.get_band(b).get_min_max(),
                };
                image.normalize_band_to_with_min_max(b, 0.0, 65535.0, mm.min, mm.max);
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

#[async_trait::async_trait]
impl RunnableSubcommand for DecorrelationStretch {
    async fn run(&self) {
        match self.cross_file {
            true => cross_file_decorrelation(&self.input_files, self.ignore_black),
            false => individual_file_decorrelation(&self.input_files, self.ignore_black),
        };
    }
}
