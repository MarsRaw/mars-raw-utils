use crate::{constants, metadata::*};

use sciimg::prelude::*;

use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Extended {
    #[serde(alias = "mastAz")]
    pub mast_az: String,

    #[serde(alias = "mastEl")]
    pub mast_el: String,
    pub sclk: String,

    #[serde(alias = "scaleFactor")]
    pub scale_factor: String,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub xyz: Option<Vec<f64>>,

    #[serde(alias = "subframeRect")]
    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub subframe_rect: Option<Vec<f64>>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub dimension: Option<Vec<f64>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageFiles {
    pub medium: String,
    pub small: String,
    pub full_res: String,
    pub large: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Camera {
    pub filter_name: String,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub camera_vector: Option<Vec<f64>>,

    #[serde(with = "crate::jsonfetch::cahvor_format")]
    pub camera_model_component_list: CameraModel,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub camera_position: Option<Vec<f64>>,
    pub instrument: String,
    pub camera_model_type: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageRecord {
    pub extended: Extended,
    pub sol: u32,
    pub attitude: String,
    pub image_files: ImageFiles,
    pub imageid: String,
    pub camera: Camera,
    pub caption: String,
    pub sample_type: String,
    pub date_taken_mars: String,
    pub credit: String,
    pub date_taken_utc: String,
    pub json_link: String,
    pub link: String,
    pub drive: String,
    pub title: String,
    pub site: u32,
    pub date_received: String,
}

#[derive(Serialize, Deserialize)]
pub struct M20ApiResults {
    pub images: Vec<ImageRecord>,
    pub per_page: String,
    pub total_results: u32,

    // Skip this for now. Some times this is encoded as a number, other times it's a string
    #[serde(skip_deserializing)]
    pub page: u32,
    pub mission: String,
    pub total_images: u32,
}

impl ImageMetadata for ImageRecord {
    fn get_date_received(&self) -> String {
        self.date_received.clone()
    }

    fn get_xyz(&self) -> Option<Vec<f64>> {
        self.extended.xyz.as_ref().cloned()
    }

    fn get_dimension(&self) -> Option<Vec<f64>> {
        self.extended.dimension.as_ref().cloned()
    }

    fn get_sample_type(&self) -> String {
        self.sample_type.clone()
    }

    fn get_link(&self) -> String {
        self.image_files.full_res.clone()
    }

    fn get_credit(&self) -> String {
        self.credit.clone()
    }

    fn get_sol(&self) -> u32 {
        self.sol
    }

    fn get_imageid(&self) -> String {
        self.imageid.clone()
    }

    fn get_caption(&self) -> String {
        self.caption.clone()
    }

    fn get_date_taken_utc(&self) -> String {
        self.date_taken_utc.clone()
    }

    fn get_date_taken_mars(&self) -> Option<String> {
        Some(self.date_taken_mars.clone())
    }

    fn get_subframe_rect(&self) -> Option<Vec<f64>> {
        self.extended.subframe_rect.as_ref().cloned()
    }

    fn get_scale_factor(&self) -> u32 {
        if self.extended.scale_factor == "UNK" {
            return 1;
        }

        let sf = self.extended.scale_factor.parse::<u32>();
        sf.unwrap_or(1)
    }

    fn get_instrument(&self) -> String {
        self.camera.instrument.clone()
    }

    fn get_filter_name(&self) -> Option<String> {
        Some(self.camera.filter_name.clone())
    }

    fn get_camera_vector(&self) -> Option<Vec<f64>> {
        self.camera.camera_vector.clone()
    }

    fn get_camera_model_component_list(&self) -> CameraModel {
        self.camera.camera_model_component_list.clone()
    }

    fn get_camera_position(&self) -> Option<Vec<f64>> {
        self.camera.camera_position.clone()
    }

    fn get_camera_model_type(&self) -> Option<String> {
        Some(self.camera.camera_model_type.clone())
    }

    fn get_site(&self) -> Option<u32> {
        Some(self.site)
    }

    fn get_drive(&self) -> Option<u32> {
        match self.drive.parse::<u32>() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    fn get_mast_az(&self) -> Option<f64> {
        match self.extended.mast_az.parse::<f64>() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    fn get_mast_el(&self) -> Option<f64> {
        match self.extended.mast_el.parse::<f64>() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    fn get_sclk(&self) -> Option<f64> {
        match self.extended.sclk.parse::<f64>() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    fn is_thumbnail(&self) -> bool {
        self.sample_type == "Thumbnail"
    }

    fn get_remote_image_url(&self) -> String {
        self.image_files.full_res.clone()
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

    Ok(convert_to_std_metadata(&res))
}
