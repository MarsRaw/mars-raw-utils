use crate::{calibfile, constants};
use anyhow::anyhow;
use anyhow::Result;
use regex::Regex;
use sciimg::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

lazy_static! {
    static ref CAL_TYPE_REGEX: Regex = Regex::new(r#"calfiletype\s+=\s+"profile""#).unwrap();
}

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

    #[serde(default = "default_decorrelate_color")]
    pub decorrelate_color: bool,

    pub mission: Option<String>,

    pub instrument: Option<String>,

    pub description: Option<String>,

    #[serde(default = "default_debayer_method")]
    pub debayer_method: DebayerMethod,

    #[serde(default = "default_false")]
    pub srgb_color_correction: bool,

    #[serde(default = "default_true")]
    pub auto_subframing: bool,
}

impl Default for CalProfile {
    fn default() -> CalProfile {
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
            decorrelate_color: default_decorrelate_color(),
            mission: None,
            instrument: None,
            description: None,
            debayer_method: default_debayer_method(),
            srgb_color_correction: default_false(),
            auto_subframing: default_true(),
        }
    }
}

fn default_debayer_method() -> DebayerMethod {
    DebayerMethod::Malvar
}

fn default_hpc_window_size() -> i32 {
    10
}

fn default_filename_suffix() -> String {
    String::from(constants::OUTPUT_FILENAME_APPEND)
}

fn default_decorrelate_color() -> bool {
    false
}

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
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

pub fn load_calibration_profile(file_path: &String) -> Result<CalProfile> {
    match calibfile::locate_calibration_file_no_extention(file_path, &".toml".to_string()) {
        Ok(located_file) => {
            let mut file = match File::open(&located_file) {
                Err(why) => panic!("couldn't open {}", why),
                Ok(file) => file,
            };

            let mut buf: Vec<u8> = Vec::default();
            file.read_to_end(&mut buf).unwrap();
            let text = String::from_utf8(buf).unwrap();

            if !CAL_TYPE_REGEX.is_match(&text) {
                return Err(anyhow!("Invalid calibration profile file"));
            }

            match toml::from_str(&text) {
                Ok(calprof) => {
                    info!("Loaded calibration profile from {}", located_file);
                    info!("Profile: {:?}", calprof);
                    Ok(calprof)
                }
                Err(why) => {
                    error!("Error parsing calibration profile file: {}", located_file);
                    error!("Reason: {:?}", why);
                    Err(anyhow!("Error parsing calibration profile file"))
                }
            }
        }
        Err(why) => Err(anyhow!(why)),
    }
}
