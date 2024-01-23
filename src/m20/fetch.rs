use crate::constants;
use crate::jsonfetch;
use crate::m20::metadata::*;
use crate::metadata::{convert_to_std_metadata, Metadata};
use crate::remotequery;
use crate::remotequery::FetchError;
use crate::util::{stringvec, stringvec_b, InstrumentMap};
use crate::{f, t};
use anyhow::Result;
use futures::future;
use serde::{Deserialize, Serialize};
use tokio;

/// Struct representation of the NASA API results from the 'latest' json endpoint
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

/// Implements the `LatestData` trait for M20LatestData which allows for the M20 API results to be translated to
/// the generic terms
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

/// Submits a query to the M20 api endpoint
async fn submit_query(query: &remotequery::RemoteQuery) -> Result<String> {
    let joined_cameras = query.cameras.join("%7C");

    let mut category = "mars2020";
    if query.cameras.contains(&String::from("HELI_NAV"))
        || query.cameras.contains(&String::from("HELI_RTE"))
    {
        category = "mars2020,ingenuity";
    }

    let mut params = vec![
        stringvec("feed", "raw_images"),
        stringvec("category", category),
        stringvec("feedtype", "json"),
        stringvec("ver", "1.2"),
        stringvec_b("num", format!("{}", query.num_per_page)),
        stringvec("order", "sol desc"),
        stringvec_b("search", joined_cameras),
        stringvec_b("condition_2", format!("{}:sol:gte", query.minsol)),
        stringvec_b("condition_3", format!("{}:sol:lte", query.maxsol)),
    ];

    if let Some(p) = query.page {
        params.push(stringvec_b("page", format!("{}", p)));
    }

    let mut extended: Vec<String> = vec![];
    if query.thumbnails {
        extended.push("sample_type::thumbnail".into());
    }

    if query.movie_only {
        extended.push("product_id::ecv".into());
    }

    query
        .product_types
        .iter()
        .for_each(|p| extended.push(format!("product_id::{}", p)));

    if !extended.is_empty() {
        params.push(stringvec_b("extended", extended.join(",")));
    }

    let uri = constants::url::M20_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri)?;

    for p in params {
        req.param(p[0].as_str(), p[1].as_str());
    }

    req.fetch_str().await
}

/// Submits the query via `submit_query` then deserializes it through serde to a `M20ApiResults` object
async fn fetch_page(query: &remotequery::RemoteQuery) -> Result<M20ApiResults> {
    match submit_query(query).await {
        Ok(v) => {
            let res: M20ApiResults = serde_json::from_str(v.as_str())?;

            Ok(res)
        }
        Err(e) => Err(e),
    }
}

/// Converts the M20-specific api results to a generic list of image metadata records
fn api_results_to_image_vec(
    results: &M20ApiResults,
    query: &remotequery::RemoteQuery,
) -> Vec<Metadata> {
    let image_records = results.images.iter().filter(|image| {
        !(image.sample_type == "Thumbnail" && !query.thumbnails
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
pub struct M20Fetch {}

impl M20Fetch {
    pub fn new() -> M20Fetch {
        M20Fetch {}
    }
}

impl remotequery::Fetch for M20Fetch {
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
        let uri = constants::url::M20_LATEST_WEBSERVICE_URL;

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
                let res: M20LatestData = serde_json::from_str(v.as_str()).unwrap();
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
                let res: M20ApiResults = serde_json::from_str(v.as_str()).unwrap();
                let pages = (res.total_results as f32 / query.num_per_page as f32).ceil() as i32;
                Ok(remotequery::RemoteStats {
                    more: (res.page as i32) < pages - 1, // Assuming a zero-indexed page number
                    error_message: String::from(""),
                    total_results: res.total_results as i32,
                    page: res.page as i32,
                    total_images: res.total_images as i32,
                })
            }
            Err(e) => Err(FetchError::RemoteError(f!("Remote error: {:?}", e))),
        }
    }

    fn make_instrument_map(&self) -> InstrumentMap {
        InstrumentMap {
            map: [
                (
                    "HAZ_FRONT",
                    vec![
                        "FRONT_HAZCAM_LEFT_A",
                        "FRONT_HAZCAM_LEFT_B",
                        "FRONT_HAZCAM_RIGHT_A",
                        "FRONT_HAZCAM_RIGHT_B",
                    ],
                ),
                ("SUPERCAM", vec!["SUPERCAM_RMI"]),
                ("HAZ_REAR", vec!["REAR_HAZCAM_LEFT", "REAR_HAZCAM_RIGHT"]),
                ("NAVCAM", vec!["NAVCAM_LEFT", "NAVCAM_RIGHT"]),
                ("MASTCAM", vec!["MCZ_LEFT", "MCZ_RIGHT"]),
                (
                    "EDLCAM",
                    vec![
                        "EDL_DDCAM",
                        "EDL_PUCAM1",
                        "EDL_PUCAM2",
                        "EDL_RUCAM",
                        "EDL_RDCAM",
                        "LCAM",
                    ],
                ),
                ("WATSON", vec!["SHERLOC_WATSON"]),
                ("HELI_NAV", vec!["HELI_NAV"]),
                ("HELI_RTE", vec!["HELI_RTE"]),
                ("CACHECAM", vec!["CACHECAM"]),
                ("PIXL", vec!["PIXL_MCC"]),
                ("SKYCAM", vec!["SKYCAM"]),
                ("SHERLOC", vec!["SHERLOC_ACI"]),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }
}
