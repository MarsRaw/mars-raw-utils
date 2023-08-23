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

    let response: Location = match req.fetch_str().await {
        Ok(v) => {
            let res: WaypointsCurrent = serde_json::from_str(v.as_str())?;
            if !res.features.is_empty() {
                res.features[0].properties.to_owned()
            } else {
                return Err(anyhow!("Error: No location found in response"));
            }
        }
        Err(e) => return Err(anyhow!("Error: {:?}", e)),
    };

    Ok(response)
}
