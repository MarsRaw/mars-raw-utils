use crate::metadata::Metadata;
use anyhow::Result;
use async_trait::async_trait;

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

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct LatestData {
//     pub latest: String,
//     pub latest_sol: u16,
//     pub latest_sols: Vec<u16>,
//     pub new_count: u16,
//     pub sol_count: u16,
//     pub total: u32,

//     #[serde(alias = "type")]
//     pub result_type: String,
// }

pub trait LatestData {
    fn latest(&self) -> String;
    fn latest_sol(&self) -> u16;
    fn latest_sols(&self) -> Vec<u16>;
    fn new_count(&self) -> u16;
    fn sol_count(&self) -> u16;
    fn total(&self) -> u32;
}

#[async_trait]
pub trait Fetch {
    fn new() -> Self;
    async fn query_remote_images(query: &RemoteQuery) -> Result<Vec<Metadata>>;
    async fn fetch_latest() -> Result<Box<dyn LatestData>>;
}
