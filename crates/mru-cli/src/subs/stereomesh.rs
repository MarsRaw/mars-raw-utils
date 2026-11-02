use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use clap::Parser;
use mru::stereomesh::{StereoMeshOptions, find_xyz_points};

use sciimg::prelude::*;
use std::fs::File;
use std::io::Write;
use std::process;
use stump;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Generate a mesh from stereo pair", long_about = None)]
pub struct StereoMesh {
    #[arg(long, short, help = "Left image")]
    left: std::path::PathBuf,

    #[arg(long, short, help = "Right image")]
    right: std::path::PathBuf,

    #[arg(long, short, help = "Output mesh (.obj)")]
    output: std::path::PathBuf,

    #[arg(long, short = 'm', help = "Minimum range (meters)")]
    min_range: Option<f64>,

    #[arg(long, short = 'M', help = "Maximum range (meters)")]
    max_range: Option<f64>,

    #[arg(long, short = 'd', help = "Point density")]
    density: Option<usize>,

    #[arg(long, short = 'k', help = "Kernel size")]
    kernel_size: Option<usize>,

    #[arg(long, short = 'e', help = "Maximum left/right consistency tolerance")]
    max_lr_error_px: Option<i64>,

    #[arg(long, short = 'x', help = "Maximum convergent ray miss distance")]
    max_miss_distance: Option<f64>,

    #[arg(long, short = 'c', help = "Minimum point correlation coefficient")]
    min_correlation: Option<f64>,
}

impl RunnableSubcommand for StereoMesh {
    async fn run(&self) -> Result<()> {
        pb_set_print!();
        stump::print_experimental();

        let p0 = String::from(self.left.as_os_str().to_str().unwrap());
        let p1 = String::from(self.right.as_os_str().to_str().unwrap());
        let out_file_path = String::from(self.output.as_os_str().to_str().unwrap());

        if !path::file_exists(&p0) {
            error!("Error: File not found (left eye): {}", p0);
            pb_done_with_error!();
            process::exit(1);
        }

        if !path::file_exists(&p1) {
            error!("Error: File not found (right eye): {}", p1);
            pb_done_with_error!();
            process::exit(1);
        }

        if !path::parent_exists_and_writable(&out_file_path) {
            error!(
                "Error: Output file directory not found or is not writable: {}",
                out_file_path
            );
            pb_done_with_error!();
            process::exit(1);
        }

        let defaults = StereoMeshOptions::default();

        let xyz_points = match find_xyz_points(
            &p0,
            &p1,
            &Some(StereoMeshOptions {
                kernel_size: self.kernel_size.unwrap_or(defaults.kernel_size),
                point_density: self.density.unwrap_or(defaults.point_density),
                near: self.min_range.unwrap_or(defaults.near),
                far: self.max_range.unwrap_or(defaults.far),
                max_lr_error_px: self.max_lr_error_px.unwrap_or(defaults.max_lr_error_px),
                max_miss_distance: self.max_miss_distance.unwrap_or(defaults.max_miss_distance),
                min_correlation: self.min_correlation.unwrap_or(defaults.min_correlation),
            }),
        ) {
            Ok(xyz_points) => xyz_points,
            Err(why) => return Err(anyhow!("Error generating points: {:?}", why)),
        };

        let mut obj_file_ptr = File::create(out_file_path)?;
        xyz_points.iter().try_for_each(|xyz| {
            match writeln!(obj_file_ptr, "v {} {} {}", xyz.x, xyz.y, xyz.z) {
                Ok(_) => Ok(()),
                Err(why) => bail!("Failed to write point to output file: {:?}", why),
            }
        })?;

        Ok(())
    }
}
