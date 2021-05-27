
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
