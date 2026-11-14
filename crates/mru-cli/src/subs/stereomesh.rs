use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use mru::stereomesh::Vertex;
use mru::stereomesh::{StereoMeshOptions, find_xyz_points};

use sciimg::prelude::*;
use sciimg::vector::Vector;
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

impl RunnableSubcommand for StereoMesh {
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
            }),
        ) {
            Ok(xyz_raster) => xyz_raster,
            Err(why) => return Err(anyhow!("Error generating points: {:?}", why)),
        };

        let mtl_path = self.output.with_extension("mtl");
        let mtl_name = mtl_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid output path for material file"))?
            .to_string();

        let obj_file_ptr = File::create(out_file_path)?;

        // Write obj file here...
        let mut out = std::io::BufWriter::new(obj_file_ptr);

        writeln!(out, "# Wavefront OBJ generated by Mars Raw Utils")?;
        writeln!(out, "mtllib ./{}", mtl_name)?;

        let valids = xyz_raster.as_flat_array_only_valids();

        // Vertices (order matches Vertex::index)
        for vtx in &valids {
            if let Some(p) = &vtx.xyz {
                writeln!(out, "v {} {} {}", p.x, p.y, p.z)?;
            }
        }

        // Texture coordinates, one per vertex in the same order.
        // OBJ's V axis points up, image rows point down, so flip V.
        let (iw, ih) = (
            xyz_raster.image_width as f64,
            xyz_raster.image_height as f64,
        );
        for vtx in &valids {
            let u = vtx.raster_x as f64 / iw;
            let v = 1.0 - (vtx.raster_y as f64 / ih);
            writeln!(out, "vt {:.6} {:.6}", u, v)?;
        }

        // Faces use the same index for position and texcoord: f v/vt
        let obj_idx = |v: &Vertex| -> Option<usize> { v.has_xyz().then(|| v.index + 1) };

        let mut tri = |a: usize, b: usize, c: usize| -> std::io::Result<()> {
            writeln!(out, "f {a}/{a} {b}/{b} {c}/{c}")
        };

        let max_edge_length = self.max_edge_length.unwrap_or(0.06);
        let max_edge_ratio = self.max_edge_ratio.unwrap_or(0.03);

        for r in 0..xyz_raster.rows.saturating_sub(1) {
            for c in 0..xyz_raster.cols.saturating_sub(1) {
                let tl_v = xyz_raster.get(r, c)?;
                let tr_v = xyz_raster.get(r, c + 1)?;
                let bl_v = xyz_raster.get(r + 1, c)?;
                let br_v = xyz_raster.get(r + 1, c + 1)?;

                let tl = obj_idx(tl_v);
                let tr = obj_idx(tr_v);
                let bl = obj_idx(bl_v);
                let br = obj_idx(br_v);

                match (tl, tr, bl, br) {
                    (Some(tl), Some(tr), Some(bl), Some(br)) => {
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
                            tri(tl, bl, tr)?;
                        }

                        if keep_face(
                            &tr_xyz,
                            &bl_xyz,
                            &br_xyz,
                            &xyz_raster.origin,
                            max_edge_ratio,
                            max_edge_length,
                        ) {
                            tri(tr, bl, br)?;
                        }
                    }
                    (Some(tl), Some(tr), Some(bl), None) => {
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
                            tri(tl, bl, tr)?
                        }
                    }
                    (Some(tl), Some(tr), None, Some(br)) => {
                        let tl_xyz = tl_v.xyz.unwrap();
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
                            tri(tl, br, tr)?
                        }
                    }
                    (Some(tl), None, Some(bl), Some(br)) => {
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
                            tri(tl, bl, br)?
                        }
                    }
                    (None, Some(tr), Some(bl), Some(br)) => {
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
                            tri(tr, bl, br)?
                        }
                    }
                    _ => {}
                }
            }
        }

        out.flush()?;

        // Texture is the left image. Prefer an absolute path so the MTL works
        // wherever the OBJ is opened from.
        let texture_path = std::fs::canonicalize(&self.left).unwrap_or_else(|_| self.left.clone());

        let mut mtl = File::create(&mtl_path)
            .map_err(|why| anyhow!("Failed to create material file: {:?}", why))?;
        writeln!(mtl, "# Material generated by Mars Raw Utils")?;
        writeln!(mtl, "newmtl stereomesh")?;
        writeln!(mtl, "illum 0")?; // constant color, no lighting or shading
        writeln!(mtl, "Ka 1.0 1.0 1.0")?;
        writeln!(mtl, "Kd 1.0 1.0 1.0")?;
        writeln!(mtl, "Ks 0.0 0.0 0.0")?;
        writeln!(
            mtl,
            "map_Ka ./{}",
            texture_path.file_name().unwrap().display()
        )?;
        writeln!(
            mtl,
            "map_Kd ./{}",
            texture_path.file_name().unwrap().display()
        )?;

        pb_done!();
        Ok(())
    }
}
