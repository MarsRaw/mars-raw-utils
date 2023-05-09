use crate::httpfetch;
use crate::print;
use crate::vprintln;
use anyhow::Result;
use dirs;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
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
pub fn get_calibration_local_store(use_local_store: &Option<String>) -> PathBuf {
    if let Some(local_store) = use_local_store {
        PathBuf::from(&local_store)
    } else if let Ok(dir) = env::var("MARS_RAW_DATA") {
        PathBuf::from(&dir)
    } else if let Some(dir) = dirs::home_dir() {
        PathBuf::from(format!("{}/.marsdata", dir.to_str().unwrap()))
    } else {
        panic!("Unable to determine where to put calibration data!");
    }
}

/// Returns the expected location on disk (local) for a specific
/// calibration file by joining the result from `get_calibration_local_store()`
/// with the `remote_file_uri` parameter.
pub fn get_calibration_file_local_path(
    remote_file_uri: &str,
    use_local_store: &Option<String>,
) -> PathBuf {
    get_calibration_local_store(use_local_store).join(PathBuf::from(remote_file_uri))
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

/// Splits the file list into a vector of Strings
pub fn parse_manifest_file_list(data: &str) -> Vec<String> {
    data.lines().map(|f| f.to_string()).collect()
}

/// Fetches the remote calibration manifest from a specified url and returns the parsed file list
pub async fn fetch_remote_calibration_manifest_from(
    uri: &str,
) -> Result<Vec<String>, &'static str> {
    if let Ok(data) = httpfetch::simple_fetch_text(uri).await {
        Ok(parse_manifest_file_list(&data))
    } else {
        Err("Failed to retrieve remote resource")
    }
}

/// Fetches the remote calibration manifest and returns the parsed file list.
/// This will use the system defined manifest url
pub async fn fetch_remote_calibration_manifest() -> Result<Vec<String>, &'static str> {
    fetch_remote_calibration_manifest_from(&get_calibration_file_remote_url("caldata.manifest"))
        .await
}

/// Success states for `fetch_and_save_file`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SaveResult {
    NotReplaced,
    Replaced,
    IsNew,
}

/// Fetch the remote file and save it to disk in a location indicated by `get_calibration_file_local_path`.
/// If `replace` is false and the file already exists, this function will quit and not overwrite.
pub async fn fetch_and_save_file(
    remote_file_uri: &str,
    replace: bool,
    use_local_store: &Option<String>,
) -> Result<SaveResult, String> {
    let save_to = get_calibration_file_local_path(remote_file_uri, use_local_store);
    let save_to_exists = save_to.exists();
    if save_to_exists && !replace {
        print::print_warn(&format!("Skipped: {}", remote_file_uri));
        vprintln!(
            "Calibraton file {} already exists and replace is set to false",
            remote_file_uri
        );
        Ok(SaveResult::NotReplaced)
    } else {
        vprintln!("Fetching {}", remote_file_uri);
        let calibration_local_store = get_calibration_local_store(use_local_store);
        if !calibration_local_store.exists() {
            fs::create_dir_all(&calibration_local_store).unwrap();
        }

        let resource_url = get_calibration_file_remote_url(remote_file_uri);

        match httpfetch::simple_fetch_bin(&resource_url).await {
            Ok(bytes_array) => {
                vprintln!("Saving to {:?}", save_to);

                if let Some(parent) = save_to.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).unwrap();
                    }
                }

                let mut file = File::create(save_to).unwrap();
                file.write_all(&bytes_array[..]).unwrap();
                if save_to_exists {
                    print::print_done(&format!("Replaced: {}", remote_file_uri));
                    Ok(SaveResult::Replaced)
                } else {
                    print::print_done(&format!("New File: {}", remote_file_uri));
                    Ok(SaveResult::IsNew)
                }
            }
            Err(why) => {
                println!("Error fetching {}: {}", remote_file_uri, why);
                print::print_fail(&format!("Failed: {}", remote_file_uri));
                Err(format!("{:?}", why))
            }
        }
    }
}

/// Retrieves the remote calibration file manifest `caldata.manifest` and downloads each
/// referenced file. If `replace` is false, existing files will not be overwritten.
pub async fn update_calibration_data(
    replace: bool,
    use_local_store: &Option<String>,
) -> Result<(), String> {
    let manifest_config_res = fetch_remote_calibration_manifest().await;

    if let Ok(file_list) = manifest_config_res {
        let tasks: Vec<_> = file_list
            .par_iter()
            .map(|f| fetch_and_save_file(f, replace, use_local_store))
            .collect();
        for task in tasks {
            task.await?;
        }

        Ok(())
    } else {
        Err("Failed to retrieve remote data manifest".to_string())
    }
}
