use crate::jsonfetch;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    #[serde(alias = "RMC")]
    pub rmc: String,
    pub site: i64,
    pub drive: i64,
    pub sol: i64,
    pub easting: f64,
    pub northing: f64,
    pub elev_geoid: f64,
    pub lon: f64,
    pub lat: f64,
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
    pub yaw_rad: f64,
    pub tilt: f64,
    pub dist_m: f64,
    pub dist_total_m: f64,
    pub dist_km: f64,
    pub dist_mi: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Feature {
    #[serde(alias = "type")]
    pub feature_type: String,
    pub properties: Location,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WaypointsCurrent {
    #[serde(alias = "type")]
    pub doc_type: String,
    pub name: String,
    pub features: Vec<Feature>,
}

fn parse_waypoints(raw_json: &str) -> Result<WaypointsCurrent> {
    match serde_json::from_str(raw_json) {
        Ok(res) => Ok(res),
        Err(why) => Err(anyhow!("Error: failed to parse waypoints json: {:?}", why)),
    }
}

pub async fn fetch_waypoints(url: &str) -> Result<Vec<Location>> {
    let req = jsonfetch::JsonFetcher::new(url)
        .map_err(|why| anyhow!("Error: Failed to create json fetch container: {:?}", why))?;

    let raw_json = match req.fetch_str().await {
        Ok(raw_json) => raw_json,
        Err(why) => {
            return Err(anyhow!(
                "Error fetching remote json from {}: {:?}",
                url,
                why
            ));
        }
    };

    let waypoints = parse_waypoints(&raw_json)?;
    let locations: Vec<Location> = waypoints
        .features
        .into_iter()
        .map(|f| f.properties)
        .collect();
    Ok(locations)
}

pub async fn fetch_location(url: &str) -> Result<Location> {
    let locations = fetch_waypoints(url).await?;

    match locations.first() {
        Some(loc) => Ok(loc.to_owned()),
        None => Err(anyhow!("Error: No location found in response")),
    }
}

pub async fn print_all(url: &str) -> Result<()> {
    let waypoints = fetch_waypoints(url).await?;
    let mut first_evel = if !waypoints.is_empty() {
        waypoints[0].elev_geoid
    } else {
        return Err(anyhow!("No waypoints found"));
    };

    println!(
        "Site  Drive   Sol     Easting    Northing  Elevation   Climb       Lon       Lat Dist(m) Total (km)"
    );
    waypoints.into_iter().for_each(|wp| {
        let elev_change = wp.elev_geoid - first_evel;
        first_evel = wp.elev_geoid;

        println!(
            "{:>5} {:>5} {:>5} {:>11.3} {:>11.3} {:>10.2} {:>7.2} {:>9.5} {:>9.5} {:>7.2} {:10.2}",
            wp.site,
            wp.drive,
            wp.sol,
            wp.easting,
            wp.northing,
            wp.elev_geoid,
            elev_change,
            wp.lon,
            wp.lat,
            wp.dist_m,
            wp.dist_km
        );
    });
    Ok(())
}

pub async fn print_all_csv(url: &str) -> Result<()> {
    let waypoints = fetch_waypoints(url).await?;
    println!("Site,Drive,Sol,Easting,Northing,Elevation,Lon,Lat,Dist(m),Total (km)");

    let mut first_evel = if !waypoints.is_empty() {
        waypoints[0].elev_geoid
    } else {
        return Err(anyhow!("No waypoints found"));
    };

    waypoints.into_iter().for_each(|wp| {
        let elev_change = wp.elev_geoid - first_evel;
        first_evel = wp.elev_geoid;

        println!(
            "{},{},{},{},{},{},{:.3},{},{},{},{}",
            wp.site,
            wp.drive,
            wp.sol,
            wp.easting,
            wp.northing,
            wp.elev_geoid,
            elev_change,
            wp.lon,
            wp.lat,
            wp.dist_m,
            wp.dist_km
        );
    });
    Ok(())
}

pub async fn print_single(url: &str) -> Result<()> {
    let loc = fetch_location(url).await?;

    println!("Site: {}", loc.site);
    println!("Drive: {}", loc.drive);
    println!("Sol: {}", loc.sol);
    println!("Easting: {}", loc.easting);
    println!("Northing: {}", loc.northing);
    println!("Elevation (geoid): {}", loc.elev_geoid);
    println!("Longitude: {}", loc.lon);
    println!("Latitude: {}", loc.lat);
    println!("Roll: {}", loc.roll);
    println!("Pitch: {}", loc.pitch);
    println!("Yaw: {}", loc.yaw);
    println!("Tilt: {}", loc.tilt);
    println!("Drive Distance (meters): {}", loc.dist_m);
    println!("Total Traverse Distance (kilometers): {}", loc.dist_km);

    Ok(())
}

#[test]
fn test_parse_waypoints_current_msl() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/mmgis")
        .join("MSL_waypoints_current.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_waypoints(&raw_json).expect("Failed to parse JSON");
}

#[test]
fn test_parse_waypoints_msl() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/mmgis")
        .join("MSL_waypoints.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_waypoints(&raw_json).expect("Failed to parse JSON");
}

#[test]
fn test_parse_waypoints_current_m20() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/mmgis")
        .join("M20_waypoints_current.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_waypoints(&raw_json).expect("Failed to parse JSON");
}

#[test]
fn test_parse_waypoints_m20() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/mmgis")
        .join("M20_waypoints.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_waypoints(&raw_json).expect("Failed to parse JSON");
}
