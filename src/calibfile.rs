use std::env;

use crate::enums::CalFileType;
use crate::{constants, enums};

use sciimg::path;

extern crate dirs;

use std::fs::File;
use std::io::Read;

use anyhow::anyhow;
use anyhow::Result;

//use serde_derive::Deserialize;
use serde::Deserialize;

fn default_blank() -> String {
    "".to_string()
}

fn default_instrument_properties() -> InstrumentProperties {
    InstrumentProperties {
        flat: default_blank(),
        inpaint_mask: default_blank(),
        mask: default_blank(),
        lut: default_blank(),
    }
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub msl: MslCalData,
    pub m20: M20CalData,
    pub nsyt: NsytCalData,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct InstrumentProperties {
    #[serde(default = "default_blank")]
    pub flat: String,

    #[serde(default = "default_blank")]
    pub inpaint_mask: String,

    #[serde(default = "default_blank")]
    pub mask: String,

    #[serde(default = "default_blank")]
    pub lut: String,
}

#[derive(Clone)]
pub struct CalFilePathAndType {
    pub file: String,
    pub file_type: CalFileType,
}

impl IntoIterator for InstrumentProperties {
    type Item = CalFilePathAndType;
    type IntoIter = std::array::IntoIter<CalFilePathAndType, 4>;

    fn into_iter(self) -> Self::IntoIter {
        [
            CalFilePathAndType {
                file: self.flat,
                file_type: enums::CalFileType::FlatField,
            },
            CalFilePathAndType {
                file: self.inpaint_mask,
                file_type: enums::CalFileType::InpaintMask,
            },
            CalFilePathAndType {
                file: self.lut,
                file_type: enums::CalFileType::Lut,
            },
            CalFilePathAndType {
                file: self.mask,
                file_type: enums::CalFileType::Mask,
            },
        ]
        .into_iter()
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct MslCalData {
    #[serde(default = "default_instrument_properties")]
    pub mahli: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub nav_right: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub nav_left: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub fhaz_right: InstrumentProperties, // We're gonna ignore ECAMs on RCE-A for now

    #[serde(default = "default_instrument_properties")]
    pub fhaz_left: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub rhaz_right: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub rhaz_left: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub mastcam_right: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub mastcam_left: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub chemcam: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub mardi: InstrumentProperties,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct M20CalData {
    #[serde(default = "default_instrument_properties")]
    pub mastcamz_right: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub mastcamz_left: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub watson: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub supercam_rmi: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub nav_left: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub nav_right: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub fhaz_right: InstrumentProperties, // We're gonna use ECAMs on RCE-A for now

    #[serde(default = "default_instrument_properties")]
    pub fhaz_left: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub rhaz_right: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub rhaz_left: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub heli_nav: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub heli_rte: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub pixl_mcc: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub skycam: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub sherloc_aci: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub cachecam: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub edl_rdcam: InstrumentProperties,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct NsytCalData {
    #[serde(default = "default_instrument_properties")]
    pub idc: InstrumentProperties,

    #[serde(default = "default_instrument_properties")]
    pub icc: InstrumentProperties,
}

pub fn parse_caldata_from_string(caldata_toml_str: &str) -> Result<Config> {
    match toml::from_str(caldata_toml_str) {
        Ok(c) => Ok(c),
        Err(_) => Err(anyhow!("Failed to parse calibration manifest")),
    }
}

pub fn load_caldata_mapping_file() -> Result<Config> {
    if let Ok(caldata_toml) = locate_calibration_file(&String::from("caldata.toml")) {
        vprintln!("Loading calibration spec from {}", caldata_toml);

        let mut file = match File::open(&caldata_toml) {
            Err(why) => panic!("couldn't open {}", why),
            Ok(file) => file,
        };

        let mut buf: Vec<u8> = Vec::default();
        file.read_to_end(&mut buf).unwrap();
        let toml = String::from_utf8(buf).unwrap();

        parse_caldata_from_string(&toml)
    } else {
        Err(anyhow!("Unable to locate calibration configuration file"))
    }
}

/// Allows the user to specify files without an extension as a shortcut. Still needs to be able
/// to guess an extension, though
pub fn locate_calibration_file_no_extention(
    file_path: &String,
    extension: &String,
) -> Result<String> {
    match locate_calibration_file(file_path) {
        Ok(fp) => Ok(fp),
        Err(_) => {
            let with_ext = format!("{}{}", file_path, extension);
            locate_calibration_file(&with_ext)
        }
    }
}

pub fn locate_calibration_file(file_path: &str) -> Result<String> {
    // If the file exists as-is, return it
    if path::file_exists(file_path) {
        return Ok(file_path.into());
    }

    // Some default locations
    let mut locations = vec![
        String::from("mars-raw-utils-data/caldata"), // Running within the repo directory (dev: cargo run --bin ...)
        String::from("/usr/share/mars_raw_utils/data/"), // Linux, installed via apt or rpm
    ];

    if let Ok(exe_path) = std::env::current_exe() {
        if cfg!(windows) {
            // I'm not even a little comfortable with this...
            // So, to figure out the installation path, we get the path to the running executable, then get the path, and then
            // append 'data' to it to get to the calibration files. We also have to get rid of those quotation marks.
            if let Some(filename) = exe_path.parent() {
                locations.insert(
                    0,
                    format!("{:?}", filename.with_file_name("data").as_os_str()).replace('\"', ""),
                );
            }
        }
    }

    // Allow for a custom data path to be defined during build.
    if let Some(v) = option_env!("MARSDATAROOT") {
        locations.insert(0, String::from(v));
    }

    // Add a path based on the location of the running executable
    // Intended for Windows installations
    if let Ok(exe_path) = std::env::current_exe() {
        if cfg!(windows) {
            let bn = format!("{:?}/../data/", exe_path.file_name());
            locations.insert(0, bn);
        }
    }

    // Prepend a home directory if known
    if let Some(dir) = dirs::home_dir() {
        let homedatadir = format!("{}/.marsdata", dir.to_str().unwrap());
        locations.insert(0, homedatadir);
    }

    // Prepend a location specified by environment variable
    if let Ok(dir) = env::var("MARS_RAW_DATA") {
        locations.insert(0, dir);
    }

    // First match wins
    for loc in locations.iter() {
        let full_file_path = format!("{}/{}", loc, file_path);
        if path::file_exists(&full_file_path) {
            return Ok(full_file_path);
        }
    }

    // Oh nos!
    Err(anyhow!("Calibration file not found: {}", file_path))
}

pub fn get_calibration_file_for_type(
    inst_props: &InstrumentProperties,
    cal_file_type: enums::CalFileType,
) -> String {
    match cal_file_type {
        enums::CalFileType::FlatField => inst_props.flat.clone(),
        enums::CalFileType::InpaintMask => inst_props.inpaint_mask.clone(),
        enums::CalFileType::Mask => inst_props.mask.clone(),
        enums::CalFileType::Lut => inst_props.lut.clone(),
    }
}

pub fn get_calibration_base_file_for_instrument(
    instrument: enums::Instrument,
    cal_file_type: enums::CalFileType,
) -> Result<String> {
    let config = load_caldata_mapping_file()?;

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
        enums::Instrument::M20SherlocAci => Ok(get_calibration_file_for_type(
            &config.m20.sherloc_aci,
            cal_file_type,
        )),
        enums::Instrument::M20CacheCam => Ok(get_calibration_file_for_type(
            &config.m20.cachecam,
            cal_file_type,
        )),
        enums::Instrument::M20EdlRdcam => Ok(get_calibration_file_for_type(
            &config.m20.edl_rdcam,
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
        enums::Instrument::None => Err(anyhow!(constants::status::UNSUPPORTED_INSTRUMENT)),
    }
}

pub fn get_calibration_file_for_instrument(
    instrument: enums::Instrument,
    cal_file_type: enums::CalFileType,
) -> Result<String> {
    match get_calibration_base_file_for_instrument(instrument, cal_file_type) {
        Ok(file_name) => match file_name.len() {
            0 => Err(anyhow!(constants::status::UNSUPPORTED_INSTRUMENT)),
            _ => locate_calibration_file(&file_name),
        },
        Err(e) => Err(e),
    }
}
