use mars_raw_utils::calprofile::load_calibration_profile;
use mars_raw_utils::prelude::*;
use sciimg::debayer::DebayerMethod;
use sciimg::path;

use crate::subs::runnable::RunnableSubcommand;

use backtrace::Backtrace;
use rayon::prelude::*;
use std::panic;
use std::process;
use std::str::FromStr;

use anyhow::{anyhow, Error, Result};
use clap::Parser;
use stump;

pb_create!();

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Calibrate {
    #[arg(long, short, help = "Input raw images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short = 'I', help = "Force instrument")]
    instrument: Option<String>,

    #[arg(long, short = 'R', help = "Red weight")]
    red_weight: Option<f32>,

    #[arg(long, short = 'G', help = "Green weight")]
    green_weight: Option<f32>,

    #[arg(long, short = 'B', help = "Blue weight")]
    blue_weight: Option<f32>,

    #[arg(long, short, help = "Raw color, skip ILT")]
    raw: bool,

    #[arg(long, short, help = "Color noise reduction amount")]
    color_noise_reduction_amount: Option<i32>,

    #[arg(long, short = 't', help = "HPC threshold")]
    hpc_threshold: Option<f32>,

    #[arg(long, short = 'w', help = "HPC window size")]
    hpc_window: Option<i32>,

    #[arg(long, short = 'd', help = "Decorrelate color channels")]
    decorrelate: bool,

    #[arg(long, short = 'P', help = "Calibration profile", num_args = 1..)]
    profile: Option<Vec<String>>,

    #[arg(long, short = 'D', help = "Debayer method (malvar, amaze)")]
    debayer: Option<String>,

    #[arg(long, short = 'C', help = "Apply sRGB color correction")]
    srgb_color_correction: bool,

    #[arg(
        long,
        short = 'S',
        help = "Skip auto subframing (cropping) of output images"
    )]
    no_subframing: bool,
}

impl Calibrate {
    fn get_calibrator_for_file(
        input_file: &str,
        default_instrument: &Option<String>,
    ) -> Option<&'static CalContainer> {
        let metadata_file = util::replace_image_extension(input_file, "-metadata.json");
        info!("Checking for metadata file at {}", metadata_file);
        if path::file_exists(metadata_file.as_str()) {
            vprintln!("Metadata file exists for loaded image: {}", metadata_file);
            match metadata::load_image_metadata(&metadata_file) {
                Err(_) => {
                    warn!("Could not load metadata file!");
                    None
                } // Error loading the metadata file
                Ok(md) => calibrator_for_instrument_from_str(&md.instrument),
            }
        } else {
            // metadata file is missing

            // If a default instrument was passed in, try and use that
            if let Some(instrument) = default_instrument {
                calibrator_for_instrument_from_str(instrument)
            } else {
                warn!("We don't know what instrument was used!");
                None // Otherwise, we don't know the instrument.
            }
        }
    }
}

