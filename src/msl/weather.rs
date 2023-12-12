use crate::jsonfetch;
use crate::serializers::{as_df_date, as_f32, as_i32};
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemsSol {
    pub id: String,

    #[serde(with = "as_df_date")]
    pub terrestrial_date: DateTime<FixedOffset>,

    #[serde(with = "as_i32")]
    pub sol: i32,

    #[serde(alias = "ls", with = "as_f32")]
    pub solar_longitude: f32,

    pub season: String,

    #[serde(with = "as_f32")]
    pub min_temp: f32,

    #[serde(with = "as_f32")]
    pub max_temp: f32,

    #[serde(with = "as_f32")]
    pub pressure: f32,

    pub pressure_string: String,

    #[serde(with = "as_f32")]
    pub abs_humidity: f32,

    #[serde(with = "as_f32")]
    pub wind_speed: f32,

    pub wind_direction: String,

    pub atmo_opacity: String,

    pub sunrise: String,

    pub sunset: String,

    pub local_uv_irradiance_index: String,

    #[serde(with = "as_f32")]
    pub min_gts_temp: f32,

    #[serde(with = "as_f32")]
    pub max_gts_temp: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemsSols {
    pub soles: Vec<RemsSol>,
}

pub async fn fetch_weather() -> Result<Vec<RemsSol>> {
    let req = jsonfetch::JsonFetcher::new(
        "https://mars.nasa.gov/rss/api/?feed=weather&category=msl&feedtype=json",
    )?;

    let response: Vec<RemsSol> = match req.fetch_str().await {
        Ok(v) => {
            let res: RemsSols = serde_json::from_str(v.as_str())?;
            res.soles
        }
        Err(e) => return Err(anyhow!("Error: {:?}", e)),
    };

    Ok(response)
}
