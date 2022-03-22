use mars_raw_utils::{
    calibfile
};



#[test]
#[ignore]
fn test_load_caldata_mapping_file() {
    let caldata_toml = calibfile::locate_calibration_file(&String::from("caldata.toml")).unwrap();
    assert_eq!(caldata_toml, "mars-raw-utils-data/caldata/caldata.toml");

    let config = calibfile::load_caldata_mapping_file().unwrap();
    assert_eq!(config.msl.mahli.inpaint_mask, "MSL_MAHLI_INPAINT_Sol2904_V1.png");
}

#[test]
#[ignore]
fn test_locate_without_extention() {
    let caldata_toml = calibfile::locate_calibration_file_no_extention(&"caldata".to_string(), &".toml".to_string()).unwrap();
    assert_eq!(caldata_toml, "mars-raw-utils-data/caldata/caldata.toml");
}
