use crate::calibfile::{parse_caldata_from_string, Config};
use crate::httpfetch;
use dirs;
use sciimg::error;
use std::env;
use std::fs;
use std::path::PathBuf;
use url::Url;

// TODO: I would prefer this not being hardcoded. Find how to define it in Cargo.toml
// which would then populate this const at compile time.
//
// Allow overriding via environment variable &CALIBRATION_FILE_REMOTE_ROOT
const CALIBRATION_FILE_REMOTE_ROOT: &str =
    "https://raw.githubusercontent.com/kmgill/mars-raw-utils-data/main/caldata/";

/// Determine where to put the calibration data files. A path
/// indicated by the environment variable `MARS_RAW_DATA` would be used first
/// followed by `$HOME/.marsdata`. It is assumed that `/usr/share/mars_raw_utils/data/`
/// or `/Program Files/mars_raw_utils/data` would be unwritable thus not considered
/// by this function. This function also does not attempt to determine if the
/// returned path exists or is writable.
pub fn get_calibration_local_store() -> PathBuf {
    if let Ok(dir) = env::var("MARS_RAW_DATA") {
        PathBuf::from(&dir)
    } else if let Some(dir) = dirs::home_dir() {
        PathBuf::from(format!("{}/.marsdata", dir.to_str().unwrap()))
    } else {
        panic!("Unable to determine where to put calibration data!");
    }
}

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
pub fn get_calibration_file_remote_url(remote_file_uri: &str) -> String {
    let base = Url::parse(&get_calibration_file_remote_root())
        .expect("hardcoded URL is known to be valid");
    let joined = base
        .join(remote_file_uri)
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

pub async fn fetch_and_save_file(remote_file_uri: &str, _replace: bool) -> error::Result<()> {
    let save_to = get_calibration_local_store();
    if !save_to.exists() && save_to.parent().unwrap().exists() {
        fs::create_dir_all(save_to).unwrap();
    }

    let resource_url = get_calibration_file_remote_url(remote_file_uri);

    let _cal_file_bytes_result = httpfetch::simple_fetch_bin(&resource_url).await;

    //Ok(())
    panic!("Not yet implemented");
}

pub async fn update_calibration_data(_replace: bool) -> error::Result<()> {
    let manifest_config_res = fetch_remote_calibration_manifest().await;

    if let Ok(_config) = manifest_config_res {
        Ok(())
    } else {
        Err("Failed to retrieve remote data manifest")
    }
}
