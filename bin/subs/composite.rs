use crate::subs::runnable::RunnableSubcommand;
use async_trait::async_trait;
use mars_raw_utils::{composite, prelude::*};
use sciimg::{drawable::*, prelude::*, quaternion::Quaternion, vector::Vector};

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

    #[clap(long, short, help = "Border margin size (pixels)")]
    border: Option<usize>,
}
#[async_trait]
impl RunnableSubcommand for Composite {
    async fn run(&self) {
        print::print_experimental();

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        let output = self.output.as_os_str().to_str().unwrap();

        let azimuth_rotation: f64 = self.azimuth.unwrap_or(0.0);

        let quat = Quaternion::from_pitch_roll_yaw(0.0, 0.0, azimuth_rotation.to_radians());

        let border = self.border.unwrap_or(30);

        // Load images into memory
        let input_files: Vec<MarsImage> = in_files
            .iter()
            .map(|fp| {
                if !path::file_exists(fp) {
                    panic!("Input file not found: {}", fp);
                }
                MarsImage::open(String::from(fp), Instrument::M20MastcamZLeft)
            })
            .collect();

        // Determine output camera model
        // NOT CORRECT
        let model1 = if let Some(cm) = composite::get_cahvor(&input_files[0]) {
            cm
        } else {
            panic!("Unable to determine output camera model");
        };

        let model2 = if let Some(cm) = composite::get_cahvor(&input_files[1]) {
            cm
        } else {
            panic!("Unable to determine output camera model");
        };

        let out_model = composite::warp_cahv_models(&model1, &model2);

        // Determine output image parameters
        // NOT CORRECT
        let map_context = composite::determine_map_context(&input_files, &quat, &out_model);
        vprintln!("Map Context: {:?}", map_context);

        if map_context.width == 0 {
            eprintln!("Output expected to have zero width. Cannot continue with that. Exiting...");
            process::exit(1);
        } else if map_context.height == 0 {
            eprintln!("Output expected to have zero height. Cannot continue with that. Exiting...");
            process::exit(1);
        }

        // Create output image
        let mut map = Image::create_masked(map_context.width, map_context.height, true);
        //let mut map = Image::create_masked(360 * 4, 180 * 4, true);

        composite::process_files(
            &input_files,
            &mut map,
            self.anaglyph,
            &quat,
            &out_model,
            border,
        );
        /*
        for in_file in in_files.iter() {
            if path::file_exists(in_file) {
                vprintln!("Processing File: {}", in_file);
                composite::process_file(
                    in_file,
                    &map_context,
                    &mut map,
                    self.anaglyph,
                    &quat,
                    &out_model,
                    border,
                );
            } else {
                eprintln!("File not found: {}", in_file);
                process::exit(1);
            }
        }
        */
        map.save(output);
    }
}
