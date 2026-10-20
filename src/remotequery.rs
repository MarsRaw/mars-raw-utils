use crate::enums::Mission;
use crate::httpfetch;
use crate::metadata::{Metadata, MissionMetadata};
use crate::util::{save_image_json, InstrumentMap};
use crate::{constants, util};
use anyhow::Result;
use chrono::{DateTime, Utc};
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
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub filter_num: Option<Vec<u32>>,
    pub filter: Option<Vec<String>>,
    pub filter_wavelenth: Option<Vec<String>>,
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
    WriteError(String),
    ProgrammingError(String),
    MissionNotSupportedError(Mission),
    SkippingFile,
    ParseError(String),
    NotImplemented,
}

impl Error for FetchError {}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error of type {:?}", self)
    }
}

pub fn image_exists_on_filesystem(image_file_name: &str, output_path: Option<&str>) -> bool {
    let write_to = match output_path {
        Some(p) => {
            let bn = image_file_name;
            format!("{}/{}", p, bn)
        }
        None => String::from(image_file_name),
    };
    path::file_exists(&write_to)
}

pub async fn fetch_image(
    image_url: &str,
    output_file_name: &str,
    output_path: Option<&str>,
) -> Result<PathBuf, FetchError> {
    let write_to = match output_path {
        Some(p) => {
            format!("{}/{}", p, output_file_name)
        }
        None => String::from(image_url),
    };

    let full_url = if util::url_has_full_hostname(image_url) {
        image_url.to_string()
    } else {
        format!("{}{}", constants::url::NASA_RAW_HOST_URL, image_url)
    };

    let image_data = match httpfetch::simple_fetch_bin(&full_url).await {
        Ok(image_data) => image_data,
        Err(why) => return Err(FetchError::WriteError(format!("Write Error: {:?}", why))),
    };

    let path = Path::new(write_to.as_str());
    info!("Writing image data to {}", write_to);

    let mut file = match File::create(path) {
        Ok(f) => f,
        Err(why) => return Err(FetchError::WriteError(format!("WriteError: {:?}", why))),
    };
    match file.write_all(&image_data[..]) {
        Ok(_) => Ok(path.to_path_buf()),
        Err(why) => Err(FetchError::WriteError(format!("WriteError: {:?}", why))),
    }
}

/// Defines the required methods needed to implement a mission-specific remote raw image query service client
#[allow(async_fn_in_trait)]
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

fn print_table_rover(images: &[Metadata], query: &RemoteQuery) {
    let table = images
        .iter()
        .map(|md| {
            let mission_metadata = match md.as_rover() {
                Some(md) => md,
                None => panic!("Received non-surface metadata. Expected surface"),
            };

            let image_destination_path =
                format!("{}/{}", query.output_path, md.image.derived_filename);

            vec![
                md.image.imageid.clone().cell(),
                mission_metadata.instrument.clone().cell(),
                mission_metadata.sol.cell(),
                md.image.date_taken_utc.clone().cell(),
                nulltostr!(mission_metadata.date_taken_mars).cell(),
                nulltostr!(mission_metadata.site).cell(),
                nulltostr!(mission_metadata.drive).cell(),
                if mission_metadata.thumbnail {
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

    println!("{}", table.display().unwrap());
}

fn print_table_psyche(images: &[Metadata], query: &RemoteQuery) {
    let table = images
        .iter()
        .map(|md| {
            let mission_metadata = match md.as_psyche() {
                Some(md) => md,
                None => panic!("Received surface metadata. Expected non-surface"),
            };

            let image_destination_path =
                format!("{}/{}.png", query.output_path, md.image.derived_filename);

            vec![
                md.image.imageid.clone().cell(),
                mission_metadata.camera_name.clone().cell(),
                mission_metadata.filter_name.clone().cell(),
                mission_metadata.filter_wavelength.clone().cell(),
                md.image.date_taken_utc.cell(),
                nulltostr!(mission_metadata.orbit_number).cell(),
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
            "Filter".cell().bold(true),
            "Wavelength".cell().bold(true),
            "Image Date (UTC)".cell().bold(true),
            "Orbit".cell().bold(true),
            "Present".cell().bold(true),
        ]);

    println!("{}", table.display().unwrap());
}

fn print_table(images: &[Metadata], query: &RemoteQuery) {
    if !images.is_empty() {
        match images.first().unwrap().mission {
            MissionMetadata::Rover(_) => print_table_rover(images, query),
            MissionMetadata::Psyche(_) => print_table_psyche(images, query),
        }
    }
}

async fn download_remote_image(
    image_md: &Metadata,
    query: &RemoteQuery,
    on_image_downloaded: OnImageDownloaded,
) -> Result<String, FetchError> {
    match fetch_image(
        &image_md.image.remote_image_url,
        &image_md.image.derived_filename,
        Some(query.output_path.as_ref()),
    )
    .await
    {
        Ok(_) => {
            _ = save_image_json(
                &image_md.image.derived_filename,
                image_md,
                Some(query.output_path.as_ref()),
            );
            on_image_downloaded(image_md);
            Ok(image_md.image.derived_filename.clone())
        }
        Err(why) => Err(why),
    }
}

/// Callback to inform the caller as to the total number of images that will be downloaded
type OnTotalKnown = fn(usize);

// Callback to inform the caller that an image has been downloaded
type OnImageDownloaded = fn(&Metadata);

pub async fn fetch_available<T: Fetch>(
    client: &T,
    query: &RemoteQuery,
) -> Result<Vec<Metadata>, FetchError> {
    client.query_remote_images(query).await
}

pub async fn perform_fetch<T: Fetch>(
    client: &T,
    query: &RemoteQuery,
    on_total_known: OnTotalKnown,
    on_image_downloaded: OnImageDownloaded,
    max_results: Option<usize>,
) -> Result<(), FetchError> {
    let results = match fetch_available(client, query).await {
        Ok(results) => results,
        Err(why) => return Err(why),
    };

    // Limit to the requested maximum number of results
    let mut trimmed_results = results.clone();
    if let Some(max_results) = max_results {
        trimmed_results.truncate(max_results);
    }

    // print a table of all the results.
    print_table(&results, query);

    // Iterate over the results and remove existing images
    // if the user has selected to skip any images that already exist locally
    let to_download: Vec<Metadata> = trimmed_results
        .into_iter()
        .filter(|_| !query.list_only)
        .filter(|md| {
            !query.only_new
                || !image_exists_on_filesystem(&md.image.derived_filename, Some(&query.output_path))
        })
        .collect();

    // Don't bother with the result if we have nothing to download
    if !to_download.is_empty() {
        on_total_known(to_download.len());

        let tasks: Vec<_> = to_download
            .par_iter()
            .map(|md| {
                info!(
                    "Fetching Image from Remote URL: {}",
                    md.image.remote_image_url
                );
                download_remote_image(md, query, on_image_downloaded)
            })
            .collect();

        for task in tasks {
            match task.await {
                Ok(_) => (),
                Err(why) => {
                    return Err(FetchError::RemoteError(f!(
                        "Remote error fetching image: {:?}",
                        why
                    )))
                }
            };
        }
    }
    Ok(())
}

pub async fn get_latest<T: Fetch>(client: &T) -> Result<Box<dyn LatestData>, FetchError> {
    client.fetch_latest().await
}

pub fn get_instrument_map<T: Fetch>(client: &T) -> Result<InstrumentMap, FetchError> {
    Ok(client.make_instrument_map())
}
