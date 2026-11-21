use mru::metadata::MissionMetadata;

use crate::subs::runnable::RunnableSubcommand;
use anyhow::{Result, anyhow};
use clap::Parser;
use colored::{self, Colorize};
use mru::m20::assemble::{Composite, NavcamTile};
use mru::m20::ncamlevels;
use mru::util;
use sciimg::path;
use std::env;
use std::process;
use stump::warn;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Reassemble M20 ECAM subframes", long_about = None)]
pub struct M20EcamAssemble {
    #[arg(long, short, help = "Input raw images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,

    #[arg(long, short, help = "Do not perform brightness matching")]
    nobright: bool,
}

impl RunnableSubcommand for M20EcamAssemble {
    async fn run(&self) -> Result<()> {
        pb_set_print!();

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();
        let output = self.output.as_os_str().to_str().unwrap();

        let mut tiles: Vec<NavcamTile> = vec![];
        for in_file in in_files.iter() {
            if !path::file_exists(in_file) {
                eprintln!("File not found: {}", in_file);
                pb_done_with_error!();
                process::exit(1);
            }
            let image = NavcamTile::new_from_file(&String::from(in_file))?;

            // Disabling destretch for now as the tool will just restretch it anyway
            // when the image is reencoded for 16 bit.
            //image.image.destretch_image();

            // Oftentimes an image will be subframed at scale factor 1 with a full frame at scale factor 4. We cannot
            // (or rather, choose not to) mix image sizes, so we will discard that full frame image.
            if !image.is_supported_scale_factor() {
                println!(
                    "{}: Discarding image at unsupported scale factor of {}: {}",
                    "WARNING".yellow(),
                    image.get_scale_factor(),
                    in_file
                );
            } else {
                tiles.push(image);
            }
        }

        // We can 'assemble' with just one. At a minimum, it just embeds it into
        // the full frame, even if most of that full frame is black.
        if tiles.is_empty() {
            eprintln!("{}: No images to assemble, exiting...", "ERROR".red());
            pb_done_with_error!();
            process::exit(1);
        }

        // Runs each tile through the level matching algorithms
        if !self.nobright {
            vprintln!("Computing relative brightnesses");
            ncamlevels::match_levels(&mut tiles);
        } else {
            vprintln!("Skipping the computation of relative brightnesses");
        }

        // Build a composite canvas. This will be the output image
        vprintln!("Creating composite structure");
        let mut composite = Composite::new(&tiles);

        // Pastes the tiles into the canvas
        vprintln!("Adding {} tiles to composite", tiles.len());
        let results = composite.paste_tiles(&tiles);

        // Determine the dimension limits of the tiles
        let mut tl_x = usize::MAX;
        let mut tl_y = usize::MAX;
        let mut br_x = usize::MIN;
        let mut br_y = usize::MIN;

        results
            .iter()
            .filter_map(|r| match r {
                Err(why) => {
                    warn!("Failed to paste subframe: {:?}", why);
                    None
                }
                Ok(tc) => Some(tc),
            })
            .for_each(|tc| {
                tl_x = tl_x.min(tc.top_left_x);
                tl_y = tl_y.min(tc.top_left_y);
                br_x = br_x.max(tc.bottom_right_x);
                br_y = br_y.max(tc.bottom_right_y);
            });

        vprintln!(
            "Cropping to subframe {}, {} - {}, {}",
            tl_x,
            tl_y,
            br_x - tl_x,
            br_y - tl_y
        );
        composite.crop(tl_x, tl_y, br_x - tl_x, br_y - tl_y);

        // Stretches image to 16 bit and saves to disk
        vprintln!("Saving composite to {}", output);
        composite.finalize_and_save(output);

        let mut metadata = tiles[0].image.metadata.clone();

        // Add the current process to the image history
        metadata
            .processing
            .history
            .push(env::args().collect::<Vec<String>>().join(" "));

        // Update the subframe region to reflect the cropped composite
        match &mut metadata.mission {
            MissionMetadata::Surface(surface_metadata) => {
                surface_metadata.subframe_rect = Some(vec![
                    1.0 + tl_x as f64,
                    1.0 + tl_y as f64,
                    composite.width as f64 * surface_metadata.scale_factor as f64,
                    composite.height as f64 * surface_metadata.scale_factor as f64,
                ]);
            }
            MissionMetadata::Flight(_) => return Err(anyhow!("Invalid metadata for image")),
        }

        util::save_image_json(output, &metadata, None).unwrap();

        pb_done!();
        Ok(())
    }
}
