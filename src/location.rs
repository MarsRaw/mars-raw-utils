use crate::jsonfetch;
use anyhow::{anyhow, Result};
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

pub async fn fetch_location(url: &str) -> Result<Location> {
    let req = jsonfetch::JsonFetcher::new(url)?;
    match req.fetch_str().await {
        Ok(v) => {
            let res: WaypointsCurrent = serde_json::from_str(v.as_str())?;
            if !res.features.is_empty() {
                Ok(res.features[0].properties.to_owned())
            } else {
                Err(anyhow!("Error: No location found in response"))
            }
        }
        Err(e) => Err(anyhow!("Error: {:?}", e)),
    }
}

pub async fn fetch_waypoints(url: &str) -> Result<Vec<Location>> {
    let req = jsonfetch::JsonFetcher::new(url)?;
    match req.fetch_str().await {
        Ok(v) => {
            let res: WaypointsCurrent = serde_json::from_str(v.as_str())?;
            let locations: Vec<Location> = res
                .features
                .into_iter()
                .map(|f| f.properties.clone())
                .collect();
            Ok(locations)
        }
        Err(e) => Err(anyhow!("Error: {:?}", e)),
    }
}
