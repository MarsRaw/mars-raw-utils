use crate::enums::Mission;
use crate::metadata::MissionMetadata::Surface;
use crate::serializers::{as_cahvore, as_quaternion, as_tuple, as_utc_datetime};
use crate::{constants, metadata::*};
use chrono::{DateTime, Utc};
use sciimg::prelude::*;
use sciimg::quaternion::Quaternion;

use std::fs::File;
use std::io::Read;

use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Extended {
    //FIXME make datetime type
    pub localtime: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageRecord {
    //FIXME: make uuid type
    pub id: u32,

    #[serde(with = "as_tuple")]
    pub camera_vector: Option<Vec<f64>>,

    pub site: Option<u32>,
    //FIXME make uuid type
    pub imageid: String,

    #[serde(with = "as_tuple")]
    pub subframe_rect: Option<Vec<f64>>,

    pub sol: u32,
    pub scale_factor: u32,

    #[serde(with = "as_cahvore")]
    pub camera_model_component_list: Cahvore,

    pub instrument: String,
    //FIXME make url type
    pub url: String,
    // FIXME use different type?
    pub spacecraft_clock: f64,

    #[serde(with = "as_quaternion")]
    pub attitude: Option<Quaternion>,

    #[serde(with = "as_tuple")]
    pub camera_position: Option<Vec<f64>>,

    pub camera_model_type: Option<String>,

    pub drive: Option<u32>,

    #[serde(with = "as_tuple")]
    pub xyz: Option<Vec<f64>>,

    #[serde(with = "as_utc_datetime")]
    pub created_at: DateTime<Utc>,

    #[serde(with = "as_utc_datetime")]
    pub updated_at: DateTime<Utc>,
    pub mission: String,
    pub extended: Extended,

    #[serde(with = "as_utc_datetime")]
    pub date_taken: DateTime<Utc>,

    #[serde(with = "as_utc_datetime")]
    pub date_received: DateTime<Utc>,
    pub instrument_sort: u32,
    pub sample_type_sort: u32,
    pub is_thumbnail: bool,
    pub title: String,
    pub description: String,
    // FIXME make url type
    pub link: String,
    pub image_credit: String,
    //FIXME make url type
    pub https_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct NsytApiResults {
    pub items: Vec<ImageRecord>,
    // FIXME update name
    pub more: bool,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

impl MetadataConvert for ImageRecord {
    fn to_mru_metadata(&self) -> Metadata {
        Metadata {
            flight_mission: Mission::InSight,
            processing: ProcessingState::default(),
            image: ImageMetadata {
                link: self.link.clone(),
                credit: self.image_credit.clone(),
                imageid: self.imageid.clone(),
                caption: "".to_owned(),
                date_taken_utc: self.date_taken,
                date_received: self.date_received,
                remote_image_url: self.url.clone(),
                created_at: Some(self.created_at),
                updated_at: Some(self.updated_at),
                derived_filename: path::basename(self.url.as_str()),
            },
            mission: Surface(SurfaceMetadata {
                sol: self.sol,
                date_taken_mars: Some(self.extended.localtime.clone()),
                subframe_rect: self.subframe_rect.clone(),
                scale_factor: self.scale_factor,
                instrument: self.instrument.clone(),
                filter_name: None,
                camera_vector: self.camera_vector.clone(),
                mast_az: None,
                mast_el: None,
                sclk: Some(self.spacecraft_clock),
                thumbnail: self.is_thumbnail,
                sample_type: String::new(),
                dimension: None,
                camera_position: self.camera_position.clone(),
                xyz: self.xyz.as_ref().cloned(),
                camera_model_type: self.camera_model_type.clone(),
                site: self.site,
                drive: self.drive,
                camera_model_component_list: self.camera_model_component_list.clone(),
                attitude: self.attitude,
            }),
        }
    }
}

pub fn load_metadata_file(file_path: String) -> Result<Metadata> {
    vprintln!("Loading metadata file from {}", file_path);

    if !path::file_exists(file_path.as_str()) {
        return Err(anyhow!(constants::status::FILE_NOT_FOUND));
    }

    let mut file = match File::open(&file_path) {
        Err(why) => panic!("couldn't open {}", why),
        Ok(file) => file,
    };

    let mut buf: Vec<u8> = Vec::default();
    file.read_to_end(&mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();

    let res: ImageRecord = serde_json::from_str(s.as_str()).unwrap();

    Ok(res.to_mru_metadata())
}
