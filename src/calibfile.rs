use std::env;

use crate::{
    path,
    error,
    constants
};



use std::fs::File;
use std::io::Read;

//use serde_derive::Deserialize;
use serde::{
    Deserialize
};

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub msl: MslCalData,
    pub m20: M20CalData,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct MslCalData {
    pub MSL_MAHLI_INPAINT_MASK_PATH : String,
    pub MSL_MAHLI_FLAT_PATH : String,
    pub MSL_NCAM_RIGHT_INPAINT_PATH : String,
    pub MSL_NCAM_RIGHT_FLAT_PATH : String,
    pub MSL_NCAM_LEFT_FLAT_PATH : String,
    pub MSL_FHAZ_RIGHT_FLAT_PATH : String,
    pub MSL_FHAZ_LEFT_FLAT_PATH : String,
    pub MSL_RHAZ_RIGHT_FLAT_PATH : String,
    pub MSL_RHAZ_LEFT_FLAT_PATH : String,
    pub MSL_MCAM_LEFT_INPAINT_PATH : String,
    pub MSL_MCAM_RIGHT_INPAINT_PATH : String,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct M20CalData {
    pub M20_INPAINT_MASK_RIGHT_PATH : String,
    pub M20_INPAINT_MASK_LEFT_PATH : String,
    pub M20_WATSON_INPAINT_MASK_PATH : String,
    pub M20_WATSON_FLAT_PATH : String,
    pub M20_SCAM_FLAT_PATH : String,
    pub M20_SCAM_MASK_PATH : String,
}

pub fn load_caldata_mapping_file() -> error::Result<Config> {
    let caldata_toml = locate_calibration_file("caldata.toml").unwrap();
    let mut file = match File::open(&caldata_toml) {
        Err(why) => panic!("couldn't open {}", why),
        Ok(file) => file,
    };

    let mut buf : Vec<u8> = Vec::default();
    file.read_to_end(&mut buf).unwrap();
    let toml = String::from_utf8(buf).unwrap();

    let config: Config = toml::from_str(&toml).unwrap();

    Ok(config)
}

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

pub fn locate_calibration_file(calib_file_name:&str) -> error::Result<String> {

    let mut file_path = calib_file_name.to_owned();

    match env::var("MARS_RAW_DATA") {
        Ok(d) => {
            file_path = format!("{}/{}", d, file_path); 
        },
        Err(_) => {
            let d = determine_data_dir();
            file_path = format!("{}/{}", d, file_path);
        }
    };

    match path::file_exists(&file_path) {
        true => Ok(file_path),
        false => Err(constants::status::FILE_NOT_FOUND)
    }   
}



pub fn calibration_file(calib_file_name:&str) -> error::Result<String> {
    let config = load_caldata_mapping_file().unwrap();

    match calib_file_name {
        constants::cal::M20_INPAINT_MASK_RIGHT_PATH => Ok(locate_calibration_file(config.m20.M20_INPAINT_MASK_RIGHT_PATH.as_str()).unwrap()),
        constants::cal::M20_INPAINT_MASK_LEFT_PATH => Ok(locate_calibration_file(config.m20.M20_INPAINT_MASK_LEFT_PATH.as_str()).unwrap()),
        constants::cal::M20_WATSON_INPAINT_MASK_PATH => Ok(locate_calibration_file(config.m20.M20_WATSON_INPAINT_MASK_PATH.as_str()).unwrap()),
        constants::cal::M20_WATSON_FLAT_PATH => Ok(locate_calibration_file(config.m20.M20_WATSON_FLAT_PATH.as_str()).unwrap()),
        constants::cal::M20_SCAM_FLAT_PATH => Ok(locate_calibration_file(config.m20.M20_SCAM_FLAT_PATH.as_str()).unwrap()),
        constants::cal::M20_SCAM_MASK_PATH => Ok(locate_calibration_file(config.m20.M20_SCAM_MASK_PATH.as_str()).unwrap()),

        constants::cal::MSL_MAHLI_INPAINT_MASK_PATH => Ok(locate_calibration_file(config.msl.MSL_MAHLI_INPAINT_MASK_PATH.as_str()).unwrap()),
        constants::cal::MSL_MAHLI_FLAT_PATH => Ok(locate_calibration_file(config.msl.MSL_MAHLI_FLAT_PATH.as_str()).unwrap()),
        constants::cal::MSL_NCAM_RIGHT_INPAINT_PATH => Ok(locate_calibration_file(config.msl.MSL_NCAM_RIGHT_INPAINT_PATH.as_str()).unwrap()),
        constants::cal::MSL_NCAM_RIGHT_FLAT_PATH => Ok(locate_calibration_file(config.msl.MSL_NCAM_RIGHT_FLAT_PATH.as_str()).unwrap()),
        constants::cal::MSL_NCAM_LEFT_FLAT_PATH => Ok(locate_calibration_file(config.msl.MSL_NCAM_LEFT_FLAT_PATH.as_str()).unwrap()),
        constants::cal::MSL_FHAZ_RIGHT_FLAT_PATH => Ok(locate_calibration_file(config.msl.MSL_FHAZ_RIGHT_FLAT_PATH.as_str()).unwrap()),
        constants::cal::MSL_FHAZ_LEFT_FLAT_PATH => Ok(locate_calibration_file(config.msl.MSL_FHAZ_LEFT_FLAT_PATH.as_str()).unwrap()),
        constants::cal::MSL_RHAZ_RIGHT_FLAT_PATH => Ok(locate_calibration_file(config.msl.MSL_RHAZ_RIGHT_FLAT_PATH.as_str()).unwrap()),
        constants::cal::MSL_RHAZ_LEFT_FLAT_PATH => Ok(locate_calibration_file(config.msl.MSL_RHAZ_LEFT_FLAT_PATH.as_str()).unwrap()),
        constants::cal::MSL_MCAM_LEFT_INPAINT_PATH => Ok(locate_calibration_file(config.msl.MSL_MCAM_LEFT_INPAINT_PATH.as_str()).unwrap()),
        constants::cal::MSL_MCAM_RIGHT_INPAINT_PATH => Ok(locate_calibration_file(config.msl.MSL_MCAM_RIGHT_INPAINT_PATH.as_str()).unwrap()),

        _ => Err(constants::status::INVALID_CALIBRATION_FILE_ID)
    }
}
