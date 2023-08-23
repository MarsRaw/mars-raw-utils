use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use mars_raw_utils::location;

#[derive(Parser)]
#[command(author, version, about = "Current reported MSL location information", long_about = None)]
pub struct MslLocation {}

#[async_trait]
impl RunnableSubcommand for MslLocation {
    async fn run(&self) -> Result<()> {
        let loc = location::fetch_location(
            "https://mars.nasa.gov/mmgis-maps/MSL/Layers/json/MSL_waypoints_current.json",
        )
        .await?;

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
}
