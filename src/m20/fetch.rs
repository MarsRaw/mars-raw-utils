use crate::constants;
use crate::jsonfetch;
use crate::metadata::Metadata;
use crate::remotequery;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use sciimg::not_implemented;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct M20LatestData {
    pub latest: String,
    pub latest_sol: u16,
    pub latest_sols: Vec<u16>,
    pub new_count: u16,
    pub sol_count: u16,
    pub total: u32,

    #[serde(alias = "type")]
    pub result_type: String,
}

impl remotequery::LatestData for M20LatestData {
    fn latest(&self) -> String {
        self.latest.clone()
    }

    fn latest_sol(&self) -> u16 {
        self.latest_sol
    }

    fn latest_sols(&self) -> Vec<u16> {
        self.latest_sols.clone()
    }

    fn new_count(&self) -> u16 {
        self.new_count
    }

    fn sol_count(&self) -> u16 {
        self.sol_count
    }

    fn total(&self) -> u32 {
        self.total
    }
}

struct M20Fetch {}

#[async_trait]
impl remotequery::Fetch for M20Fetch {
    fn new() -> M20Fetch {
        M20Fetch {}
    }

    async fn query_remote_images(_query: &remotequery::RemoteQuery) -> Result<Vec<Metadata>> {
        not_implemented!()
    }

    async fn fetch_latest() -> Result<Box<dyn remotequery::LatestData>> {
        let uri = constants::url::M20_LATEST_WEBSERVICE_URL;

        let req = jsonfetch::JsonFetcher::new(uri)?;
        match req.fetch_str().await {
            Ok(v) => {
                let res: M20LatestData = serde_json::from_str(v.as_str()).unwrap();
                Ok(Box::new(res))
            }
            Err(e) => Err(anyhow!("Serde parsing from_str failed. {}", e)),
        }
    }
}
