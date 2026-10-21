use mars_raw_utils::dsnevents;

#[tokio::test]
async fn test_fetch() {
    let _ = dsnevents::fetch_dsn_events().await.unwrap();
}
