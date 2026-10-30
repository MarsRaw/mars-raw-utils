use crate::subs::runnable::RunnableSubcommand;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::stereomesh::{find_xyz_points, StereoMeshOptions};

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

        let xyz_points = match find_xyz_points(&p0, &p1, &Some(StereoMeshOptions::default())) {
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
