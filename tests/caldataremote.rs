use mars_raw_utils::{caldata, httpfetch};
use sciimg::image::Image;
use sciimg::path;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

const CALIBRATION_FILE_REMOTE_ROOT: &str =
    "https://raw.githubusercontent.com/kmgill/mars-raw-utils-data/main/caldata/";

#[test]
fn test_get_calibration_remote_root_functions() {
    // Default root
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    assert_eq!(
        caldata::get_calibration_file_remote_root(),
        CALIBRATION_FILE_REMOTE_ROOT
    );

    // ENV value as root
    env::set_var("CALIBRATION_FILE_REMOTE_ROOT", "http://foo.com/bar");
    assert_eq!(
        caldata::get_calibration_file_remote_root(),
        "http://foo.com/bar"
    );

    // Default root
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    let toml = format!("{}foo.toml", CALIBRATION_FILE_REMOTE_ROOT);
    assert_eq!(caldata::get_calibration_file_remote_url("foo.toml"), toml);

    // ENV value as root
    env::set_var("CALIBRATION_FILE_REMOTE_ROOT", "http://foo.com/bar/");
    assert_eq!(
        caldata::get_calibration_file_remote_url("foo.toml"),
        "http://foo.com/bar/foo.toml"
    )
}

#[tokio::test]
async fn test_fetch_remote_calibration_manifest() {
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    assert!(caldata::fetch_remote_calibration_manifest().await.is_ok())
}

#[tokio::test]
async fn test_fetch_remote_calibration_manifest_from() {
    let toml = format!("{}/caldata.manifest", CALIBRATION_FILE_REMOTE_ROOT);
    assert!(caldata::fetch_remote_calibration_manifest_from(&toml)
        .await
        .is_ok());
}

#[tokio::test]
async fn test_fetch_remote_calibration_resource() {
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    if let Ok(c) = caldata::fetch_remote_calibration_manifest().await {
        // The file list should not be zero length.
        assert!(!c.is_empty());

        env::remove_var("CALIBRATION_FILE_REMOTE_ROOT"); // Another call in case tests are run concurrently and the env
                                                         // var gets set since the last remove_var was called.
        let remote_url = caldata::get_calibration_file_remote_url(&c[0]);
        let remote_url_expected = format!("{}{}", CALIBRATION_FILE_REMOTE_ROOT, c[0]);

        // Remote URL should be what we expect
        assert_eq!(remote_url, remote_url_expected, "Unexpected remote URL");

        // Fetch the resource into an array of u8 (bytes)
        let cal_file_bytes_result = httpfetch::simple_fetch_bin(&remote_url).await;
        assert!(cal_file_bytes_result.is_ok());
        let cal_file_bytes = cal_file_bytes_result.unwrap();

        // Create a temporary file, write those bytes into it then try to open
        // the resulting image
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join(&c[0]);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(&cal_file_bytes[..]).unwrap();

        Image::open(file_path.as_os_str().to_str().unwrap())
            .expect("Failed to properly read calibration file as image");

        // Clean up the temp file
        drop(file);
        temp_dir.close().unwrap();
    } else {
        panic!("Could not retrieve remote manifest");
    }
}

#[tokio::test]
async fn test_fetch_and_save_file() {
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");

    // Delete this file
    let local_file = "tests/testdata/caltesting/m20/ilut/M20_LUT2_v2a.txt";
    println!("Local file: {}", local_file);
    if path::file_exists(local_file) {
        assert!(fs::remove_file(local_file).is_ok());
    }

    // Ask to download the file. It should not exist (because we just deleted it) and setting
    // 'replace' to false shouldn't matter. Result should be that it consideres it a new file.
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    let res = caldata::fetch_and_save_file(
        "m20/ilut/M20_LUT2_v2a.txt",
        false,
        &Some("tests/testdata/caltesting".to_string()),
    )
    .await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), caldata::SaveResult::IsNew);

    // Ask to download the file. The file exists (becuase we just downloaded it in the previous step) and
    // with 'replace' set to false, the result should be that it was not replaced
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    let res = caldata::fetch_and_save_file(
        "m20/ilut/M20_LUT2_v2a.txt",
        false,
        &Some("tests/testdata/caltesting".to_string()),
    )
    .await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), caldata::SaveResult::NotReplaced);

    // Ask to download the file. The file exists (because we downloaded it two steps ago) and with
    // 'replace' set to true, the result should be that it was replaced.
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");
    let res = caldata::fetch_and_save_file(
        "m20/ilut/M20_LUT2_v2a.txt",
        true,
        &Some("tests/testdata/caltesting".to_string()),
    )
    .await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), caldata::SaveResult::Replaced);
}

#[tokio::test]
async fn test_update_calibration_data() {
    env::remove_var("CALIBRATION_FILE_REMOTE_ROOT");

    // Perform a full data download
    assert!(caldata::update_calibration_data(
        false,
        &Some("tests/testdata/caltesting".to_string()),
        |_| {},
        || {}
    )
    .await
    .is_ok());
}
