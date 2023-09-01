use crate::constants;
use crate::enums::Mission;
use crate::httpfetch;
use crate::m20::fetch::M20Fetch;
use crate::metadata::Metadata;
use crate::msl::fetch::MslFetch;
use crate::nsyt::fetch::NsytFetch;
use crate::util::{save_image_json, InstrumentMap};
use anyhow::Result;
use async_trait::async_trait;
use cli_table::{Cell, Style, Table};
use rayon::prelude::*;
use sciimg::path;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{error::Error, fmt};

/// Generic all-mission remote raw image query parameters
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

/// Generic all-mission api stats from query results
#[derive(Debug, Clone)]
pub struct RemoteStats {
    pub more: bool,
    pub error_message: String,
    pub total_results: i32,
    pub page: i32,
    pub total_images: i32,
}

/// Defines the required fields checking for the latest image stats
pub trait LatestData {
    fn latest(&self) -> String;
    fn latest_sol(&self) -> u16;
    fn latest_sols(&self) -> Vec<u16>;
    fn new_count(&self) -> u16;
    fn sol_count(&self) -> u16;
    fn total(&self) -> u32;
}

pub type FetchType = Box<dyn Fetch + 'static + Sync + Send>;

pub type ReturnsFetch = dyn Fn() -> FetchType;

#[derive(Debug, Eq, PartialEq)]
pub enum FetchError {
    RemoteError(String),
    FileExists,
    WriteError,
    ProgrammingError(String),
    MissionNotSupportedError(Mission),
    SkippingFile,
    ParseError(String),
}

impl Error for FetchError {}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error of type {:?}", self)
    }
}

pub fn image_exists_on_filesystem(image_url: &str, output_path: Option<&str>) -> bool {
    let write_to = match output_path {
        Some(p) => {
            let bn = path::basename(image_url);
            format!("{}/{}", p, bn)
        }
        None => String::from(image_url),
    };
    path::file_exists(&write_to)
}

pub async fn fetch_image(
    image_url: &str,
    output_path: Option<&str>,
) -> Result<PathBuf, FetchError> {
    let write_to = match output_path {
        Some(p) => {
            let bn = path::basename(image_url);
            format!("{}/{}", p, bn)
        }
        None => String::from(image_url),
    };

    // would rather do this as if !... but I'm assuming these vprintln! calls are.. impotant for some reason...
    if let Ok(image_data) = httpfetch::simple_fetch_bin(image_url).await {
        let path = Path::new(write_to.as_str());
        info!("Writing image data to {}", write_to);

        let mut file = match File::create(path) {
            Ok(f) => f,
            Err(_) => return Err(FetchError::WriteError),
        };
        match file.write_all(&image_data[..]) {
            Ok(_) => Ok(path.to_path_buf()),
            Err(_) => Err(FetchError::WriteError),
        }
    } else {
        Err(FetchError::WriteError)
    }
}

/// Defines the required methods needed to implement a mission-specific remote raw image query service client
#[async_trait]
pub trait Fetch {
    /// Query the remote image service with the supplied parameters
    async fn query_remote_images(&self, query: &RemoteQuery) -> Result<Vec<Metadata>, FetchError>;

    /// Query the remote image service for information regarding images tagged as 'latest'
    /// 'Latest images' are generally those images to have come down in the most recent downlink. This may
    /// include any number of sols depending on what images were still onboard the rover at the time
    /// of the downlink.
    async fn fetch_latest(&self) -> Result<Box<dyn LatestData>, FetchError>;

    /// Query the remote image service and return only the stats portion of the results
    async fn fetch_stats(&self, query: &RemoteQuery) -> Result<RemoteStats, FetchError>;

    /// Return a mission-specific map of supported instruments. Each bottom-level string should match
    /// a supported instrument string on the remote api
    fn make_instrument_map(&self) -> InstrumentMap;
}

/// Create a new remote client specific to the provided mission.
///
/// Note: this mod shouldn't know about m20/msl/nsyt/etc. Look into an auto-registration that
/// is done from each mission code.
pub fn get_fetcher_for_mission(mission: Mission) -> Result<FetchType, FetchError> {
    match mission {
        Mission::Mars2020 => Ok(M20Fetch::new_boxed()),
        Mission::MSL => Ok(MslFetch::new_boxed()),
        Mission::InSight => Ok(NsytFetch::new_boxed()),
        _ => Err(FetchError::MissionNotSupportedError(mission)),
    }
}

