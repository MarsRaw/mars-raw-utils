use std::env;

use crate::{
    path,
    error,
    constants
};

fn determine_data_dir() -> String {
    if cfg!(debug_assertions) {
        return String::from("mars-raw-utils-data/caldata");
    } else {

        if cfg!(target_os = "macos") {
            return String::from("/usr/local/share/mars_raw_utils/data/");
        } else if cfg!(target_os = "windows") {
            return String::from("mars-raw-utils-data/caldata"); // C:/something/something/something/darkside/
        } else {
            return String::from("/usr/share/mars_raw_utils/data/");
        }
        
    }
}

pub fn calibration_file(calib_file_name:&str) -> error::Result<String> {

    let mut file_path = calib_file_name.to_owned();

    match env::var("MARS_RAW_DATA") {
        Ok(d) => {
            file_path = file_path.replace("{DATADIR}", d.as_str());
        },
        Err(_) => {
            let d = determine_data_dir();
            file_path = file_path.replace("{DATADIR}", d.as_str());
        }
    };

    match path::file_exists(&file_path) {
        true => Ok(file_path),
        false => Err(constants::status::FILE_NOT_FOUND)
    }   
}


