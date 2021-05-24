use std::env;

use crate::{
    path,
    error,
    constants,
    enums
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
    pub chemcam: InstrumentProperties,
    pub mardi: InstrumentProperties,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct M20CalData {
    pub mastcamz_right: InstrumentProperties,
    pub mastcamz_left: InstrumentProperties,
    pub watson: InstrumentProperties,
    pub supercam_rmi: InstrumentProperties,
    pub nav_left: InstrumentProperties,
    pub nav_right: InstrumentProperties,
    pub fhaz_right: InstrumentProperties, // We're gonna use ECAMs on RCE-A for now
    pub fhaz_left: InstrumentProperties,
    pub rhaz_right: InstrumentProperties,
    pub rhaz_left: InstrumentProperties,
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
        String::from("mars-raw-utils-data/caldata")
    } else if cfg!(target_os = "macos") {
        String::from("/usr/local/share/mars_raw_utils/data/")
    } else if cfg!(target_os = "windows") {
        String::from("mars-raw-utils-data/caldata") // C:/something/something/something/darkside/
    } else {
        String::from("/usr/share/mars_raw_utils/data/")
    }
}

pub fn locate_calibration_file(file_path:String) -> error::Result<String> {

    let mut fp = file_path;

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



pub fn get_calibration_file_for_type(inst_props:&InstrumentProperties, cal_file_type:enums::CalFileType) -> String {
    match cal_file_type {
        enums::CalFileType::FlatField => inst_props.flat.clone(),
        enums::CalFileType::InpaintMask => inst_props.inpaint_mask.clone(),
        enums::CalFileType::Mask => inst_props.mask.clone()
    }
}

pub fn get_calibration_base_file_for_instrument(instrument:enums::Instrument, cal_file_type:enums::CalFileType) -> error::Result<String> {
    let config = load_caldata_mapping_file().unwrap();

    match instrument {
        enums::Instrument::MslMAHLI         => Ok(get_calibration_file_for_type(&config.msl.mahli, cal_file_type)),
        enums::Instrument::MslMastcamLeft   => Ok(get_calibration_file_for_type(&config.msl.mastcam_left, cal_file_type)),
        enums::Instrument::MslMastcamRight  => Ok(get_calibration_file_for_type(&config.msl.mastcam_right, cal_file_type)),
        enums::Instrument::MslNavCamRight   => Ok(get_calibration_file_for_type(&config.msl.nav_right, cal_file_type)), // Limiting to RCE-B camera for ECAM. For now.
        enums::Instrument::MslNavCamLeft    => Ok(get_calibration_file_for_type(&config.msl.nav_left, cal_file_type)),
        enums::Instrument::MslFrontHazLeft  => Ok(get_calibration_file_for_type(&config.msl.fhaz_left, cal_file_type)),
        enums::Instrument::MslFrontHazRight => Ok(get_calibration_file_for_type(&config.msl.fhaz_right, cal_file_type)),
        enums::Instrument::MslRearHazLeft   => Ok(get_calibration_file_for_type(&config.msl.rhaz_left, cal_file_type)),
        enums::Instrument::MslRearHazRight  => Ok(get_calibration_file_for_type(&config.msl.rhaz_right, cal_file_type)),
        enums::Instrument::MslMARDI         => Ok(get_calibration_file_for_type(&config.msl.mardi, cal_file_type)),
        enums::Instrument::MslChemCam       => Ok(get_calibration_file_for_type(&config.msl.chemcam, cal_file_type)),
        enums::Instrument::M20MastcamZLeft  => Ok(get_calibration_file_for_type(&config.m20.mastcamz_left, cal_file_type)),
        enums::Instrument::M20MastcamZRight => Ok(get_calibration_file_for_type(&config.m20.mastcamz_right, cal_file_type)),
        enums::Instrument::M20NavcamLeft    => Ok(get_calibration_file_for_type(&config.m20.nav_left, cal_file_type)),
        enums::Instrument::M20NavcamRight   => Ok(get_calibration_file_for_type(&config.m20.nav_right, cal_file_type)),
        enums::Instrument::M20FrontHazLeft  => Ok(get_calibration_file_for_type(&config.m20.fhaz_left, cal_file_type)),
        enums::Instrument::M20FrontHazRight => Ok(get_calibration_file_for_type(&config.m20.fhaz_right, cal_file_type)),
        enums::Instrument::M20RearHazLeft   => Ok(get_calibration_file_for_type(&config.m20.rhaz_left, cal_file_type)),
        enums::Instrument::M20RearHazRight  => Ok(get_calibration_file_for_type(&config.m20.rhaz_left, cal_file_type)),
        enums::Instrument::M20Watson        => Ok(get_calibration_file_for_type(&config.m20.watson, cal_file_type)),
        enums::Instrument::M20SuperCam      => Ok(get_calibration_file_for_type(&config.m20.supercam_rmi, cal_file_type)),
        enums::Instrument::NsytICC          => Ok(get_calibration_file_for_type(&config.nsyt.icc, cal_file_type)),
        enums::Instrument::NsytIDC          => Ok(get_calibration_file_for_type(&config.nsyt.idc, cal_file_type)),
        enums::Instrument::None             => Err(constants::status::UNSUPPORTED_INSTRUMENT)
    }
}

pub fn get_calibration_file_for_instrument(instrument:enums::Instrument, cal_file_type:enums::CalFileType) -> error::Result<String> {
    match get_calibration_base_file_for_instrument(instrument, cal_file_type) {
        Ok(file_name) => {
            match file_name.len() {
                0 => Err(constants::status::UNSUPPORTED_INSTRUMENT),
                _ => locate_calibration_file(file_name)
            }
        },
        Err(e) => Err(e)
    }
    
}
