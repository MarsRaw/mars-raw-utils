

use mars_raw_utils::m20::remote::remote_fetch;


#[test]
#[ignore] // Going to ignore this by default to prevent unneccessary load on NASA's servers
fn test_m20_instrument_fetches() {
    let instruments = vec!["FRONT_HAZCAM_LEFT_A", "FRONT_HAZCAM_LEFT_B", "FRONT_HAZCAM_RIGHT_A", "FRONT_HAZCAM_RIGHT_B",
    "HAZ_FRONT", "SUPERCAM_RMI","REAR_HAZCAM_LEFT", "REAR_HAZCAM_RIGHT", "NAVCAM_LEFT", "NAVCAM_RIGHT",
    "MCZ_LEFT","MCZ_RIGHT", "EDL_DDCAM", "EDL_PUCAM1", "EDL_PUCAM2", "EDL_RUCAM", "EDL_RDCAM", "LCAM",
    "SHERLOC_WATSON"];

    for i in instruments {
        eprintln!("Testing fetch for {}", i);
        remote_fetch(&vec![String::from(i)], 5, Some(0), 70, 79, false, false, true, "", false).unwrap();
    }

}