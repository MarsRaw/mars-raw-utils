use mars_raw_utils::util;

#[test]
fn test_filename_char_at_pos() {
    let test_string = "/data/MSL/NCAM_TEST/RLB_670398086EDR_F0871078RHAZ00311M_.jpg";
    assert_eq!(util::filename_char_at_pos(&test_string, 0), 'R');
    assert_eq!(util::filename_char_at_pos(&test_string, 1), 'L');
}

#[test]
fn test_string_is_valid_f32() {
    assert!(util::string_is_valid_f32("1.2"));
    assert!(util::string_is_valid_f32("1.0"));
    assert!(util::string_is_valid_f32("1"));
    assert!(util::string_is_valid_f32(".0"));
    assert!(util::string_is_valid_f32("0."));
    assert!(util::string_is_valid_f32("-0."));
    assert!(util::string_is_valid_f32(".45454"));
    assert!(util::string_is_valid_f32("fdsgfgdf") == false);
    assert!(util::string_is_valid_f32("1.04f") == false);
}

#[test]
fn test_string_is_valid_i32() {
    assert!(util::string_is_valid_i32("1"));
    assert!(util::string_is_valid_i32("0"));
    assert!(util::string_is_valid_i32("-1"));
    assert!(util::string_is_valid_i32("+1"));
    assert!(util::string_is_valid_i32("1l") == false);
    assert!(util::string_is_valid_i32("dfsuidhfiusdh") == false);
}

#[test]
fn test_is_name_a_remote_instrument() {
    let instrument_list = util::InstrumentMap {
        map: [
            (
                "HAZ_FRONT",
                vec!["FHAZ_RIGHT_A", "FHAZ_LEFT_A", "FHAZ_RIGHT_B", "FHAZ_LEFT_B"],
            ),
            (
                "HAZ_REAR",
                vec!["RHAZ_RIGHT_A", "RHAZ_LEFT_A", "RHAZ_RIGHT_B", "RHAZ_LEFT_B"],
            ),
            ("NAV_LEFT", vec!["NAV_LEFT_A", "NAV_LEFT_B"]),
            ("NAV_RIGHT", vec!["NAV_RIGHT_A", "NAV_RIGHT_B"]),
            ("CHEMCAM", vec!["CHEMCAM_RMI"]),
            ("MARDI", vec!["MARDI"]),
            ("MAHLI", vec!["MAHLI"]),
            ("MASTCAM", vec!["MAST_LEFT", "MAST_RIGHT"]),
        ]
        .iter()
        .cloned()
        .collect(),
    };

    assert_eq!(instrument_list.is_name_a_remote_instrument("MAHLI"), true);
    assert_eq!(
        instrument_list.is_name_a_remote_instrument("CHEMCAM"),
        false
    );
    assert_eq!(
        instrument_list.is_name_a_remote_instrument("CHEMCAM_RMI"),
        true
    );
}

#[test]
fn test_find_remote_instrument_names_fromlist() {
    let instrument_list = util::InstrumentMap {
        map: [
            (
                "HAZ_FRONT",
                vec!["FHAZ_RIGHT_A", "FHAZ_LEFT_A", "FHAZ_RIGHT_B", "FHAZ_LEFT_B"],
            ),
            (
                "HAZ_REAR",
                vec!["RHAZ_RIGHT_A", "RHAZ_LEFT_A", "RHAZ_RIGHT_B", "RHAZ_LEFT_B"],
            ),
            ("NAV_LEFT", vec!["NAV_LEFT_A", "NAV_LEFT_B"]),
            ("NAV_RIGHT", vec!["NAV_RIGHT_A", "NAV_RIGHT_B"]),
            ("CHEMCAM", vec!["CHEMCAM_RMI"]),
            ("MARDI", vec!["MARDI"]),
            ("MAHLI", vec!["MAHLI"]),
            ("MASTCAM", vec!["MAST_LEFT", "MAST_RIGHT"]),
        ]
        .iter()
        .cloned()
        .collect(),
    };

    let list: Vec<&str> = vec!["NAV_LEFT", "CHEMCAM"];

    let string_lis: Vec<String> = list.iter().map(|s| String::from(*s)).collect();
    let remote_instruments = instrument_list
        .find_remote_instrument_names_fromlist(&string_lis)
        .unwrap();

    let expected_list = vec!["NAV_LEFT_A", "NAV_LEFT_B", "CHEMCAM_RMI"];
    assert_eq!(remote_instruments, expected_list);
}

#[test]
#[should_panic]
fn test_find_remote_instrument_names_fromlist_invalid() {
    let instrument_list = util::InstrumentMap {
        map: [
            (
                "HAZ_FRONT",
                vec!["FHAZ_RIGHT_A", "FHAZ_LEFT_A", "FHAZ_RIGHT_B", "FHAZ_LEFT_B"],
            ),
            (
                "HAZ_REAR",
                vec!["RHAZ_RIGHT_A", "RHAZ_LEFT_A", "RHAZ_RIGHT_B", "RHAZ_LEFT_B"],
            ),
            ("NAV_LEFT", vec!["NAV_LEFT_A", "NAV_LEFT_B"]),
            ("NAV_RIGHT", vec!["NAV_RIGHT_A", "NAV_RIGHT_B"]),
            ("CHEMCAM", vec!["CHEMCAM_RMI"]),
            ("MARDI", vec!["MARDI"]),
            ("MAHLI", vec!["MAHLI"]),
            ("MASTCAM", vec!["MAST_LEFT", "MAST_RIGHT"]),
        ]
        .iter()
        .cloned()
        .collect(),
    };

    let list: Vec<&str> = vec!["NAV_LEFT", "CHEMCAM", "FOO"];

    let string_lis: Vec<String> = list.iter().map(|s| String::from(*s)).collect();
    let _remote_instruments = instrument_list
        .find_remote_instrument_names_fromlist(&string_lis)
        .unwrap();
}