use async_trait::async_trait;
#[async_trait]
impl RunnableSubcommand for Calibrate {
    async fn run(&self) -> Result<()> {
        let profiles: Vec<CalProfile> = match &self.profile {
            Some(profile_list) => {
                let mut v: Vec<CalProfile> = Vec::new();
                let profile_results: Vec<Result<CalProfile, Error>> = profile_list
                    .iter()
                    .map(|profile_name| {
                        match load_calibration_profile(profile_name) {
                            Ok(profile) => {
                                let mut profile_mut = profile;

                                // Overrides
                                if self.raw {
                                    profile_mut.apply_ilt = true;
                                }

                                if let Some(red_scalar) = self.red_weight {
                                    profile_mut.red_scalar = red_scalar;
                                }

                                if let Some(green_scalar) = self.green_weight {
                                    profile_mut.green_scalar = green_scalar;
                                }

                                if let Some(color_noise_reduction_amount) =
                                    self.color_noise_reduction_amount
                                {
                                    profile_mut.color_noise_reduction = true;
                                    profile_mut.color_noise_reduction_amount =
                                        color_noise_reduction_amount;
                                }

                                if let Some(hpc_threshold) = self.hpc_threshold {
                                    profile_mut.hot_pixel_detection_threshold = hpc_threshold;
                                }

                                if let Some(hpc_window) = self.hpc_window {
                                    profile_mut.hot_pixel_window_size = hpc_window;
                                }

                                if self.decorrelate {
                                    profile_mut.decorrelate_color = true;
                                }

                                if let Some(debayer) = &self.debayer {
                                    profile_mut.debayer_method =
                                        match DebayerMethod::from_str(debayer) {
                                            Ok(m) => m,
                                            Err(why) => panic!("Error: {}", why),
                                        };
                                }
                                Ok(profile_mut)
                            }
                            Err(why) => Err(anyhow!("Error loading calibration profile: {}", why)),
                        }

                        //v.push(
                    })
                    .collect();
                for f in profile_results {
                    match f {
                        Ok(cp) => v.push(cp.clone()),
                        Err(why) => return Err(anyhow!(why)),
                    };
                }
                v
            }
            None => vec![CalProfile {
                calfiletype: "profile".to_string(),
                apply_ilt: !self.raw,
                red_scalar: self.red_weight.unwrap_or(1.0),
                green_scalar: self.green_weight.unwrap_or(1.0),
                blue_scalar: self.blue_weight.unwrap_or(1.0),
                color_noise_reduction: self.color_noise_reduction_amount.is_some(),
                color_noise_reduction_amount: self.color_noise_reduction_amount.unwrap_or(0),
                hot_pixel_detection_threshold: self.hpc_threshold.unwrap_or(0.0),
                hot_pixel_window_size: self.hpc_window.unwrap_or(3),
                filename_suffix: String::from(constants::OUTPUT_FILENAME_APPEND),
                decorrelate_color: self.decorrelate,
                mission: None,
                instrument: None,
                description: None,
                debayer_method: if let Some(d) = &self.debayer {
                    match DebayerMethod::from_str(d) {
                        Ok(m) => m,
                        Err(why) => panic!("Error: {}", why),
                    }
                } else {
                    DebayerMethod::Malvar
                },
                srgb_color_correction: self.srgb_color_correction,
                auto_subframing: !self.no_subframing,
            }],
        };

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        pb_set_print_and_length!(in_files.len() * profiles.len());

        panic::set_hook(Box::new(|_info| {
            if stump::is_verbose() {
                println!("{:?}", Backtrace::new());
            }
            print_fail("Internal Error!");

            // If the user has exported MRU_EXIT_ON_PANIC=1, then we should exit here.
            // This will prevent situations where errors fly by on the screen and
            // aren't noticed when testing.
            if let Some(v) = option_env!("MRU_EXIT_ON_PANIC") {
                if v == "1" {
                    process::exit(1);
                }
            };
        }));

        in_files.par_iter().for_each(|input_file| {
            if !path::file_exists(input_file) {
                print_fail(&format!("Error: File not found: {}", input_file));
                process::exit(1);
            }

            if let Some(cal) = Calibrate::get_calibrator_for_file(input_file, &self.instrument) {
                profiles.par_iter().for_each(|p| {
                    match cal.calibrator.process_with_profile(input_file, false, p) {
                        Ok(res) => {
                            pb_println!(format_complete(
                                &format!(
                                    "{} ({})",
                                    path::basename(input_file),
                                    res.cal_context.filename_suffix
                                ),
                                res.status,
                            ));
                        }
                        Err(res) => {
                            pb_println!(format!("Error: {:?}", res));
                            pb_println!(format_fail(input_file));
                        }
                    };
                    pb_inc!();
                });
            } else {
                print_fail(&format!(
                    "{} - Error: Instrument Unknown!",
                    path::basename(input_file)
                ));
            }
        });

        Ok(())
    }
}
