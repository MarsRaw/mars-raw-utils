use mars_raw_utils::{composite, prelude::*};
use sciimg::{prelude::*, quaternion::Quaternion};

use crate::subs::runnable::RunnableSubcommand;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Create composite mosaic", long_about = None)]
pub struct Composite {
    #[clap(
        long,
        short,
        parse(from_os_str),
        help = "Input images",
        multiple_values(true)
    )]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, parse(from_os_str), help = "Output image")]
    output: std::path::PathBuf,

    #[clap(long, short, help = "Anaglyph mode")]
    anaglyph: bool,

    #[clap(long, short = 'r', help = "Azimuth rotation")]
    azimuth: Option<f64>,
}

impl RunnableSubcommand for Composite {
    fn run(&self) {
        print::print_experimental();

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        let output = self.output.as_os_str().to_str().unwrap();

        let azimuth_rotation = self.azimuth.unwrap_or(0.0);
        let quat = Quaternion::from_pitch_roll_yaw(0.0, 0.0, azimuth_rotation.to_radians());

        let map_context = composite::determine_map_context(&in_files, &quat);
        vprintln!("Map Context: {:?}", map_context);
        vprintln!(
            "FOV Vertical: {}",
            map_context.top_lat - map_context.bottom_lat
        );
        vprintln!(
            "FOV Horizontal: {}",
            map_context.right_lon - map_context.left_lon
        );

        if map_context.width == 0 {
            eprintln!("Output expected to have zero width. Cannot continue with that. Exiting...");
            process::exit(1);
        } else if map_context.height == 0 {
            eprintln!("Output expected to have zero height. Cannot continue with that. Exiting...");
            process::exit(1);
        }

        let mut map = RgbImage::create_masked(map_context.width, map_context.height, true);

        let first_image = MarsImage::open(in_files[0].to_owned(), Instrument::M20MastcamZLeft);
        let initial_origin = if let Some(model) = composite::get_cahvor(&first_image) {
            model.c()
        } else {
            eprintln!("Cannot determine initial camera origin");
            process::exit(2);
        };

        for in_file in in_files.iter() {
            if path::file_exists(in_file) {
                vprintln!("Processing File: {}", in_file);
                composite::process_file(
                    in_file,
                    &map_context,
                    &mut map,
                    self.anaglyph,
                    &quat,
                    &initial_origin,
                );
            } else {
                eprintln!("File not found: {}", in_file);
                process::exit(1);
            }
        }

        map.normalize_to_16bit_with_max(255.0);
        map.save(output);
    }
}
