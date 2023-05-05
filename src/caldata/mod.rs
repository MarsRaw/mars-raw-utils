use crate::calibfile::{parse_caldata_from_string, Config};
use crate::httpfetch;
use sciimg::error;
use std::env;
use url::Url;

// TODO: I would prefer this not being hardcoded. Find how to define it in Cargo.toml
// which would then populate this const at compile time.
//
// Allow overriding via environment variable &CALIBRATION_FILE_REMOTE_ROOT
const CALIBRATION_FILE_REMOTE_ROOT: &str =
    "https://raw.githubusercontent.com/kmgill/mars-raw-utils-data/main/caldata/";

///  Returns the remote root URL for fetching the calibration manifest (caldata.toml)
///  along with the referenced data filess. Default URL can be overriden by setting
///  the environment variable `CALIBRATION_FILE_REMOTE_ROOT`.
pub fn get_calibration_file_remote_root() -> String {
    if let Ok(v) = env::var("CALIBRATION_FILE_REMOTE_ROOT") {
        v
    } else {
        CALIBRATION_FILE_REMOTE_ROOT.to_string()
    }
}

///  Appends the relative path to the system-defined remote calibration root URL
pub fn get_calibration_file_remote_url(file_relative_path: &str) -> String {
    let base = Url::parse(&get_calibration_file_remote_root())
        .expect("hardcoded URL is known to be valid");
    let joined = base
        .join(file_relative_path)
        .expect("Failed to combine URL segments");
    joined.to_string()
}

/// Fetches the remote calibration manifest from a specified url and returns the parsed `calibfile::Config` struct
pub async fn fetch_remote_calibration_manifest_from(uri: &str) -> error::Result<Config> {
    if let Ok(data) = httpfetch::simple_fetch_text(uri).await {
        parse_caldata_from_string(&data)
    } else {
        Err("Failed to retrieve remote resource")
    }
}

/// Fetches the remote calibration manifest and returns the parsed `calibfile::Config` struct.
/// This will use the system defined manifest url
pub async fn fetch_remote_calibration_manifest() -> error::Result<Config> {
    fetch_remote_calibration_manifest_from(&get_calibration_file_remote_url("caldata.toml")).await
}
