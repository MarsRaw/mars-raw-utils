use mars_raw_utils::{constants::url, location};

#[tokio::test]
async fn test_fetch_msl() {
    let _loc = location::fetch_location(url::MSL_LOCATION_URL)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_fetch_waypoints_msl() {
    let locations = location::fetch_waypoints(url::MSL_WAYPOINTS_URL)
        .await
        .unwrap();

    assert!(!locations.is_empty());
}

#[tokio::test]
async fn test_fetch_m20() {
    let _loc = location::fetch_location(url::M20_LOCATION_URL)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_fetch_waypoints_m20() {
    let locations = location::fetch_waypoints(url::M20_WAYPOINTS_URL)
        .await
        .unwrap();

    assert!(!locations.is_empty());
}
