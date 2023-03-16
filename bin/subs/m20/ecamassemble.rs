use mars_raw_utils::prelude::*;

use crate::subs::runnable::RunnableSubcommand;
use mars_raw_utils::m20::assemble::{Composite, NavcamTile};
use mars_raw_utils::m20::ncamlevels;
use mars_raw_utils::util;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Reassemble M20 ECAM subframes", long_about = None)]
pub struct M20EcamAssemble {
    #[clap(
        long,
        short,
        parse(from_os_str),
        help = "Input raw images",
        multiple_values(true)
    )]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, parse(from_os_str), help = "Output image")]
    output: std::path::PathBuf,
}

#[async_trait::async_trait]
impl RunnableSubcommand for M20EcamAssemble {
    async fn run(&self) {
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
                process::exit(1);
            }
            let image =
                NavcamTile::new_from_file(&String::from(in_file), Instrument::M20NavcamRight);
            //image.image.destretch_image();

            tiles.push(image);
        }

        ncamlevels::match_levels(&mut tiles);

        vprintln!("Creating composite structure");
        let mut composite = Composite::new(&tiles);

        vprintln!("Adding {} tiles to composite", tiles.len());
        composite.paste_tiles(&tiles);

        // if tiles[0].get_scale_factor() == 1 {
        //     vprintln!("Cropping telemetry pixels");
        //     composite.crop(0, 0, 5120, 3840);
        // }

        vprintln!("Saving composite to {}", output);
        composite.finalize_and_save(output);

        if let Some(mut md) = tiles[0].image.metadata.clone() {
            md.subframe_rect = Some(vec![1.0, 1.0, 5120.0, 3840.0]);
            util::save_image_json(output, &md, false, None).unwrap();
        }
    }
}
