use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use mru::{composite, prelude::*};
use sciimg::{drawable::*, prelude::*, quaternion::Quaternion};
use std::process;
use stump;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Create cylindrical projection of an input image", long_about = None)]
pub struct Project {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short = 'r', help = "Azimuth rotation")]
    azimuth: Option<f64>,
}

fn project_image(in_file: &str, azimuth_rotation: f64) -> Result<()> {
    let out_file = util::append_file_name(in_file, "cyl");

    let quat = Quaternion::from_pitch_roll_yaw(0.0, 0.0, azimuth_rotation.to_radians());

    let map_context = composite::determine_map_context(&[in_file.to_string()], &quat)?;
    debug!("Map Context: {:?}", map_context);
    debug!(
        "FOV Vertical: {}",
        map_context.top_lat - map_context.bottom_lat
    );
    debug!(
        "FOV Horizontal: {}",
        map_context.right_lon - map_context.left_lon
    );

    if map_context.width == 0 {
        error!("Output expected to have zero width. Cannot continue with that. Exiting...");
        pb_done_with_error!();
        process::exit(1);
    } else if map_context.height == 0 {
        error!("Output expected to have zero height. Cannot continue with that. Exiting...");
        pb_done_with_error!();
        process::exit(1);
    }

    let mut map = Image::create_masked(map_context.width, map_context.height, true);

    let first_image = MarsImage::open(in_file)?;
    let initial_origin = if let Some(model) = composite::get_cahvor(&first_image) {
        model.c()
    } else {
        error!("Cannot determine initial camera origin");
        pb_done_with_error!();
        process::exit(2);
    };

    if path::file_exists(in_file) {
        info!("Processing File: {}", in_file);
        composite::process_file(
            in_file,
            &map_context,
            &mut map,
            false,
            &quat,
            &initial_origin,
        )?;
    } else {
        error!("File not found: {}", in_file);
        pb_done_with_error!();
        process::exit(1);
    }

    map.save(&out_file)
}

impl RunnableSubcommand for Project {
    async fn run(&self) -> Result<()> {
        pb_set_print!();

        stump::print_experimental();

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        let azimuth_rotation: f64 = self.azimuth.unwrap_or(0.0);

        in_files.iter().try_for_each(|in_file| {
            project_image(in_file, azimuth_rotation)
                .map_err(|why| anyhow!("Error reprojecting image: {:?}", why))
        })?;

        pb_done!();
        Ok(())
    }
}
