use mars_raw_utils::caldata;
use std::env;

const CALIBRATION_FILE_REMOTE_ROOT: &str =
    "https://raw.githubusercontent.com/kmgill/mars-raw-utils-data/main/caldata/";

#[test]
fn test_get_calibration_file_remote_root_default_value() {
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    assert_eq!(
        caldata::get_calibration_file_remote_root(),
        CALIBRATION_FILE_REMOTE_ROOT
    );
}

#[test]
fn test_get_calibration_file_remote_root_env_value() {
    env::set_var("CALIBRATION_FILE_REMOTE_ROOT", "/foo/bar");
    assert_eq!(caldata::get_calibration_file_remote_root(), "/foo/bar");
}

#[test]
fn test_get_calibration_file_remote_url_default_value() {
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    let foo = format!("{}/foo.toml", CALIBRATION_FILE_REMOTE_ROOT);
    assert_eq!(caldata::get_calibration_file_remote_url("foo.toml"), foo)
}

#[test]
fn test_get_calibration_file_remote_url_env_value() {
    env::set_var("CALIBRATION_FILE_REMOTE_ROOT", "/foo/bar");
    assert_eq!(
        caldata::get_calibration_file_remote_url("foo.toml"),
        "/foo/bar/foo.toml"
    )
}

#[tokio::test]
async fn test_fetch_remote_calibration_manifest() {
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    assert!(caldata::fetch_remote_calibration_manifest().await.is_ok())
}

#[tokio::test]
async fn test_fetch_remote_calibration_manifest_from() {
    let foo = format!("{}/caldata.toml", CALIBRATION_FILE_REMOTE_ROOT);
    assert!(caldata::fetch_remote_calibration_manifest_from(&foo)
        .await
        .is_ok());
}
