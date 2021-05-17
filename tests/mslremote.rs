
use mars_raw_utils::msl::remote::remote_fetch;


#[test]
#[ignore] // Going to ignore this by default to prevent unneccessary load on NASA's servers
fn test_msl_instrument_fetches() {
    remote_fetch(&vec![String::from("MAST_LEFT")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("MAST_RIGHT")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("MARDI")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("MAHLI")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("CHEMCAM_RMI")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("MAST_LEFT")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("NAV_RIGHT_A")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("NAV_RIGHT_B")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("NAV_LEFT_A")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("NAV_LEFT_B")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("FHAZ_RIGHT_A")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("FHAZ_RIGHT_B")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("FHAZ_LEFT_A")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("FHAZ_LEFT_B")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("RHAZ_RIGHT_A")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("RHAZ_RIGHT_B")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("RHAZ_LEFT_A")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    remote_fetch(&vec![String::from("RHAZ_LEFT_B")], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
}