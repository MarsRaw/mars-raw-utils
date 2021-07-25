use crate::{
    constants, 
    error,
    vprintln,
    path,
    cahvor::Cahvor,
    metadata::*
};

use std::fs::File;
use std::io::Read;

use serde::{
    Deserialize, 
    Serialize
};




#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Extended {
    pub lmst: Option<String>,
    pub bucket: String,
    pub mast_az: Option<String>,
    pub mast_el: Option<String>,
    pub url_list: String,
    pub contributor: String,
    pub filter_name: Option<String>,
    pub sample_type: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub extended: Extended,
    pub id: u32,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub camera_vector: Option<Vec<f64>>,
    pub site: Option<u32>,
    pub imageid: String,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub subframe_rect: Option<Vec<f64>>,
    pub sol: u32,
    pub scale_factor: Option<u32>,

    #[serde(with = "crate::jsonfetch::cahvor_format")]
    pub camera_model_component_list: Option<Cahvor>,
    pub instrument: String,
    pub url: String,
    pub spacecraft_clock: Option<f64>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub attitude: Option<Vec<f64>>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub camera_position: Option<Vec<f64>>,
    pub camera_model_type: Option<String>,
    pub drive: Option<u32>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub xyz: Option<Vec<f64>>,
    pub created_at: String,
    pub updated_at: String,
    pub mission: String,
    pub date_taken: String,
    pub date_received: String,
    pub instrument_sort: u32,
    pub sample_type_sort: u32,
    pub is_thumbnail: bool,
    pub title: String,
    pub description: String,
    pub link: String,
    pub image_credit: String,
    pub https_url: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MslApiResults {
    pub items: Vec<Image>,
    pub more: bool,
    pub total: u32,
    pub page: u32,
    pub per_page: u32
}


impl ImageMetadata for Image {
    fn get_link(&self) -> String {
        self.url.clone()
    }

    fn get_credit(&self) -> String {
        self.image_credit.clone()
    }

    fn get_sol(&self) -> u32 {
        self.sol
    }

    fn get_imageid(&self) -> String {
        self.imageid.clone()
    }

    fn get_caption(&self) -> String {
        self.description.clone()
    }

    fn get_date_taken_utc(&self) -> String {
        self.date_taken.clone()
    }

    fn get_date_taken_mars(&self) -> Option<String> {
        self.extended.lmst.clone()
    }

    fn get_subframe_rect(&self) -> Option<Vec<f64>> {
        match self.subframe_rect.as_ref() {
            Some(v) => Some(v.clone()),
            None => None
        }
    }

    fn get_instrument(&self) -> String {
        self.instrument.clone()
    }

    fn get_filter_name(&self) -> Option<String> {
        self.extended.filter_name.clone()
    }

    // Sigh, MastCam....
    // fn get_dimension(&self) -> Option<&[f64]> {
    //     self.extended.dimension
    // }

    fn get_scale_factor(&self) -> u32 {
        match self.scale_factor {
            Some(v) => v,
            None => 1
        }
    }

    fn get_camera_vector(&self) -> Option<Vec<f64>> {
        self.camera_vector.clone()
    }

    fn get_camera_model_component_list(&self) -> Option<Cahvor> {
        self.camera_model_component_list.clone()
    }

    fn get_camera_position(&self) -> Option<Vec<f64>> {
        self.camera_position.clone()
    }

    fn get_camera_model_type(&self) -> Option<String> {
        self.camera_model_type.clone()
    }

    fn get_site(&self) -> Option<u32> {
        self.site.clone()
    }

    fn get_drive(&self) -> Option<u32> {
        self.drive.clone()
    }

    fn get_mast_az(&self) -> Option<f64> {
        if let Some(ref mast_az_string) = self.extended.mast_az {
            match mast_az_string.parse::<f64>() {
                Ok(v) => Some(v),
                Err(_) => None
            }
        } else {
            None
        }
    }

    fn get_mast_el(&self) -> Option<f64> {
        if let Some(ref mast_el_string) = self.extended.mast_el {
            match mast_el_string.parse::<f64>() {
                Ok(v) => Some(v),
                Err(_) => None
            }
        } else {
            None
        }
    }

    fn get_sclk(&self) -> Option<f64> {
        self.spacecraft_clock.clone()
    }
}

pub fn load_metadata_file(file_path:String) -> error::Result<Metadata> {

    vprintln!("Loading metadata file from {}", file_path);

    if !path::file_exists(&file_path.as_str()) {
        return Err(constants::status::FILE_NOT_FOUND);
    }

    let mut file = match File::open(&file_path) {
        Err(why) => panic!("couldn't open {}", why),
        Ok(file) => file,
    };

    let mut buf : Vec<u8> = Vec::default();
    file.read_to_end(&mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();

    let res: Image = serde_json::from_str(&s.as_str()).unwrap();

    Ok(convert_to_std_metadata(&res))
}