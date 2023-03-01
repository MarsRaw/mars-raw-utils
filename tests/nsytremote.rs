use mars_raw_utils::nsyt::{latest::LatestData, remote::fetch_latest, remote::remote_fetch};

#[tokio::test]
async fn test_nsyt_latest() {
    fetch_latest().await.expect("Failed to fetch latest data");
}

#[tokio::test]
async fn test_nsyt_instrument_fetches() {
    let instruments = vec!["idc", "icc"];
    let f: Vec<String> = vec![];

    for i in instruments {
        eprintln!("Testing fetch for {}", i);
        remote_fetch(
            &[String::from(i)],
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
