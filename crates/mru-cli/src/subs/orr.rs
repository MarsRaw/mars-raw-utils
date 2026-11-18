use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use mru::enums::DownsampleScale;
use mru::marsimage::MarsImage;
use mru::stereomesh::Vertex;
use mru::stereomesh::{StereoMeshOptions, find_xyz_points};

use sciimg::drawable::Drawable;
use sciimg::drawable::Point;
use sciimg::drawable::Triangle;
use sciimg::prelude::*;
use sciimg::vector::Vector;
use std::process;
use stump;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Generate an orthogonal map from a stereo pair", long_about = None)]
pub struct OrrMap {
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

    #[arg(
        long,
        short = 'H',
        help = "Maximum deviations from the median height from the surface"
    )]
    max_height_deviations: Option<f64>,

    #[arg(
        long,
        short = 'R',
        help = "Maximum length to range ratio of any edge before a face is discarded"
    )]
    max_edge_ratio: Option<f64>,

    #[arg(
        long,
        short = 'E',
        help = "Maximum length of any edge before a face is discarded"
    )]
    max_edge_length: Option<f64>,

    #[arg(long, short = 'p', help = "Disable multithreaded processing")]
    no_multithreading: bool,

    #[arg(
        long,
        short = 's',
        help = "Image size used as length & height for the output map (default: 2048)"
    )]
    size: Option<usize>,

    #[arg(
        long,
        short = 'D',
        help = "Downsample scale. 1 or a power of 2 from 2-8 "
    )]
    downsample: Option<DownsampleScale>,
}

// So, to limit large anomolous spikes in the mesh, we will limit the length of any edge
// to some reasonable value. If an edge exceeds the limit, the face is discarded.
fn keep_face(
    a: &Vector,
    b: &Vector,
    c: &Vector,
    cam_center: &Vector,
    max_edge_ratio: f64,
    max_edge_length: f64,
) -> bool {
    let longest = a.distance_to(b).max(b.distance_to(c)).max(c.distance_to(a));

    // Range from the camera to the face centroid
    let centroid = (*a + *b + *c) / 3.0;
    let range = centroid.distance_to(cam_center);

    let limit = f64::INFINITY
        .min(max_edge_ratio * range)
        .min(max_edge_length);

    longest <= limit
}

