use mars_raw_utils::calprofile::*;

use crate::subs::runnable::RunnableSubcommand;
use glob::glob;
use std::env;
use std::path::Path;

#[derive(clap::Args)]
#[clap(author, version, about = "Calibration profile information", long_about = None)]
pub struct Profile {
    #[clap(long, short = 'p', help = "Calibration profile")]
    profile: Option<String>,

    #[clap(long, short, help = "List available profiles")]
    list: bool,
}

fn print_list_header() {
    println!("Profile:                       Mission:             Instrument:          Path:");
}

fn list_profiles_in_directory(path: &str) {
    let profile_search_pattern = format!("{}/*.toml", path);

    for file_path in glob(&profile_search_pattern)
        .expect("Failed to read glob pattern")
        .flatten()
    {
        let file_path_str = String::from(file_path.to_str().unwrap());
        if let Ok(profile) = load_calibration_profile(&file_path_str) {
            println!(
                "{:30} {:20} {:20} {}",
                Path::new(&file_path_str)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                profile.mission.unwrap_or("Not set".to_string()),
                profile.instrument.unwrap_or("Not set".to_string()),
                file_path_str
            );
        }
    }
}

use async_trait::async_trait;
#[async_trait]
impl RunnableSubcommand for Profile {
    async fn run(&self) {
        if self.list && self.profile.is_some() {
            eprintln!("Error: Two actions specified, please only select one at a time");
        } else if self.list {
            print_list_header();

            //list_profiles_in_directory("mars-raw-utils-data/caldata");
            list_profiles_in_directory("/usr/share/mars_raw_utils/data");

            if let Some(v) = option_env!("MARSDATAROOT") {
                list_profiles_in_directory(v);
            }

            if let Some(dir) = dirs::home_dir() {
                let homedatadir = format!("{}/.marsdata", dir.to_str().unwrap());
                list_profiles_in_directory(homedatadir.as_str());
            }

            if let Ok(dir) = env::var("MARS_RAW_DATA") {
                list_profiles_in_directory(dir.as_str());
            }
        } else if let Some(profile) = self.profile.clone() {
            match load_calibration_profile(&profile) {
                Ok(profile) => {
                    println!(
                        "Mission: {}",
                        profile.mission.unwrap_or("Not set".to_string())
                    );

                    println!(
                        "Instrument: {}",
                        profile.instrument.unwrap_or("Not set".to_string())
                    );
                    println!(
                        "Description: {}",
                        profile.description.unwrap_or("Not set".to_string())
                    );

                    println!("Apply Decompanding: {}", profile.apply_ilt);
                    println!("Red Scalar: {}", profile.red_scalar);
                    println!("Green Scalar: {}", profile.green_scalar);
                    println!("Blue Scalar: {}", profile.blue_scalar);
                    println!(
                        "Apply Color Noise Reduction: {}",
                        profile.color_noise_reduction
                    );
                    if profile.color_noise_reduction {
                        println!(
                            "Color Noise Reduction Amount: {}",
                            profile.color_noise_reduction_amount
                        );
                    }
                    println!(
                        "Apply Hot Pixel Correction: {}",
                        profile.hot_pixel_detection_threshold > 0.0
                    );
                    if profile.hot_pixel_detection_threshold > 0.0 {
                        println!("HPC Threshold: {}", profile.hot_pixel_detection_threshold);
                        println!("HPC Window Size: {}", profile.hot_pixel_window_size);
                    }
                    println!("Output Filename Suffix: {}", profile.filename_suffix);
                }
                Err(why) => {
                    eprintln!("Error: {}", why);
                }
            };
        } else {
            eprintln!("Error: No action specified");
        }
    }
}
