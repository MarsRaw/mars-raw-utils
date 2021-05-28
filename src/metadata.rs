
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
}


pub struct Metadata  {
    link:String,
    credit:String,
    sol:u32,
    imageid:String,
    caption:String,
    date_taken_utc:String,
    date_taken_mars:Option<String>,
    subframe_rect:Option<Vec<f64>>,
    scale_factor:u32
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
        scale_factor:im.get_scale_factor()
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

}
