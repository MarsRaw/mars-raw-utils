use crate::{calibfile, constants, veprintln, vprintln};

use sciimg::error;

use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CalProfile {
    pub calfiletype: String,

    #[serde(default = "default_false")]
    pub apply_ilt: bool,

    #[serde(default = "default_color_scalar")]
    pub red_scalar: f32,

    #[serde(default = "default_color_scalar")]
    pub green_scalar: f32,

    #[serde(default = "default_color_scalar")]
    pub blue_scalar: f32,

    #[serde(default = "default_false")]
    pub color_noise_reduction: bool,

    #[serde(default = "default_color_noise_reduction_amount")]
    pub color_noise_reduction_amount: i32,

    #[serde(default = "default_hpc_threshold")]
    pub hot_pixel_detection_threshold: f32,

    #[serde(default = "default_hpc_window_size")]
    pub hot_pixel_window_size: i32,

    #[serde(default = "default_filename_suffix")]
    pub filename_suffix: String,

    pub mission: Option<String>,

    pub instrument: Option<String>,

    pub description: Option<String>,
}

impl CalProfile {
    pub fn default() -> CalProfile {
        CalProfile {
            calfiletype: "profile".to_string(),
            apply_ilt: default_false(),
            red_scalar: default_color_scalar(),
            green_scalar: default_color_scalar(),
            blue_scalar: default_color_scalar(),
            color_noise_reduction: default_false(),
            color_noise_reduction_amount: default_color_noise_reduction_amount(),
            hot_pixel_detection_threshold: default_hpc_threshold(),
            hot_pixel_window_size: default_hpc_window_size(),
            filename_suffix: default_filename_suffix(),
            mission: None,
            instrument: None,
            description: None,
        }
    }
}

fn default_hpc_window_size() -> i32 {
    10
}

fn default_filename_suffix() -> String {
    String::from(constants::OUTPUT_FILENAME_APPEND)
}

fn default_false() -> bool {
    false
}

fn default_color_scalar() -> f32 {
    1.0
}

fn default_color_noise_reduction_amount() -> i32 {
    0
}

fn default_hpc_threshold() -> f32 {
    0.0
}

pub fn load_calibration_profile(file_path: &String) -> error::Result<CalProfile> {
    match calibfile::locate_calibration_file_no_extention(file_path, &".toml".to_string()) {
        Ok(located_file) => {
            let mut file = match File::open(&located_file) {
                Err(why) => panic!("couldn't open {}", why),
                Ok(file) => file,
            };

            let mut buf: Vec<u8> = Vec::default();
            file.read_to_end(&mut buf).unwrap();
            let text = String::from_utf8(buf).unwrap();

            match toml::from_str(&text) {
                Ok(calprof) => {
                    vprintln!("Loaded calibration profile from {}", located_file);
                    vprintln!("Profile: {:?}", calprof);
                    Ok(calprof)
                }
                Err(why) => {
                    veprintln!("Error parsing calibration profile: {:?}", why);
                    Err("Error parsing calibration profile file")
                }
            }
        }
        Err(why) => Err(why),
    }
}