macro_rules! nulltostr {
    ($o: expr) => {
        match &$o {
            None => String::from(""),
            Some(v) => {
                format!("{}", v)
            }
        }
    };
}

fn print_table(images: &[Metadata], query: &RemoteQuery) {
    let table = images
        .iter()
        .map(|md| {
            let image_destination_path = format!(
                "{}/{}",
                query.output_path,
                path::basename(md.remote_image_url.as_str())
            );

            vec![
                md.imageid.clone().cell(),
                md.instrument.clone().cell(),
                md.sol.cell(),
                md.date_taken_utc.clone().cell(),
                nulltostr!(md.date_taken_mars).cell(),
                nulltostr!(md.site).cell(),
                nulltostr!(md.drive).cell(),
                if md.thumbnail {
                    constants::status::YES
                } else {
                    constants::status::NO
                }
                .cell(),
                if path::file_exists(&image_destination_path) {
                    constants::status::YES
                } else {
                    constants::status::NO
                }
                .cell(),
            ]
        })
        .collect::<Vec<_>>()
        .table()
        .title(vec![
            "ID".cell().bold(true),
            "Instrument".cell().bold(true),
            "Sol".cell().bold(true),
            "Image Date (UTC)".cell().bold(true),
            "Image Date (Mars)".cell().bold(true),
            "Site".cell().bold(true),
            "Drive".cell().bold(true),
            "Thumb".cell().bold(true),
            "Present".cell().bold(true),
        ]);

    println!("{}", &table.display().unwrap());
}

async fn download_remote_image(
    image_md: &Metadata,
    query: &RemoteQuery,
    on_image_downloaded: OnImageDownloaded,
) -> Result<String, FetchError> {
    match fetch_image(&image_md.remote_image_url, Some(query.output_path.as_ref())).await {
        Ok(_) => {
            let image_base_name = path::basename(image_md.remote_image_url.as_str());
            _ = save_image_json(&image_base_name, image_md, Some(query.output_path.as_ref()));
            on_image_downloaded(image_md);
            Ok(image_base_name)
        }
        Err(why) => Err(why),
    }
}

/// Callback to inform the caller as to the total number of images that will be downloaded
type OnTotalKnown = fn(usize);

// Callback to inform the caller that an image has been downloaded
type OnImageDownloaded = fn(&Metadata);

pub async fn perform_fetch(
    mission: Mission,
    query: &RemoteQuery,
    on_total_known: OnTotalKnown,
    on_image_downloaded: OnImageDownloaded,
) -> Result<(), FetchError> {
    if let Ok(client) = get_fetcher_for_mission(mission) {
        match client.query_remote_images(query).await {
            Ok(results) => {
                // print a table of all the results.
                print_table(&results, query);

                // Iterate over the results and remove existing images
                // if the user has selected to skip any images that already exist locally
                let to_download: Vec<Metadata> = results
                    .into_iter()
                    .filter(|_| !query.list_only)
                    .filter(|md| {
                        !query.only_new
                            || !image_exists_on_filesystem(
                                &md.remote_image_url,
                                Some(&query.output_path),
                            )
                    })
                    .collect();

                // Don't bother with the result if we have nothing to download
                if !to_download.is_empty() {
                    on_total_known(to_download.len());

                    let tasks: Vec<_> = to_download
                        .par_iter()
                        .map(|md| {
                            info!("Fetching Image from Remote URL: {}", md.remote_image_url);
                            download_remote_image(md, query, on_image_downloaded)
                        })
                        .collect();
                    for task in tasks {
                        task.await?;
                    }
                }
            }
            Err(why) => error!("Error: {}", why),
        };

        Ok(())
    } else {
        Err(FetchError::MissionNotSupportedError(mission))
    }
}

pub async fn get_latest(mission: Mission) -> Result<Box<dyn LatestData>, FetchError> {
    if let Ok(client) = get_fetcher_for_mission(mission) {
        client.fetch_latest().await
    } else {
        Err(FetchError::MissionNotSupportedError(mission))
    }
}

pub fn get_instrument_map(mission: Mission) -> Result<InstrumentMap, FetchError> {
    if let Ok(client) = get_fetcher_for_mission(mission) {
        Ok(client.make_instrument_map())
    } else {
        Err(FetchError::MissionNotSupportedError(mission))
    }
}
