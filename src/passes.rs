use crate::serializers::{as_df_doy, as_f32, as_u32};
use crate::{constants, jsonfetch};
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreAct {
    #[serde(alias = "HAILSTART", with = "as_df_doy")]
    pub hail_start: DateTime<FixedOffset>,

    #[serde(alias = "HAILEND", with = "as_df_doy")]
    pub hail_end: DateTime<FixedOffset>,

    #[serde(alias = "RETURNVOL", with = "as_f32")]
    pub return_volume: f32,

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

pub async fn fetch_passes() -> Result<Vec<Overflight>> {
    let req = jsonfetch::JsonFetcher::new(constants::url::MARS_RELAY_PASSES)?;

    let response_string = match req.fetch_str().await {
        Ok(response_string) => response_string,
        Err(why) => return Err(anyhow!("Remote error fetching overpasses: {:?}", why)),
    };

    match serde_json::from_str(response_string.as_str()) {
        Ok(res) => Ok(res),
        Err(why) => Err(anyhow!("Failed to parse overpass json: {:?}", why)),
    }
}
