use crate::constants;
use crate::jsonfetch;
use crate::metadata::Metadata;
use crate::metadata::MetadataConvert;
use crate::msl::metadata::MslApiResults;
use crate::remotequery;
use crate::remotequery::FetchError;
use crate::util::{InstrumentMap, stringvec, stringvec_b};
use crate::{f, t};
use anyhow::Result;
use anyhow::anyhow;
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
        stringvec_b("search", query.cameras.join("%7C")),
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

fn parse_api_results_json(raw_json: &str) -> Result<MslApiResults> {
    match serde_json::from_str(raw_json) {
        Ok(api_results) => Ok(api_results),
        Err(why) => Err(anyhow!("Error parsing MSL API results JSON: {:?}", why)),
    }
}

fn parse_msl_latest_json(raw_json: &str) -> Result<MslLatest> {
    match serde_json::from_str(raw_json) {
        Ok(api_results) => Ok(api_results),
        Err(why) => Err(anyhow!(
            "Error parsing MSL Lastest Images API results JSON: {:?}",
            why
        )),
    }
}

/// Submits the query via `submit_query` then deserializes it through serde to a `M20ApiResults` object
async fn fetch_page(query: &remotequery::RemoteQuery) -> Result<MslApiResults> {
    parse_api_results_json(&submit_query(query).await?)
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

    image_records.map(|f| f.to_mru_metadata()).collect()
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
        let req = match jsonfetch::JsonFetcher::new(constants::url::MSL_LATEST_WEBSERVICE_URL) {
            Ok(req) => req,
            Err(why) => {
                return Err(FetchError::ProgrammingError(f!(
                    "Failed to create json fetch object {:?}",
                    why
                )));
            }
        };

        let json = match req.fetch_str().await {
            Ok(json) => json,
            Err(why) => {
                return Err(FetchError::RemoteError(f!(
                    "Error fetching from remote MSL API: {:?}",
                    why
                )));
            }
        };

        match parse_msl_latest_json(&json) {
            Ok(res) => Ok(Box::new(res)),
            Err(why) => {
                return Err(FetchError::ParseError(f!(
                    "Error parsing remote JSON from MSL API {:?}",
                    why
                )));
            }
        }
    }

    async fn fetch_stats(
        &self,
        query: &remotequery::RemoteQuery,
    ) -> Result<remotequery::RemoteStats, FetchError> {
        let raw_json = match submit_query(query).await {
            Ok(raw_json) => raw_json,
            Err(why) => {
                return Err(FetchError::RemoteError(f!(
                    "Error fetching remote JSON from MSL API {:?}",
                    why
                )));
            }
        };

        let api_results: MslApiResults = match serde_json::from_str(&raw_json) {
            Ok(api_results) => api_results,
            Err(why) => {
                return Err(FetchError::ParseError(f!(
                    "Failed to parse MSL API JSON output: {:?}",
                    why
                )));
            }
        };

        let pages = (api_results.total as f32 / query.num_per_page as f32).ceil() as i32;
        Ok(remotequery::RemoteStats {
            more: (api_results.page as i32) < pages - 1, // Assuming a zero-indexed page number
            error_message: String::from(""),
            total_results: api_results.total as i32,
            page: api_results.page as i32,
            total_images: api_results.total as i32,
        })
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

#[test]
fn test_parse_api_results_json() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/marsapi")
        .join("msl_fetch.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_api_results_json(&raw_json).expect("Failed to parse json");
}

#[test]
fn test_parse_msl_latest_json() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/marsapi")
        .join("msl_latest.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_msl_latest_json(&raw_json).expect("Failed to parse json");
}
