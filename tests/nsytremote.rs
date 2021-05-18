use mars_raw_utils::nsyt::remote::remote_fetch;


#[test]
#[ignore] // Ignoring this by default to prevent unneccessary load on NASA's servers
fn test_nsyt_instrument_fetches() {
    let instruments = vec!["idc", "icc"];

    for i in instruments {
        eprintln!("Testing fetch for {}", i);
        remote_fetch(&vec![String::from(i)], 5, Some(0), 3119, 3119, false, true, "", false).unwrap();
    }

}