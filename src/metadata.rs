
use serde::{
    Deserialize, 
    Serialize
};

use crate::{
    error
};

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
    //fn get_dimension(&self) -> Option<&[f64]>;
    fn get_scale_factor(&self) -> u32;
    fn get_instrument(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata  {
    link:String,
    credit:String,
    sol:u32,
    imageid:String,
    caption:String,
    date_taken_utc:String,
    date_taken_mars:Option<String>,
    subframe_rect:Option<Vec<f64>>,
    scale_factor:u32,
    instrument:String,

    #[serde(default = "default_step_status")]
    pub decompand:bool,

    #[serde(default = "default_step_status")]
    pub debayer:bool,

    #[serde(default = "default_step_status")]
    pub flatfield:bool,

    #[serde(default = "default_step_status")]
    pub radiometric:bool,

    #[serde(default = "default_step_status")]
    pub inpaint:bool,

    #[serde(default = "default_step_status")]
    pub cropped:bool
}

fn default_step_status() -> bool {
    false
}

pub fn convert_to_std_metadata<T:ImageMetadata>(im:&T) -> Metadata {
    Metadata{
        link:im.get_link(),
        credit:im.get_credit(),
        sol:im.get_sol(),
        imageid:im.get_imageid(),
        caption:im.get_caption(),
        date_taken_utc:im.get_date_taken_utc(),
        date_taken_mars:im.get_date_taken_mars(),
        subframe_rect:im.get_subframe_rect(),
        scale_factor:im.get_scale_factor(),
        instrument:im.get_instrument(),
        decompand:default_step_status(),
        debayer:default_step_status(),
        flatfield:default_step_status(),
        radiometric:default_step_status(),
        inpaint:default_step_status(),
        cropped:default_step_status()
    }
}


impl Metadata {


    pub fn get_link(&self) -> String {
        self.link.clone()
    }

    pub fn get_credit(&self) -> String {
        self.credit.clone()
    }

    pub fn get_sol(&self) -> u32 {
        self.sol
    }

    pub fn get_imageid(&self) -> String {
        self.imageid.clone()
    }

    pub fn get_caption(&self) -> String {
        self.caption.clone()
    }

    pub fn get_date_taken_utc(&self) -> String {
        self.date_taken_utc.clone()
    }

    pub fn get_date_taken_mars(&self) -> Option<String> {
        self.date_taken_mars.clone()
    }

    pub fn get_subframe_rect(&self) -> Option<Vec<f64>> {
        self.subframe_rect.clone()
    }

    pub fn get_scale_factor(&self) -> u32 {
        self.scale_factor
    }

    pub fn get_instrument(&self) -> String {
        self.instrument.clone()
    }

}


pub fn load_image_metadata(json_path:&String) -> error::Result<Metadata> {
    let mut file = match File::open(&json_path) {
        Err(why) => panic!("couldn't open {}", why),
        Ok(file) => file,
    };

    let mut buf : Vec<u8> = Vec::default();
    file.read_to_end(&mut buf).unwrap();
    let json = String::from_utf8(buf).unwrap();

    let metadata = serde_json::from_str(&json).unwrap();

    Ok(metadata)
}