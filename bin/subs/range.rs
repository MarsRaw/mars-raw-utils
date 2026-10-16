use crate::subs::runnable::RunnableSubcommand;
use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;
use mars_raw_utils::range;
use mars_raw_utils::range::RangeKind;
use mars_raw_utils::range::StereoParams;
use sciimg::prelude::*;
use std::process;
use stump;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Generate range product from stereo images", long_about = None)]
pub struct Range {
    #[arg(long, short, help = "Left image")]
    left: std::path::PathBuf,

    #[arg(long, short, help = "Right image")]
    right: std::path::PathBuf,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,

    #[arg(long, short = 'm', help = "Minimum range (meters)")]
    min_range: Option<f64>,

    #[arg(long, short = 'M', help = "Maximum range (meters)")]
    max_range: Option<f64>,

    #[arg(long, short = 's', help = "Stretch to full value range")]
    stretch: bool,
}

impl RunnableSubcommand for Range {
    async fn run(&self) -> Result<()> {
        pb_set_print!();
        stump::print_experimental();

        let left_image_path = String::from(self.left.as_os_str().to_str().unwrap());
        let right_image_path = String::from(self.right.as_os_str().to_str().unwrap());
        let out_file_path = self.output.as_os_str().to_str().unwrap();

        if !path::file_exists(&left_image_path) {
            error!("Error: File not found (left eye): {}", left_image_path);
            pb_done_with_error!();
            process::exit(1);
        }

        if !path::file_exists(&right_image_path) {
            error!("Error: File not found (right eye): {}", right_image_path);
            pb_done_with_error!();
            process::exit(1);
        }

        if !path::parent_exists_and_writable(out_file_path) {
            error!(
                "Error: Output file directory not found or is not writable: {}",
                out_file_path
            );
            pb_done_with_error!();
            process::exit(1);
        }

        let left_img = MarsImage::open(&left_image_path, Instrument::M20MastcamZLeft);
        let right_img = MarsImage::open(&right_image_path, Instrument::M20MastcamZRight);

        let result = range::compute_stereo_range(
            &left_img,
            &right_img,
            &StereoParams {
                min_range_m: self.min_range.unwrap_or(0.5),
                max_range_m: self.max_range.unwrap_or(10.0),
                window_radius: 4,
                prefilter_radius: 7,
                max_cost: 0.04,
                lr_tolerance: 1,
                range_kind: RangeKind::Distance,
            },
        );

        let range_model = if let Ok(range_model) = result {
            range_model
        } else if let Err(why) = result {
            return Err(anyhow!("Error generating range: {}", why));
        } else {
            return Err(anyhow!("Undetermined error"));
        };

        let range_img_result = range_model.to_mars_image(&left_img, 1.0);
        let mut range_img = if let Ok(range_img) = range_img_result {
            range_img
        } else if let Err(why) = range_img_result {
            return Err(anyhow!("Error converting range map to image: {}", why));
        } else {
            return Err(anyhow!("Undetermined error"));
        };

        if self.stretch {
            range_img.image.normalize_between(0.0, 65535.0);
        }

        if let Err(why) = range_img.save(out_file_path) {
            return Err(anyhow!(
                "File error: Failed to write output file to {}: {:?}",
                out_file_path,
                why
            ));
        }

        pb_done!();
        Ok(())
    }
}
