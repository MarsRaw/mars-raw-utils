use mars_raw_utils::calibfile;

#[test]
fn test_load_caldata_mapping_file() {
    calibfile::locate_calibration_file(&String::from("caldata.toml")).expect("Failed to locate caldata.toml");
    let config = calibfile::load_caldata_mapping_file().unwrap();
    assert_eq!(
        config.msl.mahli.inpaint_mask,
        "MSL_MAHLI_INPAINT_Sol2904_V1.png"
    );
}

#[test]
fn test_locate_without_extention() {
    calibfile::locate_calibration_file_no_extention(
        &"caldata".to_string(),
        &".toml".to_string(),
    )
    .expect("Failed to locate caldata.toml");
}
