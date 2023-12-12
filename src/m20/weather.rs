use crate::jsonfetch;
use crate::serializers::{as_df_date, as_f32, as_i32};
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MedaSol {
    #[serde(with = "as_df_date")]
    pub terrestrial_date: DateTime<FixedOffset>,

    #[serde(with = "as_i32")]
    pub sol: i32,

    #[serde(alias = "ls", with = "as_f32")]
    pub solar_longitude: f32,

    pub season: String,

    pub min_temp: f32,

    pub max_temp: f32,

    pub pressure: f32,

    pub sunrise: String,

    pub sunset: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MedaSols {
    pub sols: Vec<MedaSol>,
}

pub async fn fetch_weather() -> Result<Vec<MedaSol>> {
    let req = jsonfetch::JsonFetcher::new(
        "https://mars.nasa.gov/rss/api/?feed=weather&category=mars2020&feedtype=json",
    )?;

    let response: Vec<MedaSol> = match req.fetch_str().await {
        Ok(v) => {
            let res: MedaSols = serde_json::from_str(v.as_str())?;
            res.sols
        }
        Err(e) => return Err(anyhow!("Error: {:?}", e)),
    };

    Ok(response)
}
