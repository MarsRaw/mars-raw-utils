use crate::constants;
use crate::jsonfetch;
use crate::metadata::Metadata;
use crate::metadata::MetadataConvert;
use crate::psyche::metadata::*;
use crate::remotequery;
use crate::remotequery::FetchError;
use crate::util::{stringvec, stringvec_b, InstrumentMap};
use crate::{f, t};
use anyhow::{anyhow, Result};
use futures::future;
use tokio;

/// Submits a query to the M20 api endpoint
async fn submit_query(query: &remotequery::RemoteQuery) -> Result<String> {
    let joined_cameras = query.cameras.join("|");

    //https://solarsystem.nasa.gov/api/v1/raw_image_psyche_items/?order=date_received+desc&per_page=60&page=0&search=(A)%3Acamera
    //https://solarsystem.nasa.gov/api/v1/raw_image_psyche_items/?order=date_received+desc&per_page=60&page=0&condition_1=2025%2F12%2F01%3Adate_received%3Agte&condition_2=2025-12-6%3Adate_received%3Alt&search=(A)%3Acamera
    //https://solarsystem.nasa.gov/api/v1/raw_image_psyche_items/?order=date_received+desc&per_page=60&page=0&condition_1=2025%2F12%2F01%3Adate_received%3Agte&condition_2=2025-12-6%3Adate_received%3Alt&search=(A%7CB)%3Acamera

    let start_date = if let Some(start_date) = &query.start_date {
        start_date
    } else {
        return Err(anyhow!("Start date is required for Psyche queries"));
    };

    let end_date = if let Some(end_date) = &query.end_date {
        end_date
    } else {
        return Err(anyhow!("End date is required for Psyche queries"));
    };

    let mut params = vec![
        stringvec("feedtype", "json"),
        stringvec_b("per_page", format!("{}", query.num_per_page)),
        stringvec("order", "date_received+desc"),
        stringvec_b("search", format!("({}):camera", joined_cameras)),
        stringvec_b(
            "condition_1",
            format!("{}:date_received:gte", start_date.format("%Y-%m-%d")),
        ),
        stringvec_b(
            "condition_2",
            format!("{}:date_received:lte", end_date.format("%Y-%m-%d")),
        ),
    ];

    if let Some(p) = query.page {
        params.push(stringvec_b("page", format!("{}", p)));
    }

    let uri = constants::url::PSYCHE_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri)?;

    for p in params {
        req.param(p[0].as_str(), p[1].as_str());
    }

    req.fetch_str().await
}

/// Submits the query via `submit_query` then deserializes it through serde to a `PsycheApiResults` object
async fn fetch_page(query: &remotequery::RemoteQuery) -> Result<PsycheApiResults> {
    match submit_query(query).await {
        Ok(v) => {
            let res: PsycheApiResults = serde_json::from_str(v.as_str())?;

            Ok(res)
        }
        Err(e) => Err(e),
    }
}

/// Converts the M20-specific api results to a generic list of image metadata records
fn api_results_to_image_vec(
    results: &PsycheApiResults,
    query: &remotequery::RemoteQuery,
) -> Vec<Metadata> {
    let image_records = results
        .items
        .iter()
        .filter(|image| {
            !(!query.search.is_empty() && !query.search.iter().any(|i| image.image_id.contains(i)))
        })
        .filter(|image| {
            if let Some(filter) = &query.filter {
                filter.contains(&image.filter_name)
            } else {
                true
            }
        })
        .filter(|image| {
            if let Some(filter) = &query.filter_num {
                filter.contains(&image.filter)
            } else {
                true
            }
        })
        .filter(|image| {
            if let Some(filter) = &query.filter_wavelenth {
                filter.contains(&image.filter_wavelength)
            } else {
                true
            }
        });

    image_records.map(|f| f.to_mru_metadata()).collect()
}

/// Fetches a page via `fetch_page` and filters the results through `api_results_to_image_vec`.
async fn fetch_page_as_metadata_vec(query: &remotequery::RemoteQuery) -> Result<Vec<Metadata>> {
    Ok(api_results_to_image_vec(&fetch_page(query).await?, query))
}

/// Container struct for the M20 remote fetch API implementation
#[derive(Clone, Default)]
pub struct PsycheFetch {}

impl PsycheFetch {
    pub fn new() -> PsycheFetch {
        PsycheFetch {}
    }
}

impl remotequery::Fetch for PsycheFetch {
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

    async fn fetch_stats(
        &self,
        query: &remotequery::RemoteQuery,
    ) -> Result<remotequery::RemoteStats, FetchError> {
        match submit_query(query).await {
            Ok(v) => {
                let res: PsycheApiResults = serde_json::from_str(v.as_str()).unwrap();
                let pages = (res.total as f32 / query.num_per_page as f32).ceil() as i32;
                Ok(remotequery::RemoteStats {
                    more: (res.page as i32) < pages - 1, // Assuming a zero-indexed page number
                    error_message: String::from(""),
                    total_results: res.total as i32,
                    page: res.page as i32,
                    total_images: res.items.len() as i32,
                })
            }
            Err(e) => Err(FetchError::RemoteError(f!("Remote error: {:?}", e))),
        }
    }

    async fn fetch_latest(&self) -> Result<Box<dyn remotequery::LatestData>, FetchError> {
        Err(FetchError::NotImplemented)
    }

    /*

    https://solarsystem.nasa.gov/api/v1/raw_image_psyche_items/?order=date_received+desc&per_page=60&page=0&search=(A)%3Acamera
     */
    fn make_instrument_map(&self) -> InstrumentMap {
        InstrumentMap {
            map: [("A", vec!["A"]), ("B", vec!["B"])]
                .iter()
                .cloned()
                .collect(),
        }
    }
}
