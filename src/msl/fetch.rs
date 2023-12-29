use crate::constants;
use crate::jsonfetch;
use crate::metadata::{convert_to_std_metadata, Metadata};
use crate::msl::metadata::*;
use crate::remotequery;
use crate::remotequery::FetchError;
use crate::util::{stringvec, stringvec_b, InstrumentMap};
use crate::{f, t};
use anyhow::anyhow;
use anyhow::Result;
use futures::future;
use serde::{Deserialize, Serialize};
use tokio;

#[derive(Debug, Clone)]
pub struct MslRemoteStats {
    pub more: bool,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MslLatestData {
    pub latest: String,
    pub latest_sol: u16,
    pub latest_sols: Vec<u16>,
    pub new_count: u16,
    pub sol_count: u16,
    pub total: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MslLatest {
    pub success: bool,
    pub latest_data: MslLatestData,
}

/// Implements the `LatestData` trait for M20LatestData which allows for the M20 API results to be translated to
/// the generic terms
impl remotequery::LatestData for MslLatest {
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
        stringvec("condition_1", "msl:mission"),
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

    let uri = constants::url::MSL_RAW_WEBSERVICE_URL;

    if let Ok(mut req) = jsonfetch::JsonFetcher::new(uri) {
        for p in params {
            req.param(p[0].as_str(), p[1].as_str());
        }
        return req.fetch_str().await;
    }

    Err(anyhow!("Unable to submit query."))
}

/// Submits the query via `submit_query` then deserializes it through serde to a `M20ApiResults` object
async fn fetch_page(query: &remotequery::RemoteQuery) -> Result<MslApiResults> {
    match submit_query(query).await {
        Ok(v) => {
            let res: MslApiResults = serde_json::from_str(v.as_str())?;

            Ok(res)
        }
        Err(e) => Err(e),
    }
}

/// Converts the M20-specific api results to a generic list of image metadata records
fn api_results_to_image_vec(
    results: &MslApiResults,
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
pub struct MslFetch {}

impl MslFetch {
    pub fn new() -> MslFetch {
        MslFetch {}
    }
}

impl remotequery::Fetch for MslFetch {
    async fn query_remote_images(
        &self,
        query: &remotequery::RemoteQuery,
    ) -> Result<Vec<Metadata>, FetchError> {
        let stats = self.fetch_stats(query).await?;

        if query.page.is_some() {
            if let Ok(results) = fetch_page(query).await {
                Ok(api_results_to_image_vec(&results, query))
            } else {
                Err(FetchError::ProgrammingError(t!("Error fetching page")))
            }
        } else {
            let pages = (stats.total_results as f32 / query.num_per_page as f32).ceil() as i32;

            let tasks: Vec<_> = (0..pages)
                .map(|page| {
                    let mut q: remotequery::RemoteQuery = query.clone();
                    q.page = Some(page);
                    tokio::spawn(async move { fetch_page_as_metadata_vec(&q).await })
                })
                .collect();

            match future::try_join_all(tasks).await {
                Ok(r) => Ok(r.into_iter().flat_map(|md_vec| md_vec.unwrap()).collect()),
                Err(why) => Err(FetchError::ProgrammingError(format!("{:?}", why))),
            }
        }
    }

    async fn fetch_latest(&self) -> Result<Box<dyn remotequery::LatestData>, FetchError> {
        let uri = constants::url::MSL_LATEST_WEBSERVICE_URL;

        let req = match jsonfetch::JsonFetcher::new(uri) {
            Ok(req) => req,
            Err(why) => {
                return Err(FetchError::ProgrammingError(f!(
                    "Failed to create json fetch object {:?}",
                    why
                )))
            }
        };
        match req.fetch_str().await {
            Ok(v) => {
                let res: MslLatest = serde_json::from_str(v.as_str()).unwrap();
                Ok(Box::new(res))
            }
            Err(e) => Err(FetchError::ParseError(f!(
                "Serde parsing from_str failed. {}",
                e
            ))),
        }
    }

    async fn fetch_stats(
        &self,
        query: &remotequery::RemoteQuery,
    ) -> Result<remotequery::RemoteStats, FetchError> {
        match submit_query(query).await {
            Ok(v) => {
                let res: MslApiResults = serde_json::from_str(v.as_str()).unwrap();
                let pages = (res.total as f32 / query.num_per_page as f32).ceil() as i32;
                Ok(remotequery::RemoteStats {
                    more: (res.page as i32) < pages - 1, // Assuming a zero-indexed page number
                    error_message: String::from(""),
                    total_results: res.total as i32,
                    page: res.page as i32,
                    total_images: res.total as i32,
                })
            }
            Err(e) => Err(FetchError::RemoteError(t!(e))),
        }
    }

    fn make_instrument_map(&self) -> InstrumentMap {
        InstrumentMap {
            map: [
                (
                    "HAZ_FRONT",
                    vec!["FHAZ_RIGHT_A", "FHAZ_LEFT_A", "FHAZ_RIGHT_B", "FHAZ_LEFT_B"],
                ),
                (
                    "HAZ_REAR",
                    vec!["RHAZ_RIGHT_A", "RHAZ_LEFT_A", "RHAZ_RIGHT_B", "RHAZ_LEFT_B"],
                ),
                ("NAV_LEFT", vec!["NAV_LEFT_A", "NAV_LEFT_B"]),
                ("NAV_RIGHT", vec!["NAV_RIGHT_A", "NAV_RIGHT_B"]),
                ("CHEMCAM", vec!["CHEMCAM_RMI"]),
                ("MARDI", vec!["MARDI"]),
                ("MAHLI", vec!["MAHLI"]),
                ("MASTCAM", vec!["MAST_LEFT", "MAST_RIGHT"]),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }
}
