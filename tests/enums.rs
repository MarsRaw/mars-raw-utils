
use mars_raw_utils::enums;


#[test]
fn test_maxvalue() {
    assert_eq!(enums::ImageMode::maxvalue(enums::ImageMode::U8BIT), 255.0);
    assert_eq!(enums::ImageMode::maxvalue(enums::ImageMode::U12BIT), 2033.0);
    assert_eq!(enums::ImageMode::maxvalue(enums::ImageMode::U16BIT), 65535.0);
}