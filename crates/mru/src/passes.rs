use crate::serializers::{as_df_doy, as_f32, as_f32_opt, as_u32};
use crate::{constants, jsonfetch};
use anyhow::{Result, anyhow};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreAct {
    #[serde(alias = "HAILSTART", with = "as_df_doy")]
    pub hail_start: DateTime<FixedOffset>,

    #[serde(alias = "HAILEND", with = "as_df_doy")]
    pub hail_end: DateTime<FixedOffset>,

    #[serde(alias = "RETURNVOL", with = "as_f32_opt", default)]
    pub return_volume: Option<f32>,

    #[serde(alias = "RETURNRATE", with = "as_u32")]
    pub return_rate: u32,

    #[serde(alias = "FORWARDRATE", with = "as_u32")]
    pub forward_rate: u32,

    #[serde(alias = "ADR")]
    pub adr: Option<String>,

    #[serde(alias = "LINKTYPE")]
    pub link_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Overflight {
    #[serde(alias = "ID")]
    pub overflight_id: String,

    #[serde(alias = "ORBITER")]
    pub spacecraft_orbiter: String,

    #[serde(alias = "LANDER")]
    pub spacecraft_lander: String,

    #[serde(alias = "MAXELEV", with = "as_f32")]
    pub maximum_elevation: f32,

    #[serde(alias = "MAXELEVTIME", with = "as_df_doy")]
    pub maximum_elevation_time: DateTime<FixedOffset>,

    #[serde(alias = "STARTTIME", with = "as_df_doy")]
    pub start_time: DateTime<FixedOffset>,

    #[serde(alias = "ENDTIME", with = "as_df_doy")]
    pub end_time: DateTime<FixedOffset>,

    #[serde(alias = "PRE")]
    pub predict: Option<PreAct>,

    #[serde(alias = "ACT")]
    pub actual: Option<PreAct>,
}

fn parse_json(raw_json: &str) -> Result<Vec<Overflight>> {
    match serde_json::from_str(raw_json) {
        Ok(res) => Ok(res),
        Err(why) => Err(anyhow!("Failed to parse overpass json: {:?}", why)),
    }
}

pub async fn fetch_passes() -> Result<Vec<Overflight>> {
    let req = jsonfetch::JsonFetcher::new(constants::url::MARS_RELAY_PASSES)?;

    let response_string = match req.fetch_str().await {
        Ok(response_string) => response_string,
        Err(why) => return Err(anyhow!("Remote error fetching overpasses: {:?}", why)),
    };

    parse_json(&response_string)
}

#[test]
fn test_parse_json() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/maros")
        .join("overflights.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_json(&raw_json).expect("Failed to parse json");
}
