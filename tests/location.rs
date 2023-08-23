use mars_raw_utils::location;

#[tokio::test]
async fn test_fetch_msl() {
    let _loc = location::fetch_location(
        "https://mars.nasa.gov/mmgis-maps/MSL/Layers/json/MSL_waypoints_current.json",
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_fetch_m20() {
    let _loc = location::fetch_location(
        "https://mars.nasa.gov/mmgis-maps/M20/Layers/json/M20_waypoints_current.json",
    )
    .await
    .unwrap();
}
