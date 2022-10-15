use mars_raw_utils::nsyt::{latest::LatestData, remote::fetch_latest, remote::remote_fetch};

#[tokio::test]
#[ignore]
async fn test_msl_latest() {
    let latest: LatestData = fetch_latest().await.expect("Failed to fetch latest data");
    assert_eq!(latest.latest, "2022-02-14T15:11:15Z");
    assert_eq!(latest.latest_sols.len(), 1);
}

#[tokio::test]
#[ignore] // Ignoring this by default to prevent unneccessary load on NASA's servers
async fn test_nsyt_instrument_fetches() {
    let instruments = vec!["idc", "icc"];
    let f: Vec<String> = vec![];

    for i in instruments {
        eprintln!("Testing fetch for {}", i);
        remote_fetch(
            &vec![String::from(i)],
            5,
            Some(0),
            3119,
            3119,
            false,
            true,
            &f,
            false,
            "",
        )
        .await
        .unwrap();
    }
}
