use mars_raw_utils::prelude::*;

use crate::subs::runnable::RunnableSubcommand;

use backtrace::Backtrace;
use rayon::prelude::*;
use std::panic;
use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Batch raw image calibration", long_about = None)]
pub struct Calibrate {
    #[clap(
        long,
        short,
        parse(from_os_str),
        help = "Input raw images",
        multiple_values(true)
    )]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short = 'I', help = "Force instrument")]
    instrument: Option<String>,

    #[clap(long, short = 'R', help = "Red weight")]
    red_weight: Option<f32>,

    #[clap(long, short = 'G', help = "Green weight")]
    green_weight: Option<f32>,

    #[clap(long, short = 'B', help = "Blue weight")]
    blue_weight: Option<f32>,

    #[clap(long, short, help = "Raw color, skip ILT")]
    raw: bool,

    #[clap(long, short, help = "Color noise reduction amount")]
    color_noise_reduction_amount: Option<i32>,

    #[clap(long, short = 't', help = "HPC threshold")]
    hpc_threshold: Option<f32>,

    #[clap(long, short = 'w', help = "HPC window size")]
    hpc_window: Option<i32>,

    #[clap(long, short = 'd', help = "Decorrelate color channels")]
    decorrelate: bool,

    #[clap(long, short = 'P', help = "Calibration profile", multiple_values(true))]
    profile: Option<Vec<String>>,
}

impl Calibrate {
    fn get_calibrator_for_file(
        input_file: &str,
        default_instrument: &Option<String>,
    ) -> Option<&'static CalContainer> {
        let metadata_file = util::replace_image_extension(input_file, "-metadata.json");
        vprintln!("Checking for metadata file at {}", metadata_file);
        if path::file_exists(metadata_file.as_str()) {
            vprintln!("Metadata file exists for loaded image: {}", metadata_file);
            match metadata::load_image_metadata(&metadata_file) {
                Err(_) => {
                    vprintln!("Could not load metadata file!");
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
                vprintln!("We don't know what instrument was used!");
                None // Otherwise, we don't know the instrument.
            }
        }
    }
}
use async_trait::async_trait;
#[async_trait]
impl RunnableSubcommand for Calibrate {
    async fn run(&self) {
        let cal_context = CalProfile {
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
        };

        let profiles: Vec<String> = match &self.profile {
            Some(p) => p.clone(),
            None => vec![],
        };

        panic::set_hook(Box::new(|_info| {
            if print::is_verbose() {
                println!("{:?}", Backtrace::new());
            }
            print_fail(&"Internal Error!".to_string());

            // If the user has exported MRU_EXIT_ON_PANIC=1, then we should exit here.
            // This will prevent situations where errors fly by on the screen and
            // aren't noticed when testing.
            if let Some(v) = option_env!("MRU_EXIT_ON_PANIC") {
                if v == "1" {
                    process::exit(1);
                }
            };
        }));

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        in_files.par_iter().for_each(|input_file| {
            if !path::file_exists(input_file) {
                print_fail(&format!("Error: File not found: {}", input_file));
                process::exit(1);
            }
            let calibrator = Calibrate::get_calibrator_for_file(input_file, &self.instrument);
            match calibrator {
                Some(cal) => {
                    if !profiles.is_empty() {
                        process_with_profiles(cal, input_file, false, &profiles, |result| {
                            match result {
                                Ok(cc) => print_complete(
                                    &format!(
                                        "{} ({})",
                                        path::basename(input_file),
                                        cc.cal_context.filename_suffix
                                    ),
                                    cc.status,
                                ),
                                Err(why) => {
                                    eprintln!("Error: {}", why);
                                    print_fail(input_file);
                                }
                            }
                        });
                    } else {
                        match cal.calibrator.process_file(input_file, &cal_context, false) {
                            Ok(cc) => print_complete(
                                &format!(
                                    "{} ({})",
                                    path::basename(input_file),
                                    cc.cal_context.filename_suffix
                                ),
                                cc.status,
                            ),
                            Err(why) => {
                                eprintln!("Error: {}", why);
                                print_fail(input_file);
                            }
                        }
                    }
                }
                None => {
                    print_fail(&format!(
                        "{} - Error: Instrument Unknown!",
                        path::basename(input_file)
                    ));
                }
            }
        });
    }
}
