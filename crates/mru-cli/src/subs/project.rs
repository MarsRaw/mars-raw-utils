use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use mru::metadata::{ImageProjection, Mosaic, MosaicInput};
use mru::{
    composite::{determine_map_context, process_file},
    prelude::*,
};
use sciimg::vector::Vector;
use sciimg::{drawable::Drawable, prelude::*, quaternion::Quaternion};
use std::process;
use stump;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Create cylindrical projection of an input image", long_about = None)]
pub struct Project {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(
        long,
        short = 'r',
        help = "Azimuth rotation",
        allow_negative_numbers = true
    )]
    azimuth: Option<f64>,
}

fn mast_azimuth_from_metadata(image: &MarsImage) -> Result<f64> {
    let metadata = match image.metadata.as_surface() {
        Some(md) => md,
        None => return Err(anyhow!("Error: Insufficient or missing image metadata")),
    };

    let cv = match &metadata.camera_vector {
        Some(cv) => Vector::from_vec(cv)?,
        None => Vector::default(),
    }
    .normalized();

    Ok(cv.y.atan2(cv.x).rem_euclid(std::f64::consts::TAU))
}

fn mast_azimuth_from_camera_model(image: &MarsImage) -> Result<f64> {
    let camera_model = match image.get_cahvor() {
        Some(cm) => cm,
        None => {
            return Err(anyhow!("Error: missing camera model from metadata"));
        }
    };

    let lv = camera_model.ls_to_look_vector(&ImageCoordinate {
        line: image.image.height as f64 / 2.0,
        sample: image.image.width as f64 / 2.0,
    })?;
    let d = lv.look_direction.normalized();
    Ok(d.y.atan2(d.x).rem_euclid(std::f64::consts::TAU))
}

fn project_image(in_file: &str, azimuth_rotation: f64) -> Result<()> {
    let out_file = util::append_file_name(in_file, "cyl");

    let image = MarsImage::open(in_file)?;

    // Try to get the azimuth from the metadata first. If that fails, try the camera model.
    // If the camera model fails, just blow up because we won't be able to project the
    // image anyways
    let look_azimuth = if let Ok(az) = mast_azimuth_from_metadata(&image) {
        az
    } else {
        mast_azimuth_from_camera_model(&image)?
    };

    debug!("Azimuth from camera model: {}", look_azimuth.to_degrees());

    // Add an additional 90 degrees to center the image in frame
    let quat =
        Quaternion::from_pitch_roll_yaw(0.0, 0.0, -look_azimuth + azimuth_rotation.to_radians());

    let map_context = determine_map_context(&[in_file.to_string()], &quat)?;

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

    let initial_origin = if let Some(model) = image.get_cahvor() {
        model.c()
    } else {
        error!("Cannot determine initial camera origin");
        pb_done_with_error!();
        process::exit(2);
    };

    if path::file_exists(in_file) {
        info!("Processing File: {}", in_file);
        process_file(
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

    // Unrotated projection model
    let unrotated_map_context =
        determine_map_context(&[in_file.to_string()], &Quaternion::default())?;
    debug!("Unrotated Map Context: {:?}", unrotated_map_context);

    // Rebundle as a MarsImage with metadata
    let mut projected_mars_image = image.clone();
    projected_mars_image.image = map;
    projected_mars_image.metadata.mosaic = Some(Mosaic {
        projection: ImageProjection {
            projection: Projection::Cylindrical, // Eventually this won't be hardcoded as I add additional projections (polar, etc)
            top_lat: unrotated_map_context.top_lat,
            bottom_lat: unrotated_map_context.bottom_lat,
            left_lon: unrotated_map_context.left_lon,
            right_lon: unrotated_map_context.right_lon,
            width: unrotated_map_context.width,
            height: unrotated_map_context.height,
            degrees_per_pixel: unrotated_map_context.degrees_per_pixel,
        },
        sources: vec![MosaicInput {
            file: in_file.to_string(),
            gain: 0.0,
        }],
    });
    projected_mars_image.update_history();

    projected_mars_image.save(&out_file)
}

impl RunnableSubcommand for Project {
    async fn run(&self) -> Result<()> {
        pb_set_print!();

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
