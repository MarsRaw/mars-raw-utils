use mars_raw_utils::{
    calibfile
};



#[test]
fn test_load_caldata_mapping_file() {

    let caldata_toml = calibfile::locate_calibration_file("caldata.toml").unwrap();
    assert_eq!(caldata_toml, "mars-raw-utils-data/caldata/caldata.toml");

    let config = calibfile::load_caldata_mapping_file().unwrap();
    assert_eq!(config.msl.MSL_MAHLI_INPAINT_MASK_PATH, "MSL_MAHLI_INPAINT_Sol2904_V1.png");
}

