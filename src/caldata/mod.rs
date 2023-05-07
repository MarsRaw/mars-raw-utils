use crate::calibfile::{
    parse_caldata_from_string, Config, InstrumentProperties, M20CalData, MslCalData, NsytCalData,
};
use crate::httpfetch;
use anyhow::Result;
use dirs;
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
pub fn get_calibration_local_store() -> PathBuf {
    if let Ok(dir) = env::var("MARS_RAW_DATA") {
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
pub fn get_calibration_file_local_path(remote_file_uri: &str) -> PathBuf {
    get_calibration_local_store().join(PathBuf::from(remote_file_uri))
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
pub async fn fetch_remote_calibration_manifest_from(uri: &str) -> Result<Config, &'static str> {
    if let Ok(data) = httpfetch::simple_fetch_text(uri).await {
        parse_caldata_from_string(&data)
    } else {
        Err("Failed to retrieve remote resource")
    }
}

/// Fetches the remote calibration manifest and returns the parsed `calibfile::Config` struct.
/// This will use the system defined manifest url
pub async fn fetch_remote_calibration_manifest() -> Result<Config, &'static str> {
    fetch_remote_calibration_manifest_from(&get_calibration_file_remote_url("caldata.toml")).await
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
) -> Result<SaveResult, String> {
    let save_to = get_calibration_file_local_path(remote_file_uri);
    let save_to_exists = save_to.exists();
    if save_to_exists && !replace {
        println!(
            "Calibraton file {} already exists and replace is set to false",
            remote_file_uri
        );
        Ok(SaveResult::NotReplaced)
    } else {
        println!("Fetching {}", remote_file_uri);
        let calibration_local_store = get_calibration_local_store();
        if !calibration_local_store.exists() {
            fs::create_dir_all(&calibration_local_store).unwrap();
        }

        let resource_url = get_calibration_file_remote_url(remote_file_uri);

        match httpfetch::simple_fetch_bin(&resource_url).await {
            Ok(bytes_array) => {
                println!("Saving to {:?}", save_to);
                let mut file = File::create(save_to).unwrap();
                file.write_all(&bytes_array[..]).unwrap();
                if save_to_exists {
                    Ok(SaveResult::Replaced)
                } else {
                    Ok(SaveResult::IsNew)
                }
            }
            Err(why) => {
                println!("Error fetching {}: {}", remote_file_uri, why);
                Err(format!("{:?}", why))
            }
        }
    }
}

pub async fn fetch_and_save_files_for_instrument(
    inst_props: &InstrumentProperties,
    replace: bool,
) -> Result<(), String> {
    for inst_prop in inst_props.clone().into_iter() {
        if !inst_prop.file.is_empty()
            && fetch_and_save_file(&inst_prop.file, replace).await.is_err()
        {
            return Err(format!("Failed to download remote file {}", inst_prop.file));
        }
    }

    Ok(())
}

pub async fn update_mission_msl(mission_config: &MslCalData, replace: bool) -> Result<(), String> {
    for f in [
        &mission_config.chemcam,
        &mission_config.fhaz_left,
        &mission_config.fhaz_right,
        &mission_config.mahli,
        &mission_config.mardi,
        &mission_config.mastcam_left,
        &mission_config.mastcam_right,
        &mission_config.nav_left,
        &mission_config.nav_right,
        &mission_config.rhaz_left,
        &mission_config.rhaz_right,
    ]
    .into_iter()
    {
        fetch_and_save_files_for_instrument(f, replace).await?;
    }
    Ok(())
}

pub async fn update_mission_m20(mission_config: &M20CalData, replace: bool) -> Result<(), String> {
    for f in [
        &mission_config.cachecam,
        &mission_config.edl_rdcam,
        &mission_config.fhaz_left,
        &mission_config.fhaz_right,
        &mission_config.heli_nav,
        &mission_config.heli_rte,
        &mission_config.mastcamz_left,
        &mission_config.mastcamz_right,
        &mission_config.nav_left,
        &mission_config.nav_right,
        &mission_config.pixl_mcc,
        &mission_config.rhaz_left,
        &mission_config.rhaz_right,
        &mission_config.sherloc_aci,
        &mission_config.skycam,
        &mission_config.supercam_rmi,
        &mission_config.watson,
    ]
    .into_iter()
    {
        fetch_and_save_files_for_instrument(f, replace).await?;
    }

    for v in [0, 2448, 3834, 5196, 6720, 8652, 9600].into_iter() {
        let motor_stop_str = format!("{:04}", v);

        for inst in [
            &mission_config.mastcamz_left,
            &mission_config.mastcamz_right,
        ]
        .into_iter()
        {
            let file_path = inst.flat.replace("-motorcount-", motor_stop_str.as_str());
            fetch_and_save_file(&file_path, replace).await?;
        }
    }

    for sf in [1, 2, 4].into_iter() {
        for inst in [
            &mission_config.nav_left,
            &mission_config.nav_right,
            &mission_config.fhaz_left,
            &mission_config.fhaz_right,
            &mission_config.rhaz_left,
            &mission_config.rhaz_right,
        ]
        .into_iter()
        {
            // Iterate navcam and hazcam scale factors
            let sf_s = format!("sf{}", sf);
            for f in [&inst.mask, &inst.flat].into_iter() {
                if f.is_empty() {
                    continue;
                }
                let file_path = f.replace("-scalefactor-", &sf_s);
                fetch_and_save_file(&file_path, replace).await?;
            }
        }
    }

    Ok(())
}

pub async fn update_mission_nsyt(
    mission_config: &NsytCalData,
    replace: bool,
) -> Result<(), String> {
    for f in [&mission_config.icc, &mission_config.idc].into_iter() {
        fetch_and_save_files_for_instrument(f, replace).await?;
    }
    Ok(())
}

/// Retrieves the remote calibration file manifest `caldata.toml` and downloads each
/// referenced file. If `replace` is false, existing files will not be overwritten.
pub async fn update_calibration_data(replace: bool) -> Result<(), &'static str> {
    let manifest_config_res = fetch_remote_calibration_manifest().await;

    if let Ok(config) = manifest_config_res {
        // I really don't like the design of this....

        assert!(fetch_and_save_file("caldata.toml", replace).await.is_ok());
        assert!(update_mission_msl(&config.msl, replace).await.is_ok());
        assert!(update_mission_m20(&config.m20, replace).await.is_ok());
        assert!(update_mission_nsyt(&config.nsyt, replace).await.is_ok());

        Ok(())
    } else {
        Err("Failed to retrieve remote data manifest")
    }
}
