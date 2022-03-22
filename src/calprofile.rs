
use crate::{
    error,
    calibfile,
    vprintln,
    constants
};

use serde::{
    Deserialize, 
    Serialize
};

use std::fs::File;
use std::io::Read;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CalProfile {

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

    #[serde(default = "default_filename_suffix")]
    pub filename_suffix: String
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


pub fn load_calibration_profile(file_path:&String) -> error::Result<CalProfile> {
    match calibfile::locate_calibration_file_no_extention(file_path, &".toml".to_string()) {
        Ok(located_file) => {
            let mut file = match File::open(&located_file) {
                Err(why) => panic!("couldn't open {}", why),
                Ok(file) => file,
            };
        
            let mut buf : Vec<u8> = Vec::default();
            file.read_to_end(&mut buf).unwrap();
            let text = String::from_utf8(buf).unwrap();
        
            match toml::from_str(&text) {
                Ok(calprof) => {
                    vprintln!("Loaded calibration profile from {}", located_file);
                    vprintln!("Profile: {:?}", calprof);
                    Ok(calprof)
                },
                Err(why) => {
                    eprintln!("Error parsing calibration profile: {:?}", why);
                    Err("Error parsing calibration profile file")
                }
            }
        },
        Err(why) => {
            Err(why)
        }
    }
}