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

