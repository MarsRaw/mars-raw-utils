use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use mru::composite::create_composite;
use mru::metadata::{ImageProjection, Mosaic, MosaicInput};
use mru::prelude::*;
use mru::projection::cylindrical::determine_map_extent_for_files;
use sciimg::vector::Vector;
use sciimg::{prelude::*, quaternion::Quaternion};
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

    #[arg(
        long,
        short = 's',
        help = "Image size used as length & height for polar projections (default: 2048)"
    )]
    size: Option<usize>,

    #[arg(
        long,
        short = 'd',
        help = "Maximum distance (meters) for polar projections (default: 25.0)"
    )]
    distance: Option<f64>,

    #[arg(long, short = 'p', help = "Mosaic projection (default: cylindrical)")]
    projection: Option<Projection>,

    #[arg(
        long,
        short = 'e',
        help = "Eye height (meters) for polar projections (default: 1.98)"
    )]
    eye_height: Option<f64>,
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

fn project_image(
    in_file: &str,
    azimuth_rotation: f64,
    size: usize,
    max_radius: f64,
    eye_height: f64,
    projection: Projection,
) -> Result<()> {
    let out_file = util::append_file_name(in_file, "cyl");

    let image = MarsImage::open(in_file)?;

    // Try to get the azimuth from the metadata first. If that fails, try the camera model.
    // If the camera model fails, just blow up because we won't be able to project the
    // image anyways
    let look_azimuth = -if let Ok(az) = mast_azimuth_from_metadata(&image) {
        az
    } else {
        mast_azimuth_from_camera_model(&image)?
    } + azimuth_rotation.to_radians();

    debug!("Azimuth from camera model: {}", look_azimuth.to_degrees());

    let canvas = create_composite(
        &[in_file.to_string()],
        false,
        look_azimuth,
        size,
        max_radius,
        eye_height,
        projection,
    )?;

    // Unrotated projection model
    let unrotated_map_context =
        determine_map_extent_for_files(&[in_file.to_string()], &Quaternion::default())?;
    debug!("Unrotated Map Context: {:?}", unrotated_map_context);

    // Rebundle as a MarsImage with metadata
    let mut projected_mars_image = image.clone();
    projected_mars_image.image = canvas;
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
        let size = self.size.unwrap_or(2048);
        let max_radius = self.distance.unwrap_or(25.0);
        let projection = self.projection.unwrap_or(Projection::Cylindrical);
        let eye_height = self.eye_height.unwrap_or(1.98);

        in_files.iter().try_for_each(|in_file| {
            project_image(
                in_file,
                azimuth_rotation,
                size,
                max_radius,
                eye_height,
                projection,
            )
            .map_err(|why| anyhow!("Error reprojecting image: {:?}", why))
        })?;

        pb_done!();
        Ok(())
    }
}
