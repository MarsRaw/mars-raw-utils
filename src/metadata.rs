use serde::{Deserialize, Serialize};

use crate::jsonfetch;

use sciimg::prelude::*;

use anyhow::Result;
use std::fs::File;
use std::io::Read;

pub trait ImageMetadata {
    fn get_link(&self) -> String;
    fn get_credit(&self) -> String;
    fn get_sol(&self) -> u32;
    fn get_imageid(&self) -> String;
    fn get_caption(&self) -> String;
    fn get_date_taken_utc(&self) -> String;
    fn get_date_taken_mars(&self) -> Option<String>;
    fn get_subframe_rect(&self) -> Option<Vec<f64>>;
    // fn get_dimension(&self) -> Option<&[f64]>;
    fn get_scale_factor(&self) -> u32;
    fn get_instrument(&self) -> String;
    fn get_filter_name(&self) -> Option<String>;
    fn get_camera_vector(&self) -> Option<Vec<f64>>;
    fn get_camera_model_component_list(&self) -> CameraModel;
    fn get_camera_position(&self) -> Option<Vec<f64>>;
    fn get_camera_model_type(&self) -> Option<String>;
    fn get_site(&self) -> Option<u32>;
    fn get_drive(&self) -> Option<u32>;
    fn get_mast_az(&self) -> Option<f64>;
    fn get_mast_el(&self) -> Option<f64>;
    fn get_sclk(&self) -> Option<f64>;
    fn get_date_received(&self) -> String;
    fn get_xyz(&self) -> Option<Vec<f64>>;
    fn get_dimension(&self) -> Option<Vec<f64>>;
    fn get_sample_type(&self) -> String;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub link: String,
    pub credit: String,
    pub sol: u32,
    pub imageid: String,
    pub caption: String,
    pub date_taken_utc: String,
    pub date_taken_mars: Option<String>,
    pub subframe_rect: Option<Vec<f64>>,
    pub scale_factor: u32,
    pub instrument: String,
    pub filter_name: Option<String>,
    pub camera_vector: Option<Vec<f64>>,
    pub mast_az: Option<f64>,
    pub mast_el: Option<f64>,
    pub sclk: Option<f64>,

    #[serde(default = "crate::jsonfetch::default_blank")]
    pub date_received: String,

    #[serde(default = "crate::jsonfetch::default_blank")]
    pub sample_type: String,

    #[serde(
        with = "crate::jsonfetch::tuple_format",
        default = "crate::jsonfetch::default_vec_f64_none"
    )]
    pub dimension: Option<Vec<f64>>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub camera_position: Option<Vec<f64>>,

    #[serde(
        with = "crate::jsonfetch::tuple_format",
        default = "crate::jsonfetch::default_vec_f64_none"
    )]
    pub xyz: Option<Vec<f64>>,

    pub camera_model_type: Option<String>,
    pub site: Option<u32>,
    pub drive: Option<u32>,

    #[serde(with = "crate::jsonfetch::cahvor_format")]
    pub camera_model_component_list: CameraModel,

    #[serde(default = "crate::jsonfetch::default_false")]
    pub decompand: bool,

    #[serde(default = "crate::jsonfetch::default_false")]
    pub debayer: bool,

    #[serde(default = "crate::jsonfetch::default_false")]
    pub flatfield: bool,

    #[serde(default = "crate::jsonfetch::default_false")]
    pub radiometric: bool,

    #[serde(default = "crate::jsonfetch::default_false")]
    pub inpaint: bool,

    #[serde(default = "crate::jsonfetch::default_false")]
    pub cropped: bool,
}

pub fn convert_to_std_metadata<T: ImageMetadata>(im: &T) -> Metadata {
    Metadata {
        link: im.get_link(),
        credit: im.get_credit(),
        sol: im.get_sol(),
        imageid: im.get_imageid(),
        caption: im.get_caption(),
        date_taken_utc: im.get_date_taken_utc(),
        date_taken_mars: im.get_date_taken_mars(),
        subframe_rect: im.get_subframe_rect(),
        scale_factor: im.get_scale_factor(),
        instrument: im.get_instrument(),
        filter_name: im.get_filter_name(),
        decompand: jsonfetch::default_false(),
        debayer: jsonfetch::default_false(),
        flatfield: jsonfetch::default_false(),
        radiometric: jsonfetch::default_false(),
        inpaint: jsonfetch::default_false(),
        cropped: jsonfetch::default_false(),
        camera_vector: im.get_camera_vector(),
        camera_model_component_list: im.get_camera_model_component_list(),
        camera_position: im.get_camera_position(),
        camera_model_type: im.get_camera_model_type(),
        site: im.get_site(),
        drive: im.get_drive(),
        mast_el: im.get_mast_el(),
        mast_az: im.get_mast_az(),
        sclk: im.get_sclk(),
        dimension: im.get_dimension(),
        xyz: im.get_xyz(),
        date_received: im.get_date_received(),
        sample_type: im.get_sample_type(),
    }
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
