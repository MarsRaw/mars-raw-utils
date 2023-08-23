use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use mars_raw_utils::{constants::url, location};

#[derive(Parser)]
#[command(author, version, about = "Current reported Mars2020 location information", long_about = None)]
pub struct M20Location {}

#[async_trait]
impl RunnableSubcommand for M20Location {
    async fn run(&self) -> Result<()> {
        let loc = location::fetch_location(url::M20_LOCATION_URL).await?;

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
