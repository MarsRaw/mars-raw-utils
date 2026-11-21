use crate::serializers::{as_df_date, as_f32, as_i32, none_on_error};
use crate::{constants, jsonfetch};
use anyhow::{Result, anyhow};
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

    #[serde(default, deserialize_with = "none_on_error")]
    pub min_temp: Option<f32>,

    #[serde(default, deserialize_with = "none_on_error")]
    pub max_temp: Option<f32>,

    #[serde(default, deserialize_with = "none_on_error")]
    pub pressure: Option<f32>,

    pub sunrise: String,

    pub sunset: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MedaSols {
    pub sols: Vec<MedaSol>,
}

fn parse_json_data(raw_json: &str) -> Result<Vec<MedaSol>> {
    let res: MedaSols = match serde_json::from_str(raw_json) {
        Ok(res) => res,
        Err(why) => return Err(anyhow!("Failed to parse M20 weather json data: {:?}", why)),
    };
    Ok(res.sols)
}

pub async fn fetch_weather() -> Result<Vec<MedaSol>> {
    let req = jsonfetch::JsonFetcher::new(constants::url::M20_WEATHER_URL)?;
    let raw_json = req.fetch_str().await?;

    parse_json_data(&raw_json)
}

#[test]
fn test_parse_json() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/marsapi")
        .join("m20_weather.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_json_data(&raw_json).expect("Failed to parse json");
}
