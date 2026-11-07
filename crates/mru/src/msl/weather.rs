use crate::serializers::{as_df_date, as_f32, as_f32_opt, as_i32};
use crate::{constants, jsonfetch};
use anyhow::{Result, anyhow};
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

    #[serde(with = "as_f32_opt")]
    pub min_temp: Option<f32>,

    #[serde(with = "as_f32_opt")]
    pub max_temp: Option<f32>,

    #[serde(with = "as_f32_opt")]
    pub pressure: Option<f32>,

    pub pressure_string: String,

    #[serde(with = "as_f32_opt")]
    pub abs_humidity: Option<f32>,

    #[serde(with = "as_f32_opt")]
    pub wind_speed: Option<f32>,

    pub wind_direction: String,

    pub atmo_opacity: String,

    pub sunrise: String,

    pub sunset: String,

    pub local_uv_irradiance_index: String,

    #[serde(with = "as_f32_opt")]
    pub min_gts_temp: Option<f32>,

    #[serde(with = "as_f32_opt")]
    pub max_gts_temp: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemsSols {
    #[serde(alias = "soles")]
    pub sols: Vec<RemsSol>,
}

fn parse_json_data(raw_json: &str) -> Result<Vec<RemsSol>> {
    let res: RemsSols = match serde_json::from_str(raw_json) {
        Ok(res) => res,
        Err(why) => return Err(anyhow!("Failed to parse M20 weather json data: {:?}", why)),
    };
    Ok(res.sols)
}

pub async fn fetch_weather() -> Result<Vec<RemsSol>> {
    let req = jsonfetch::JsonFetcher::new(constants::url::MSL_WEATHER_URL)?;
    let raw_json = req.fetch_str().await?;

    parse_json_data(&raw_json)
}

#[test]
fn test_parse_json() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/marsapi")
        .join("msl_weather.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_json_data(&raw_json).expect("Failed to parse json");
}
