use crate::enums::Mission;
use crate::metadata::MissionMetadata::Flight;
use crate::serializers::as_utc_datetime;
use crate::{constants, metadata::*};
use chrono::DateTime;
use chrono::Utc;
use sciimg::prelude::*;

use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageRecord {
    pub id: u32,
    pub image_id: String,
    pub url: String,
    #[serde(with = "as_utc_datetime")]
    pub date: DateTime<Utc>,
    #[serde(with = "as_utc_datetime")]
    pub date_received: DateTime<Utc>,
    pub width: u32,
    pub height: u32,
    pub camera: String,
    pub camera_name: String,
    pub camera_title: String,
    pub filter: u32,
    pub filter_name: String,
    pub filter_wavelength: String,
    pub target: Option<String>,
    pub distance: Option<u32>,
    pub orbit_number: Option<u32>,
    pub spacecraft_clock: Option<f64>,
    #[serde(with = "as_utc_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "as_utc_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct PsycheApiResults {
    pub items: Vec<ImageRecord>,
    pub per_page: u32,
    pub total: u32,
    pub page: u32,
    pub more: bool,
}

impl MetadataConvert for ImageRecord {
    fn to_mru_metadata(&self) -> Metadata {
        Metadata {
            flight_mission: Mission::Psyche,
            processing: ProcessingState::default(),
            image: ImageMetadata {
                link: self.url.clone(),
                credit: constants::credit::PSYCHE_IMAGER_DEFAULT_CREDIT.to_owned(),
                imageid: self.image_id.clone(),
                caption: "".to_owned(),
                date_taken_utc: self.date,
                date_received: self.date_received,
                remote_image_url: self.url.clone(),
                created_at: Some(self.created_at),
                updated_at: Some(self.updated_at),
                derived_filename: assemble_output_file_base_name(self),
            },
            mission: Flight(FlightMetadata {
                id: self.id,
                width: self.width,
                height: self.height,
                instrument: self.camera.clone(),
                camera_name: self.camera_name.clone(),
                camera_title: self.camera_title.clone(),
                filter: self.filter,
                filter_name: self.filter_name.clone(),
                filter_wavelength: self.filter_wavelength.clone(),
                target: self.target.clone(),
                distance: self.distance,
                orbit_number: self.orbit_number,
                spacecraft_clock: self.spacecraft_clock,
                subframe_rect: None,
            }),
        }
    }
}

fn assemble_output_file_base_name(image_rec: &ImageRecord) -> String {
    format!(
        "PSY_{}_{}_{}.png",
        match image_rec.spacecraft_clock {
            Some(sc) => ((sc * 100.0).round() / 100.0).to_string().replace(".", "-"),
            None => image_rec.image_id.clone(),
        },
        image_rec.camera,
        image_rec.filter_name.to_uppercase().replace(" ", "_"),
    )
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
