use mars_raw_utils::m20::zcam;

#[test]
fn test_focal_length_from_file_name() {
    assert_eq!(
        zcam::focal_length_from_file_name(
            "ZR0_0395_0702017827_081EBY_N0171064ZCAM08419_1100LMJ01.png"
        ),
        Ok(110.0_f32)
    );
    assert_eq!(
        zcam::focal_length_from_file_name("foo.png"),
        Err("Filename is invalid M20/MCZ format")
    );
    assert_eq!(
        zcam::focal_length_from_file_name(
            "ZR0_0395_0702017827_081EBY_N0171064ZCAM08419_1G00LMJ01.png"
        ),
        Err("Invalid value")
    );
    assert_eq!(
        zcam::focal_length_from_file_name(
            "/data/M20/0395/ZCAM/ZR0_0395_0702017827_081EBY_N0171064ZCAM08419_1100LMJ01.png"
        ),
        Ok(110.0_f32)
    );

    assert_eq!(
        zcam::focal_length_from_file_name(
            "ZR0_0395_0702017827_081EBY_N0171064ZCAM08419_0340LMJ01.png"
        ),
        Ok(34.0_f32)
    );
    assert_eq!(
        zcam::focal_length_from_file_name(
            "/data/M20/0395/ZCAM/ZR0_0395_0702017827_081EBY_N0171064ZCAM08419_0340LMJ01.png"
        ),
        Ok(34.0_f32)
    );
    assert_eq!(
        zcam::focal_length_from_file_name(
            "ZR0_0395_0702017827_081EBY_N0171064ZCAM08419_M340LMJ01.png"
        ),
        Err("Invalid value")
    );
}
