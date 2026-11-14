use crate::serializers::as_df_doy;
use crate::{constants, jsonfetch};
use anyhow::{Result, anyhow};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DsnEvent {
    #[serde(alias = "ORBITER")]
    pub orbiter: String,

    #[serde(alias = "EVENTTYPE")]
    pub event_type: String,

    #[serde(alias = "STARTTIME", with = "as_df_doy")]
    pub start_time: DateTime<FixedOffset>,

    #[serde(alias = "ENDTIME", with = "as_df_doy")]
    pub end_time: DateTime<FixedOffset>,

    #[serde(alias = "DSS")]
    pub dss: String,
}

fn parse_json(raw_json: &str) -> Result<Vec<DsnEvent>> {
    match serde_json::from_str(raw_json) {
        Ok(res) => Ok(res),
        Err(why) => Err(anyhow!("Failed to parse dsn event json: {:?}", why)),
    }
}

pub async fn fetch_dsn_events() -> Result<Vec<DsnEvent>> {
    let req = jsonfetch::JsonFetcher::new(constants::url::MARS_RELAY_DSN_EVENTS)?;

    let response_string = match req.fetch_str().await {
        Ok(response_string) => response_string,
        Err(why) => return Err(anyhow!("Remote error fetching dsn events: {:?}", why)),
    };

    parse_json(&response_string)
}

#[test]
fn test_parse_json() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let file_path = std::path::Path::new(manifest_dir)
        .join("tests/testdata/maros")
        .join("dsn-event.json");

    let raw_json = std::fs::read_to_string(file_path).expect("Failed to load test data");

    parse_json(&raw_json).expect("Failed to parse json");
}
