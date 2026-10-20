use crate::enums::Mission;
use crate::serializers::{
    as_cahvore, as_quaternion, as_tuple, as_utc_datetime, as_utc_datetime_opt, default_blank,
    default_false, default_vec_f64_none,
};
use chrono::{DateTime, Utc};
use sciimg::prelude::*;
use sciimg::quaternion::Quaternion;
use serde::{Deserialize, Serialize};

use anyhow::Result;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ProcessingState {
    pub decompand: bool,
    pub debayer: bool,
    pub flatfield: bool,
    pub radiometric: bool,
    pub inpaint: bool,
    pub cropped: bool,
    pub history: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ImageMetadata {
    pub link: String,
    pub credit: String,
    pub imageid: String,
    pub caption: String,
    #[serde(with = "as_utc_datetime")]
    pub date_taken_utc: DateTime<Utc>,
    #[serde(with = "as_utc_datetime")]
    pub date_received: DateTime<Utc>,
    #[serde(default = "default_blank")]
    pub remote_image_url: String,
    #[serde(with = "as_utc_datetime_opt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "as_utc_datetime_opt")]
    pub updated_at: Option<DateTime<Utc>>,
    pub derived_filename: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RoverMetadata {
    pub sol: u32,

    pub date_taken_mars: Option<String>,
    pub subframe_rect: Option<Vec<f64>>,
    pub scale_factor: u32,
    pub instrument: String,
    pub filter_name: Option<String>,
    pub camera_vector: Option<Vec<f64>>,
    pub mast_az: Option<f64>,
    pub mast_el: Option<f64>,
    pub sclk: Option<f64>,

    #[serde(default = "default_false")]
    pub thumbnail: bool,

    #[serde(default = "default_blank")]
    pub sample_type: String,

    #[serde(with = "as_tuple", default = "default_vec_f64_none")]
    pub dimension: Option<Vec<f64>>,

    #[serde(with = "as_tuple")]
    pub camera_position: Option<Vec<f64>>,

    #[serde(with = "as_tuple", default = "default_vec_f64_none")]
    pub xyz: Option<Vec<f64>>,

    pub camera_model_type: Option<String>,
    pub site: Option<u32>,
    pub drive: Option<u32>,

    #[serde(with = "as_cahvore")]
    pub camera_model_component_list: Cahvore,

    #[serde(with = "as_quaternion")]
    pub attitude: Option<Quaternion>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PsycheMetadata {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub instrument: String,
    pub camera_name: String,
    pub camera_title: String,
    pub filter: u32,
    pub filter_name: String,
    pub filter_wavelength: String,
    pub target: Option<String>,
    pub distance: Option<u32>,
    pub orbit_number: Option<u32>,
    pub spacecraft_clock: Option<f64>,
    pub subframe_rect: Option<Vec<f64>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum MissionMetadata {
    Rover(RoverMetadata),
    Psyche(PsycheMetadata),
}

impl Default for MissionMetadata {
    fn default() -> Self {
        MissionMetadata::Rover(RoverMetadata::default())
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Metadata {
    pub flight_mission: Mission,
    pub processing: ProcessingState,
    pub image: ImageMetadata,
    pub mission: MissionMetadata,
}

impl Metadata {
    pub fn imageid(&self) -> &str {
        &self.image.imageid
    }

    pub fn instrument(&self) -> String {
        match &self.mission {
            MissionMetadata::Rover(m) => m.instrument.clone(),
            MissionMetadata::Psyche(m) => m.instrument.clone(),
        }
    }

    pub fn from_rover(&mut self, rover_metadata: &RoverMetadata) {
        self.mission = MissionMetadata::Rover(rover_metadata.clone())
    }

    pub fn from_psyche(&mut self, psyche_metadata: &PsycheMetadata) {
        self.mission = MissionMetadata::Psyche(psyche_metadata.clone())
    }

    pub fn as_rover(&self) -> Option<&RoverMetadata> {
        if let MissionMetadata::Rover(m) = &self.mission {
            Some(m)
        } else {
            None
        }
    }

    pub fn as_psyche(&self) -> Option<&PsycheMetadata> {
        if let MissionMetadata::Psyche(m) = &self.mission {
            Some(m)
        } else {
            None
        }
    }
}

pub trait MetadataConvert {
    fn to_mru_metadata(&self) -> Metadata;
}

pub fn load_image_metadata(json_path: &String) -> Result<Metadata> {
    let mut file = match File::open(json_path) {
        Err(why) => panic!("couldn't open {}", why),
        Ok(file) => file,
    };

    let mut buf: Vec<u8> = Vec::default();
    file.read_to_end(&mut buf).unwrap();
    let json = String::from_utf8(buf).unwrap();

    let metadata = serde_json::from_str(&json).unwrap();

    Ok(metadata)
}
