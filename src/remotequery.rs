#[derive(Debug, Clone)]
pub struct RemoteQuery {
    pub cameras: Vec<String>,
    pub num_per_page: i32,
    pub page: Option<i32>,
    pub minsol: i32,
    pub maxsol: i32,
    pub thumbnails: bool,
    pub movie_only: bool,
    pub list_only: bool,
    pub search: Vec<String>,
    pub only_new: bool,
    pub product_types: Vec<String>,
    pub output_path: String,
}
