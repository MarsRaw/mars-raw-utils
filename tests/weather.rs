use mars_raw_utils::m20::weather as m20weather;
use mars_raw_utils::msl::weather as mslweather;

#[tokio::test]
async fn test_m20_fetch() {
    m20weather::fetch_weather().await.unwrap();
}

#[tokio::test]
async fn test_msl_fetch() {
    mslweather::fetch_weather().await.unwrap();
}