impl RunnableSubcommand for OrrMap {
    async fn run(&self) -> Result<()> {
        pb_set_print!();
        stump::print_experimental();

        // The mesh code uses all available threads to parallelize the process.
        // This can be a bit much for some systems, especially if keeping MRU
        // to a small footprint is desirable.
        if self.no_multithreading {
            info!("User requested single thread execution.");
            info!("Setting thread pool to 1.");
            rayon::ThreadPoolBuilder::new()
                .num_threads(1)
                .build_global()
                .unwrap();
        }

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

        let max_range = self.max_range.unwrap_or(defaults.far);
        let min_range = self.min_range.unwrap_or(defaults.near);
        let downsample = self.downsample.unwrap_or(defaults.downsample);

        let left_metadata = MarsImage::load_image_metadata(&p0);

        let xyz_raster = match find_xyz_points(
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
                max_height_deviations: self
                    .max_height_deviations
                    .unwrap_or(defaults.max_height_deviations),
                downsample,
                linearize: !left_metadata.processing.linearized,
            }),
        ) {
            Ok(xyz_raster) => xyz_raster,
            Err(why) => return Err(anyhow!("Error generating points: {:?}", why)),
        };

        let map_size = self.size.unwrap_or(2048);
        let mut orr_map = Image::create_masked(map_size, map_size, true);

        let origin = Vector::default();

        let img = xyz_raster.left_eye_img.to_owned();

        let band_0 = img.image.get_band(0);
        let band_1 = img.image.get_band(1);
        let band_2 = img.image.get_band(2);

        let vector_to_xy = |vtx: &Vector| -> (f64, f64) {
            let h_len = (map_size / 2) as f64;
            let m_x = vtx.x / max_range * h_len + h_len;
            let m_y = vtx.y / max_range * h_len + h_len;
            (m_x, m_y)
        };

        let mut tri = |a: &Vertex, b: &Vertex, c: &Vertex| -> std::io::Result<()> {
            // Draw face on map
            let a_xyz = a.xyz.unwrap();
            let b_xyz = b.xyz.unwrap();
            let c_xyz = c.xyz.unwrap();

            let centroid = a_xyz.add(&b_xyz).add(&c_xyz) / 3.0;
            let face_distance = origin.distance_to(&centroid);

            if face_distance < min_range || face_distance > max_range {
                return Ok(());
            }

            let (a_x, a_y) = vector_to_xy(&a_xyz);
            let (b_x, b_y) = vector_to_xy(&b_xyz);
            let (c_x, c_y) = vector_to_xy(&c_xyz);

            let a_pt = Point::create_rgb(
                a_x,
                a_y,
                band_0.get(a.raster_x, a.raster_y) as f64,
                band_1.get(a.raster_x, a.raster_y) as f64,
                band_2.get(a.raster_x, a.raster_y) as f64,
            );

            let b_pt = Point::create_rgb(
                b_x,
                b_y,
                band_0.get(b.raster_x, b.raster_y) as f64,
                band_1.get(b.raster_x, b.raster_y) as f64,
                band_2.get(b.raster_x, b.raster_y) as f64,
            );

            let c_pt = Point::create_rgb(
                c_x,
                c_y,
                band_0.get(c.raster_x, c.raster_y) as f64,
                band_1.get(c.raster_x, c.raster_y) as f64,
                band_2.get(c.raster_x, c.raster_y) as f64,
            );
            orr_map.paint_tri(
                &Triangle {
                    p0: a_pt,
                    p1: b_pt,
                    p2: c_pt,
                },
                false,
            );
            Ok(())
        };

        let max_edge_length = self.max_edge_length.unwrap_or(0.06);
        let max_edge_ratio = self.max_edge_ratio.unwrap_or(0.03);

        for r in 0..xyz_raster.rows.saturating_sub(1) {
            for c in 0..xyz_raster.cols.saturating_sub(1) {
                let tl_v = xyz_raster.get(r, c)?;
                let tr_v = xyz_raster.get(r, c + 1)?;
                let bl_v = xyz_raster.get(r + 1, c)?;
                let br_v = xyz_raster.get(r + 1, c + 1)?;

                match (
                    tl_v.has_xyz(),
                    tr_v.has_xyz(),
                    bl_v.has_xyz(),
                    br_v.has_xyz(),
                ) {
                    (true, true, true, true) => {
                        let tl_xyz = tl_v.xyz.unwrap();
                        let tr_xyz = tr_v.xyz.unwrap();
                        let bl_xyz = bl_v.xyz.unwrap();
                        let br_xyz = br_v.xyz.unwrap();

                        if keep_face(
                            &tl_xyz,
                            &bl_xyz,
                            &tr_xyz,
                            &xyz_raster.origin,
                            max_edge_ratio,
                            max_edge_length,
                        ) {
                            tri(tl_v, bl_v, tr_v)?;
                        }

                        if keep_face(
                            &tr_xyz,
                            &bl_xyz,
                            &br_xyz,
                            &xyz_raster.origin,
                            max_edge_ratio,
                            max_edge_length,
                        ) {
                            tri(tr_v, bl_v, br_v)?;
                        }
                    }
                    (true, true, true, false) => {
                        let tl_xyz = tl_v.xyz.unwrap();
                        let tr_xyz = tr_v.xyz.unwrap();
                        let bl_xyz = bl_v.xyz.unwrap();

                        if keep_face(
                            &tl_xyz,
                            &bl_xyz,
                            &tr_xyz,
                            &xyz_raster.origin,
                            max_edge_ratio,
                            max_edge_length,
                        ) {
                            tri(tl_v, bl_v, tr_v)?
                        }
                    }
                    (true, true, false, true) => {
                        let tl_xyz: Vector = tl_v.xyz.unwrap();
                        let tr_xyz = tr_v.xyz.unwrap();
                        let br_xyz = br_v.xyz.unwrap();

                        if keep_face(
                            &tl_xyz,
                            &br_xyz,
                            &tr_xyz,
                            &xyz_raster.origin,
                            max_edge_ratio,
                            max_edge_length,
                        ) {
                            tri(tl_v, br_v, tr_v)?
                        }
                    }
                    (true, false, true, true) => {
                        let tl_xyz = tl_v.xyz.unwrap();
                        let bl_xyz = bl_v.xyz.unwrap();
                        let br_xyz = br_v.xyz.unwrap();

                        if keep_face(
                            &tl_xyz,
                            &bl_xyz,
                            &br_xyz,
                            &xyz_raster.origin,
                            max_edge_ratio,
                            max_edge_length,
                        ) {
                            tri(tl_v, bl_v, br_v)?
                        }
                    }
                    (false, true, true, true) => {
                        let tr_xyz = tr_v.xyz.unwrap();
                        let bl_xyz = bl_v.xyz.unwrap();
                        let br_xyz = br_v.xyz.unwrap();

                        if keep_face(
                            &tr_xyz,
                            &bl_xyz,
                            &br_xyz,
                            &xyz_raster.origin,
                            max_edge_ratio,
                            max_edge_length,
                        ) {
                            tri(tr_v, bl_v, br_v)?
                        }
                    }
                    _ => {}
                }
            }
        }

        orr_map.save(&out_file_path)
        // Ok(())
    }
}
