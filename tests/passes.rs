use mars_raw_utils::passes;

#[tokio::test]
async fn test_fetch() {
    let _r = passes::fetch_passes().await.unwrap();
}
