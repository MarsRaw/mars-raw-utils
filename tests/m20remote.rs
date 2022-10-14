use mars_raw_utils::m20::{latest, remote};

#[test]
#[ignore]
fn test_msl_latest() {
    let latest: latest::LatestData = remote::fetch_latest().expect("Failed to fetch latest data");
    assert_eq!(latest.latest, "2022-02-19T16:36:39Z");
    assert_eq!(latest.latest_sols.len(), 3);
}

#[test]
#[ignore] // Going to ignore this by default to prevent unneccessary load on NASA's servers
fn test_m20_instrument_fetches() {
    let instruments = vec![
        "FRONT_HAZCAM_LEFT_A",
        "FRONT_HAZCAM_LEFT_B",
        "FRONT_HAZCAM_RIGHT_A",
        "FRONT_HAZCAM_RIGHT_B",
        "HAZ_FRONT",
        "SUPERCAM_RMI",
        "REAR_HAZCAM_LEFT",
        "REAR_HAZCAM_RIGHT",
        "NAVCAM_LEFT",
        "NAVCAM_RIGHT",
        "MCZ_LEFT",
        "MCZ_RIGHT",
        "EDL_DDCAM",
        "EDL_PUCAM1",
        "EDL_PUCAM2",
        "EDL_RUCAM",
        "EDL_RDCAM",
        "LCAM",
        "SHERLOC_WATSON",
    ];

    let f: Vec<String> = vec![];

    for i in instruments {
        eprintln!("Testing fetch for {}", i);
        remote::remote_fetch(
            &vec![String::from(i)],
            5,
            Some(0),
            70,
            79,
            false,
            false,
            true,
            &f,
            false,
            "",
        )
        .unwrap();
    }
}
