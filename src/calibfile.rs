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
    pub nsyt: NsytCalData
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct InstrumentProperties {
    pub flat: String,
    pub inpaint_mask: String,
    pub mask: String
}


#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct MslCalData {
    pub mahli: InstrumentProperties,
    pub nav_right: InstrumentProperties,
    pub nav_left: InstrumentProperties,
    pub fhaz_right: InstrumentProperties, // We're gonna ignore ECAMs on RCE-A for now
    pub fhaz_left: InstrumentProperties,
    pub rhaz_right: InstrumentProperties,
    pub rhaz_left: InstrumentProperties,
    pub mastcam_right: InstrumentProperties,
    pub mastcam_left: InstrumentProperties,
    pub chemcam: InstrumentProperties
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct M20CalData {
    pub mastcamz_right: InstrumentProperties,
    pub mastcamz_left: InstrumentProperties,
    pub watson: InstrumentProperties,
    pub supercam_rmi: InstrumentProperties
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct NsytCalData {
    pub idc: InstrumentProperties,
    pub icc: InstrumentProperties
}

pub fn load_caldata_mapping_file() -> error::Result<Config> {
    let caldata_toml = locate_calibration_file(String::from("caldata.toml")).unwrap();
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

pub fn locate_calibration_file(file_path:String) -> error::Result<String> {

    let mut fp = file_path.to_owned();

    match env::var("MARS_RAW_DATA") {
        Ok(d) => {
            fp = format!("{}/{}", d, fp); 
        },
        Err(_) => {
            let d = determine_data_dir();
            fp = format!("{}/{}", d, fp);
        }
    };

    match path::file_exists(&fp) {
        true => Ok(fp),
        false => Err(constants::status::FILE_NOT_FOUND)
    }   
}



pub fn calibration_file(calib_file_name:&str) -> error::Result<String> {
    let config = load_caldata_mapping_file().unwrap();

    match calib_file_name {
        constants::cal::M20_INPAINT_MASK_RIGHT_PATH => Ok(locate_calibration_file(config.m20.mastcamz_right.inpaint_mask).unwrap()),
        constants::cal::M20_INPAINT_MASK_LEFT_PATH => Ok(locate_calibration_file(config.m20.mastcamz_left.inpaint_mask).unwrap()),
        constants::cal::M20_WATSON_INPAINT_MASK_PATH => Ok(locate_calibration_file(config.m20.watson.inpaint_mask).unwrap()),
        constants::cal::M20_WATSON_FLAT_PATH => Ok(locate_calibration_file(config.m20.watson.flat).unwrap()),
        constants::cal::M20_SCAM_FLAT_PATH => Ok(locate_calibration_file(config.m20.supercam_rmi.flat).unwrap()),
        constants::cal::M20_SCAM_MASK_PATH => Ok(locate_calibration_file(config.m20.supercam_rmi.mask).unwrap()),

        constants::cal::MSL_MAHLI_INPAINT_MASK_PATH => Ok(locate_calibration_file(config.msl.mahli.inpaint_mask).unwrap()),
        constants::cal::MSL_MAHLI_FLAT_PATH => Ok(locate_calibration_file(config.msl.mahli.flat).unwrap()),

        constants::cal::MSL_NCAM_RIGHT_INPAINT_PATH => Ok(locate_calibration_file(config.msl.nav_right.inpaint_mask).unwrap()),
        constants::cal::MSL_NCAM_RIGHT_FLAT_PATH => Ok(locate_calibration_file(config.msl.nav_right.flat).unwrap()),

        constants::cal::MSL_NCAM_LEFT_FLAT_PATH => Ok(locate_calibration_file(config.msl.nav_left.flat).unwrap()),

        constants::cal::MSL_FHAZ_RIGHT_FLAT_PATH => Ok(locate_calibration_file(config.msl.fhaz_right.flat).unwrap()),
        constants::cal::MSL_FHAZ_LEFT_FLAT_PATH => Ok(locate_calibration_file(config.msl.fhaz_left.flat).unwrap()),
        constants::cal::MSL_RHAZ_RIGHT_FLAT_PATH => Ok(locate_calibration_file(config.msl.rhaz_right.flat).unwrap()),
        constants::cal::MSL_RHAZ_LEFT_FLAT_PATH => Ok(locate_calibration_file(config.msl.rhaz_left.flat).unwrap()),
        constants::cal::MSL_MCAM_LEFT_INPAINT_PATH => Ok(locate_calibration_file(config.msl.mastcam_left.inpaint_mask).unwrap()),
        constants::cal::MSL_MCAM_RIGHT_INPAINT_PATH => Ok(locate_calibration_file(config.msl.mastcam_right.inpaint_mask).unwrap()),
        constants::cal::MSL_CCAM_MASK_PATH => Ok(locate_calibration_file(config.msl.chemcam.mask).unwrap()),
        
        constants::cal::NSYT_IDC_FLAT_PATH => Ok(locate_calibration_file(config.nsyt.idc.flat).unwrap()),
        constants::cal::NSYT_ICC_FLAT_PATH => Ok(locate_calibration_file(config.nsyt.icc.flat).unwrap()),

        _ => Err(constants::status::INVALID_CALIBRATION_FILE_ID)
    }
}
