use crate::constants;
use crate::jsonfetch;
use crate::metadata::{convert_to_std_metadata, Metadata};
use crate::nsyt::metadata::*;
use crate::remotequery;
use crate::util::{stringvec, stringvec_b, InstrumentMap};
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use futures::future;
use serde::{Deserialize, Serialize};
use tokio;

#[derive(Debug, Clone)]
pub struct NsytRemoteStats {
    pub more: bool,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NsytLatestData {
    pub latest: String,
    pub latest_sol: u16,
    pub latest_sols: Vec<u16>,
    pub new_count: u16,
    pub sol_count: u16,
    pub total: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NsytLatest {
    pub success: bool,
    pub latest_data: NsytLatestData,
}

/// Implements the `LatestData` trait for NsytLatest which allows for the M20 API results to be translated to
/// the generic terms
impl remotequery::LatestData for NsytLatest {
    fn latest(&self) -> String {
        self.latest_data.latest.clone()
    }

    fn latest_sol(&self) -> u16 {
        self.latest_data.latest_sol
    }

    fn latest_sols(&self) -> Vec<u16> {
        self.latest_data.latest_sols.clone()
    }

    fn new_count(&self) -> u16 {
        self.latest_data.new_count
    }

    fn sol_count(&self) -> u16 {
        self.latest_data.sol_count
    }

    fn total(&self) -> u32 {
        self.latest_data.total
    }
}

/// Submits a query to the M20 api endpoint
async fn submit_query(query: &remotequery::RemoteQuery) -> Result<String> {
    let mut params = vec![
        stringvec("condition_1", "insight:mission"),
        stringvec_b("per_page", format!("{}", query.num_per_page)),
        stringvec(
            "order",
            "sol desc,instrument_sort asc,sample_type_sort asc, date_taken desc",
        ),
        stringvec_b("search", query.cameras.join("|")),
        stringvec_b("condition_2", format!("{}:sol:gte", query.minsol)),
        stringvec_b("condition_3", format!("{}:sol:lte", query.maxsol)),
    ];

    if let Some(p) = query.page {
        params.push(stringvec_b("page", format!("{}", p)));
    }

    let uri = constants::url::NSYT_RAW_WEBSERVICE_URL;

    if let Ok(mut req) = jsonfetch::JsonFetcher::new(uri) {
        for p in params {
            req.param(p[0].as_str(), p[1].as_str());
        }
        return req.fetch_str().await;
    }

    Err(anyhow!("Unable to submit query."))
}

/// Submits the query via `submit_query` then deserializes it through serde to a `M20ApiResults` object
async fn fetch_page(query: &remotequery::RemoteQuery) -> Result<NsytApiResults> {
    match submit_query(query).await {
        Ok(v) => {
            let res: NsytApiResults = serde_json::from_str(v.as_str())?;

            Ok(res)
        }
        Err(e) => Err(e),
    }
}

/// Converts the M20-specific api results to a generic list of image metadata records
fn api_results_to_image_vec(
    results: &NsytApiResults,
    query: &remotequery::RemoteQuery,
) -> Vec<Metadata> {
    let image_records = results.items.iter().filter(|image| {
        !(image.is_thumbnail && !query.thumbnails
            || !query.search.is_empty() && !query.search.iter().any(|i| image.imageid.contains(i)))
    });

    image_records.map(convert_to_std_metadata).collect()
}

/// Fetches a page via `fetch_page` and filters the results through `api_results_to_image_vec`.
async fn fetch_page_as_metadata_vec(query: &remotequery::RemoteQuery) -> Result<Vec<Metadata>> {
    Ok(api_results_to_image_vec(&fetch_page(query).await?, query))
}

/// Container struct for the M20 remote fetch API implementation
#[derive(Clone, Default)]
pub struct NsytFetch {}

impl NsytFetch {
    pub fn new() -> NsytFetch {
        NsytFetch {}
    }

    pub fn new_boxed() -> remotequery::FetchType {
        Box::new(NsytFetch::new())
    }
}

#[async_trait]
impl remotequery::Fetch for NsytFetch {
    async fn query_remote_images(&self, query: &remotequery::RemoteQuery) -> Result<Vec<Metadata>> {
        let stats = self.fetch_stats(query).await?;

        Ok(if query.page.is_some() {
            let results = fetch_page(query).await?;
            api_results_to_image_vec(&results, query)
        } else {
            let pages = (stats.total_results as f32 / query.num_per_page as f32).ceil() as i32;

            let tasks: Vec<_> = (0..pages)
                .map(|page| {
                    let mut q: remotequery::RemoteQuery = query.clone();
                    q.page = Some(page);
                    tokio::spawn(async move { fetch_page_as_metadata_vec(&q).await })
                })
                .collect();

            let fetch_results = future::try_join_all(tasks).await?;
            fetch_results
                .into_iter()
                .flat_map(|md_vec| md_vec.unwrap())
                .collect()
        })
    }

    async fn fetch_latest(&self) -> Result<Box<dyn remotequery::LatestData>> {
        let uri = constants::url::NSYT_LATEST_WEBSERVICE_URL;

        let req = jsonfetch::JsonFetcher::new(uri)?;
        match req.fetch_str().await {
            Ok(v) => {
                let res: NsytLatest = serde_json::from_str(v.as_str()).unwrap();
                Ok(Box::new(res))
            }
            Err(e) => Err(anyhow!("Serde parsing from_str failed. {}", e)),
        }
    }

    async fn fetch_stats(
        &self,
        query: &remotequery::RemoteQuery,
    ) -> Result<remotequery::RemoteStats> {
        match submit_query(query).await {
            Ok(v) => {
                let res: NsytApiResults = serde_json::from_str(v.as_str()).unwrap();
                let pages = (res.total as f32 / query.num_per_page as f32).ceil() as i32;
                Ok(remotequery::RemoteStats {
                    more: (res.page as i32) < pages - 1, // Assuming a zero-indexed page number
                    error_message: String::from(""),
                    total_results: res.total as i32,
                    page: res.page as i32,
                    total_images: res.total as i32,
                })
            }
            Err(e) => Err(e),
        }
    }

    fn make_instrument_map(&self) -> InstrumentMap {
        InstrumentMap {
            map: [("IDC", vec!["idc"]), ("ICC", vec!["icc"])]
                .iter()
                .cloned()
                .collect(),
        }
    }
}
