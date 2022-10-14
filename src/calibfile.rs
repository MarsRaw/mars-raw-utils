use std::env;

use crate::{constants, enums, path};

use sciimg::error;

extern crate dirs;

use std::fs::File;
use std::io::Read;

//use serde_derive::Deserialize;
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub msl: MslCalData,
    pub m20: M20CalData,
    pub nsyt: NsytCalData,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct InstrumentProperties {
    pub flat: String,
    pub inpaint_mask: String,
    pub mask: String,
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
    pub heli_nav: InstrumentProperties,
    pub heli_rte: InstrumentProperties,
    pub pixl_mcc: InstrumentProperties,
    pub skycam: InstrumentProperties,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct NsytCalData {
    pub idc: InstrumentProperties,
    pub icc: InstrumentProperties,
}

pub fn load_caldata_mapping_file() -> error::Result<Config> {
    let cal_file_path = locate_calibration_file(&String::from("caldata.toml"));

    match cal_file_path {
        Ok(caldata_toml) => {
            let mut file = match File::open(&caldata_toml) {
                Err(why) => panic!("couldn't open {}", why),
                Ok(file) => file,
            };

            let mut buf: Vec<u8> = Vec::default();
            file.read_to_end(&mut buf).unwrap();
            let toml = String::from_utf8(buf).unwrap();

            let config: Config = toml::from_str(&toml).unwrap();

            Ok(config)
        }
        Err(_) => {
            panic!("Unable to locate calibration configuration file")
        }
    }
}

// Allows the user to specify files without an extension as a shortcut. Still needs to be able
// to guess an extension, though
pub fn locate_calibration_file_no_extention(
    file_path: &String,
    extension: &String,
) -> error::Result<String> {
    match locate_calibration_file(file_path) {
        Ok(fp) => Ok(fp),
        Err(_) => {
            let with_ext = format!("{}{}", file_path, extension);
            locate_calibration_file(&with_ext)
        }
    }
}

pub fn locate_calibration_file(file_path: &String) -> error::Result<String> {
    // If the file exists as-is, return it
    if path::file_exists(file_path) {
        return Ok(file_path.clone());
    }

    // Some default locations
    let mut locations = vec![
        String::from("mars-raw-utils-data/caldata"), // Running within the repo directory (dev: cargo run --bin ...)
        String::from("/usr/share/mars_raw_utils/data/"), // Linux, installed via apt or rpm
    ];

    match std::env::current_exe() {
        Ok(exe_path) => {
            if cfg!(windows) {
                // I'm not even a little comfortable with this...
                // So, to figure out the installation path, we get the path to the running executable, then get the path, and then
                // append 'data' to it to get to the calibration files. We also have to get rid of those quotation marks.
                if let Some(filename) = exe_path.parent() {
                    locations.insert(
                        0,
                        format!("{:?}", filename.with_file_name("data").as_os_str())
                            .replace('\"', ""),
                    );
                }
            }
        }
        Err(_) => {}
    };

    // Allow for a custom data path to be defined during build.
    match option_env!("MARSDATAROOT") {
        Some(v) => locations.insert(0, String::from(v)),
        None => {}
    };

    // Add a path based on the location of the running executable
    // Intended for Windows installations
    match std::env::current_exe() {
        Ok(exe_path) => {
            if cfg!(windows) {
                let bn = format!("{:?}/../data/", exe_path.file_name());
                locations.insert(0, bn);
            }
        }
        Err(_) => {}
    };

    // Prepend a home directory if known
    match dirs::home_dir() {
        Some(dir) => {
            let homedatadir = format!("{}/.marsdata", dir.to_str().unwrap());
            locations.insert(0, homedatadir);
        }
        None => {}
    };

    // Prepend a location specified by environment variable
    match env::var("MARS_RAW_DATA") {
        Ok(dir) => {
            locations.insert(0, dir);
        }
        Err(_) => {}
    };

    // First match wins
    for loc in locations.iter() {
        let full_file_path = format!("{}/{}", loc, file_path);
        if path::file_exists(&full_file_path) {
            return Ok(full_file_path);
        }
    }

    // Oh nos!
    Err(constants::status::FILE_NOT_FOUND)
}

pub fn get_calibration_file_for_type(
    inst_props: &InstrumentProperties,
    cal_file_type: enums::CalFileType,
) -> String {
    match cal_file_type {
        enums::CalFileType::FlatField => inst_props.flat.clone(),
        enums::CalFileType::InpaintMask => inst_props.inpaint_mask.clone(),
        enums::CalFileType::Mask => inst_props.mask.clone(),
    }
}

pub fn get_calibration_base_file_for_instrument(
    instrument: enums::Instrument,
    cal_file_type: enums::CalFileType,
) -> error::Result<String> {
    let config = load_caldata_mapping_file().unwrap();

    match instrument {
        enums::Instrument::MslMAHLI => Ok(get_calibration_file_for_type(
            &config.msl.mahli,
            cal_file_type,
        )),
        enums::Instrument::MslMastcamLeft => Ok(get_calibration_file_for_type(
            &config.msl.mastcam_left,
            cal_file_type,
        )),
        enums::Instrument::MslMastcamRight => Ok(get_calibration_file_for_type(
            &config.msl.mastcam_right,
            cal_file_type,
        )),
        enums::Instrument::MslNavCamRight => Ok(get_calibration_file_for_type(
            &config.msl.nav_right,
            cal_file_type,
        )), // Limiting to RCE-B camera for ECAM. For now.
        enums::Instrument::MslNavCamLeft => Ok(get_calibration_file_for_type(
            &config.msl.nav_left,
            cal_file_type,
        )),
        enums::Instrument::MslFrontHazLeft => Ok(get_calibration_file_for_type(
            &config.msl.fhaz_left,
            cal_file_type,
        )),
        enums::Instrument::MslFrontHazRight => Ok(get_calibration_file_for_type(
            &config.msl.fhaz_right,
            cal_file_type,
        )),
        enums::Instrument::MslRearHazLeft => Ok(get_calibration_file_for_type(
            &config.msl.rhaz_left,
            cal_file_type,
        )),
        enums::Instrument::MslRearHazRight => Ok(get_calibration_file_for_type(
            &config.msl.rhaz_right,
            cal_file_type,
        )),
        enums::Instrument::MslMARDI => Ok(get_calibration_file_for_type(
            &config.msl.mardi,
            cal_file_type,
        )),
        enums::Instrument::MslChemCam => Ok(get_calibration_file_for_type(
            &config.msl.chemcam,
            cal_file_type,
        )),
        enums::Instrument::M20MastcamZLeft => Ok(get_calibration_file_for_type(
            &config.m20.mastcamz_left,
            cal_file_type,
        )),
        enums::Instrument::M20MastcamZRight => Ok(get_calibration_file_for_type(
            &config.m20.mastcamz_right,
            cal_file_type,
        )),
        enums::Instrument::M20NavcamLeft => Ok(get_calibration_file_for_type(
            &config.m20.nav_left,
            cal_file_type,
        )),
        enums::Instrument::M20NavcamRight => Ok(get_calibration_file_for_type(
            &config.m20.nav_right,
            cal_file_type,
        )),
        enums::Instrument::M20FrontHazLeft => Ok(get_calibration_file_for_type(
            &config.m20.fhaz_left,
            cal_file_type,
        )),
        enums::Instrument::M20FrontHazRight => Ok(get_calibration_file_for_type(
            &config.m20.fhaz_right,
            cal_file_type,
        )),
        enums::Instrument::M20RearHazLeft => Ok(get_calibration_file_for_type(
            &config.m20.rhaz_left,
            cal_file_type,
        )),
        enums::Instrument::M20RearHazRight => Ok(get_calibration_file_for_type(
            &config.m20.rhaz_left,
            cal_file_type,
        )),
        enums::Instrument::M20Watson => Ok(get_calibration_file_for_type(
            &config.m20.watson,
            cal_file_type,
        )),
        enums::Instrument::M20SuperCam => Ok(get_calibration_file_for_type(
            &config.m20.supercam_rmi,
            cal_file_type,
        )),
        enums::Instrument::M20HeliNav => Ok(get_calibration_file_for_type(
            &config.m20.heli_nav,
            cal_file_type,
        )),
        enums::Instrument::M20HeliRte => Ok(get_calibration_file_for_type(
            &config.m20.heli_rte,
            cal_file_type,
        )),
        enums::Instrument::M20Pixl => Ok(get_calibration_file_for_type(
            &config.m20.pixl_mcc,
            cal_file_type,
        )),
        enums::Instrument::M20SkyCam => Ok(get_calibration_file_for_type(
            &config.m20.skycam,
            cal_file_type,
        )),
        enums::Instrument::NsytICC => Ok(get_calibration_file_for_type(
            &config.nsyt.icc,
            cal_file_type,
        )),
        enums::Instrument::NsytIDC => Ok(get_calibration_file_for_type(
            &config.nsyt.idc,
            cal_file_type,
        )),
        enums::Instrument::None => Err(constants::status::UNSUPPORTED_INSTRUMENT),
    }
}

pub fn get_calibration_file_for_instrument(
    instrument: enums::Instrument,
    cal_file_type: enums::CalFileType,
) -> error::Result<String> {
    match get_calibration_base_file_for_instrument(instrument, cal_file_type) {
        Ok(file_name) => match file_name.len() {
            0 => Err(constants::status::UNSUPPORTED_INSTRUMENT),
            _ => locate_calibration_file(&file_name),
        },
        Err(e) => Err(e),
    }
}
