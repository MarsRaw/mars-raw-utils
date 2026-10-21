use crate::serializers::as_df_doy;
use crate::{constants, jsonfetch};
use anyhow::{anyhow, Result};
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

pub async fn fetch_dsn_events() -> Result<Vec<DsnEvent>> {
    let req = jsonfetch::JsonFetcher::new(constants::url::MARS_RELAY_DSN_EVENTS)?;

    let response_string = match req.fetch_str().await {
        Ok(response_string) => response_string,
        Err(why) => return Err(anyhow!("Remote error fetching dsn events: {:?}", why)),
    };

    match serde_json::from_str(response_string.as_str()) {
        Ok(res) => Ok(res),
        Err(why) => Err(anyhow!("Failed to parse dsn event json: {:?}", why)),
    }
}
