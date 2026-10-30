#![allow(clippy::needless_range_loop)]

use crate::subs::runnable::RunnableSubcommand;
use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use rayon::prelude::*;
use sciimg::lowpass;
use sciimg::prelude::*;
use sciimg::MinMax;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pb_create!();

#[derive(Parser)]
#[command(author, version, about = "Decorrelation stretching", long_about = None)]
pub struct DecorrelationStretch {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(
        long,
        short,
        help = "Cross-File decorrelation (value ranges determined across all files rather than individually)"
    )]
    cross_file: bool,

    #[arg(long, short = 'b', help = "Ignore black values")]
    ignore_black: bool,
}

#[allow(dead_code)]
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

impl NormalizeRgbImageSingleChannels for Image {
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

#[allow(dead_code)]
trait MinMaxIgnoreBlack {
    fn get_min_max_ignore_black(&self) -> MinMax;
}

impl MinMaxIgnoreBlack for ImageBuffer {
    fn get_min_max_ignore_black(&self) -> MinMax {
        let mut mm = MinMax {
            min: f32::MAX,
            max: f32::MIN,
        };
        (0..self.buffer.len()).for_each(|i| {
            if self.buffer[i] != f32::INFINITY && self.buffer[i] > 0.0 {
                mm.min = min!(mm.min, self.buffer[i]);
                mm.max = max!(mm.max, self.buffer[i]);
            }
        });
        mm
    }
}

fn color_range_determine_prep(image: &Image) -> Image {
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

fn cross_file_decorrelation(input_files: &[PathBuf], ignore_black: bool) -> Result<()> {
    let ranges_mtx = Arc::new(Mutex::new(vec![
        MinMax {
            min: f32::MAX,
            max: f32::MIN,
        },
        MinMax {
            min: f32::MAX,
            max: f32::MIN,
        },
        MinMax {
            min: f32::MAX,
            max: f32::MIN,
        },
    ]));

    info!("Computing value ranges...");
    input_files.par_iter().try_for_each(|in_file| {
        if in_file.exists() {
            let image = match MarsImage::open(&String::from(in_file.as_os_str().to_str().unwrap()))
            {
                Ok(img) => img,
                Err(why) => bail!("Failed to open image: {:?}", why),
            };

            let prepped = color_range_determine_prep(&image.image);

            (0..prepped.num_bands()).for_each(|b| {
                let mm = match ignore_black {
                    true => prepped.get_band(b).get_min_max_ignore_black(),
                    false => prepped.get_band(b).get_min_max(),
                };

                if let Ok(mut ranges) = ranges_mtx.lock() {
                    ranges[b].min = min!(mm.min, ranges[b].min);
                    ranges[b].max = max!(mm.max, ranges[b].max);
                }
            });
        } else {
            pb_done_with_error!();
            bail!("File not found: {:?}", in_file);
        }
        pb_inc!();
        Ok(())
    })?;

    pb_set_zero!();

    // Mutex no longer needed
    let ranges = ranges_mtx.lock().unwrap();

    input_files.par_iter().try_for_each(|in_file| {
        if in_file.exists() {
            info!("Processing File: {:?}", in_file);

            let mut image =
                match MarsImage::open(&String::from(in_file.as_os_str().to_str().unwrap())) {
                    Ok(img) => img,
                    Err(why) => bail!("Failed to open file: {:?}", why),
                };

            for b in 0..image.image.num_bands() {
                image.image.normalize_band_to_with_min_max(
                    b,
                    0.0,
                    65535.0,
                    ranges[b].min,
                    ranges[b].max,
                );
            }

            image.image.set_mode(ImageMode::U16BIT);

            info!("Writing to disk...");
            image.update_history();
            image
                .save(&util::append_file_name(
                    in_file.as_os_str().to_str().unwrap(),
                    "decorr",
                ))
                .expect("Failed to save image");
        } else {
            pb_done_with_error!();
            bail!("File not found: {:?}", in_file);
        }
        pb_inc!();
        Ok(())
    })?;
    Ok(())
}

fn individual_file_decorrelation(input_files: &Vec<PathBuf>, ignore_black: bool) -> Result<()> {
    input_files.par_iter().try_for_each(|in_file| {
        if in_file.exists() {
            info!("Processing File: {:?}", in_file);

            let mut image =
                match MarsImage::open(&String::from(in_file.as_os_str().to_str().unwrap())) {
                    Ok(img) => img,
                    Err(why) => bail!("Failed to open file: {:?}", why),
                };

            let prepped = color_range_determine_prep(&image.image);
            for b in 0..3 {
                let mm = match ignore_black {
                    true => prepped.get_band(b).get_min_max_ignore_black(),
                    false => prepped.get_band(b).get_min_max(),
                };
                image
                    .image
                    .normalize_band_to_with_min_max(b, 0.0, 65535.0, mm.min, mm.max);
            }

            image.image.set_mode(ImageMode::U16BIT);

            info!("Writing to disk...");
            image.update_history();
            image
                .save(&util::append_file_name(
                    in_file.as_os_str().to_str().unwrap(),
                    "decorr",
                ))
                .expect("Failed to save image");
        } else {
            // error!("File not found: {:?}", in_file);
            bail!("File not found: {:?}", in_file);
        }
        pb_inc!();
        Ok(())
    })?;
    Ok(())
}

impl RunnableSubcommand for DecorrelationStretch {
    async fn run(&self) -> Result<()> {
        pb_set_print_and_length!(self.input_files.len());
        match self.cross_file {
            true => cross_file_decorrelation(&self.input_files, self.ignore_black),
            false => individual_file_decorrelation(&self.input_files, self.ignore_black),
        }
    }
}
