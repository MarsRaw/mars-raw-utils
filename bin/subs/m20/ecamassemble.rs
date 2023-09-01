use mars_raw_utils::prelude::*;

use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use colored::{self, Colorize};
use mars_raw_utils::m20::assemble::{Composite, NavcamTile};
use mars_raw_utils::m20::ncamlevels;
use mars_raw_utils::util;
use sciimg::path;
use std::env;
use std::process;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Reassemble M20 ECAM subframes", long_about = None)]
pub struct M20EcamAssemble {
    #[arg(long, short, help = "Input raw images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,
}

#[async_trait::async_trait]
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
            let image =
                NavcamTile::new_from_file(&String::from(in_file), Instrument::M20NavcamRight);

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
        ncamlevels::match_levels(&mut tiles);

        // Build a composite canvas. This will be the output image
        vprintln!("Creating composite structure");
        let mut composite = Composite::new(&tiles);

        // Pastes the tiles into the canvas
        vprintln!("Adding {} tiles to composite", tiles.len());
        composite.paste_tiles(&tiles);

        // Stretches image to 16 bit and saves to disk
        vprintln!("Saving composite to {}", output);
        composite.finalize_and_save(output);

        // Update 'scale_factor' in the metadata and save to disk
        tiles[0].image.metadata.subframe_rect = Some(vec![1.0, 1.0, 5120.0, 3840.0]);
        tiles[0]
            .image
            .metadata
            .history
            .push(env::args().collect::<Vec<String>>().join(" "));
        util::save_image_json(output, &tiles[0].image.metadata, None).unwrap();

        pb_done!();
        Ok(())
    }
}
