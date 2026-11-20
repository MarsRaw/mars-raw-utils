use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use mru::composite::create_composite;
use mru::metadata::CylindricalProjectionParameters;
use mru::metadata::PolarProjectionParameters;
use mru::metadata::ProjectionParameters;
use mru::metadata::{ImageProjection, Mosaic, MosaicInput};
use mru::prelude::*;
use mru::projection::cylindrical;
use mru::projection::polar;
use mru::projection::polar::PolarView;
use mru::projection::polar::Pole;
use rayon::iter::IntoParallelRefIterator as _;
use rayon::iter::ParallelIterator;
use sciimg::camera::model::ImageCoordinate;
use sciimg::vector::Vector;

use stump;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Create cylindrical projection of an input image", long_about = None)]
pub struct Project {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short = 't', help = "Transparent background")]
    transparent: bool,

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

    #[arg(
        long,
        short = 'P',
        help = "Vertical perspective (ground, sky) for polar projections (default: ground)"
    )]
    perspective: Option<VerticalPerspective>,

    #[arg(
        long,
        short = 'p',
        help = "Mosaic projection (default: equirectangular)"
    )]
    projection: Option<Projection>,

    #[arg(
        long,
        short = 'e',
        help = "Override eye height (meters) for polar projections"
    )]
    eye_height: Option<f64>,

    #[arg(
        long,
        short = 'z',
        help = "Maximum angle from center (zenith or nadir) (default: 90/deg)"
    )]
    max_zenith_angle: Option<f64>,
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
    polar_view: PolarView,
    projection: Projection,
    transparent: bool,
) -> Result<()> {
    let file_suffix = match &projection {
        Projection::Equirectangular
        | Projection::EqualArea
        | Projection::Perspective
        | Projection::Mercator => "cyl",
        _ => "pol",
    };

    let out_file = util::append_file_name(in_file, file_suffix);

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
        polar_view,
        projection,
        false,
        transparent,
    )?;

    // Rebundle as a MarsImage with metadata
    let mut projected_mars_image = image.clone();
    projected_mars_image.image = canvas;

    let projection_parameters = match &projection {
        Projection::Equirectangular
        | Projection::EqualArea
        | Projection::Perspective
        | Projection::Mercator => {
            // Unrotated projection model

            let unrotated_map_extent = cylindrical::CylindricalMapExtent::new_from_files(
                &[in_file.to_string()],
                None,
                cylindrical::CylMap::from_projection(&projection)?,
            )?;
            debug!("Unrotated Map Context: {:?}", unrotated_map_extent);

            ProjectionParameters::Cylindrical(CylindricalProjectionParameters {
                top_lat: unrotated_map_extent.top_lat,
                bottom_lat: unrotated_map_extent.bottom_lat,
                left_lon: unrotated_map_extent.left_lon,
                right_lon: unrotated_map_extent.right_lon,
                width: unrotated_map_extent.width,
                height: unrotated_map_extent.height,
                degrees_per_pixel: unrotated_map_extent.degrees_per_pixel,
            })
        }
        // All the others are polar at the moment
        _ => {
            let radial_map = polar::RadialMap::from_projection(&projection)?;
            let unrotated_map_extent = polar::PolarMapExtent::new_from_files(
                &[in_file.to_string()],
                None,
                size,
                polar_view,
                radial_map,
            )?;
            debug!("Unrotated Map Context: {:?}", unrotated_map_extent);
            ProjectionParameters::Polar(PolarProjectionParameters {
                min_x: unrotated_map_extent.min_x,
                max_x: unrotated_map_extent.max_x,
                min_y: unrotated_map_extent.min_y,
                max_y: unrotated_map_extent.max_y,
                max_radius: unrotated_map_extent.max_radius,
                max_edge_length: unrotated_map_extent.max_edge_length,
                eye_height: unrotated_map_extent.eye_height,
                pole: match unrotated_map_extent.pole {
                    Pole::Zenith => metadata::Pole::Zenith,
                    Pole::Nadir => metadata::Pole::Nadir,
                },
                max_theta: unrotated_map_extent.max_theta,
            })
        }
    };

    projected_mars_image.metadata.mosaic = Some(Mosaic {
        projection: ImageProjection {
            projection: Projection::Equirectangular, // Eventually this won't be hardcoded as I add additional projections (polar, etc)
            parameters: projection_parameters,
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
        let projection = self.projection.unwrap_or(Projection::Equirectangular);

        let polar_view = match self.perspective.unwrap_or(VerticalPerspective::Ground) {
            VerticalPerspective::Sky => PolarView::Sky {
                max_zenith_deg: self.max_zenith_angle.unwrap_or(90.0),
            },
            VerticalPerspective::GroundAngle => PolarView::GroundAngle {
                max_zenith_deg: self.max_zenith_angle.unwrap_or(90.0),
            },
            VerticalPerspective::Ground => PolarView::Ground {
                max_radius,
                eye_height: self.eye_height,
            },
        };

        in_files.par_iter().try_for_each(|in_file| {
            project_image(
                in_file,
                azimuth_rotation,
                size,
                polar_view,
                projection,
                self.transparent,
            )
            .map_err(|why| anyhow!("Error reprojecting image: {:?}", why))
        })?;

        pb_done!();
        Ok(())
    }
}
