use crate::jsonfetch;
use crate::serializers::{as_df_doy, as_f64, as_i64};
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Overflight {
    #[serde(alias = "OVERFLIGHTID")]
    pub overflight_id: String,

    #[serde(alias = "SPACECRAFTORBITER")]
    pub spacecraft_orbiter: String,

    #[serde(alias = "SPACECRAFTLANDER")]
    pub spacecraft_lander: String,

    #[serde(alias = "STARTRISEYEAR", with = "as_i64")]
    pub start_rise_year: i64,

    #[serde(alias = "STARTRISEDAYOFYEAR", with = "as_i64")]
    pub start_rise_day_of_year: i64,

    #[serde(alias = "OVERFLIGHTPASSNUMBER", with = "as_i64")]
    pub overflight_pass_number: i64,

    #[serde(alias = "MAXIMUMELEVATION", with = "as_f64")]
    pub maximum_elevation: f64,

    #[serde(alias = "MAXIMUMELEVATIONTIME", with = "as_df_doy")]
    pub maximum_elevation_time: DateTime<FixedOffset>,

    #[serde(alias = "MAXIMUMELEVATIONRANGE", with = "as_f64")]
    pub maximum_elevation_range: f64,

    #[serde(alias = "STARTTIME", with = "as_df_doy")]
    pub start_time: DateTime<FixedOffset>,

    #[serde(alias = "ENDTIME", with = "as_df_doy")]
    pub end_time: DateTime<FixedOffset>,

    #[serde(alias = "RISESETDURATION", with = "as_f64")]
    pub rise_set_duration: f64,

    #[serde(alias = "REQUESTTYPE")]
    pub request_type: String,

    #[serde(alias = "REQUESTCATEGORY")]
    pub request_category: String,

    #[serde(alias = "REQUESTFORWARDLINKDATARATE", with = "as_i64")]
    pub request_forward_link_data_rate: i64,

    #[serde(alias = "REQUESTRETURNLINKDATARATE", with = "as_i64")]
    pub request_return_link_data_rate: i64,

    #[serde(alias = "REQUESTDATAVOLUMERETURNED", with = "as_f64")]
    pub request_data_volume_returned: f64,

    #[serde(alias = "REQUESTADR_ENABLE_FLAG")]
    pub request_adr_enable_flag: String,

    #[serde(alias = "ACKTYPE")]
    pub ack_type: String,

    #[serde(alias = "ACKSUPPORTPLAN")]
    pub ack_support_plan: String,

    #[serde(alias = "ACKFORWARDLINKDATARATE", with = "as_i64")]
    pub ack_forward_link_data_rate: i64,

    #[serde(alias = "ACKRETURNLINKDATARATE", with = "as_i64")]
    pub ack_return_link_data_rate: i64,

    #[serde(alias = "ACKADR_ENABLE_FLAG")]
    pub ack_adr_enable_flag: String,

    #[serde(alias = "ORBITERSCORECARDFORWARDLINKDATARATE", with = "as_i64")]
    pub orbiter_scorecard_forward_link_data_rate: i64,

    #[serde(alias = "ORBITERSCORECARDRETURNLINKDATARATE", with = "as_i64")]
    pub orbiter_scorecard_return_link_data_rate: i64,

    #[serde(alias = "ORBITERSCORECARDDATAVOLUMERETURNED", with = "as_i64")]
    pub orbiter_scorecard_data_volume_returned: i64,

    #[serde(alias = "LINKTYPE")]
    pub link_type: String,

    #[serde(alias = "HAILSTARTSRC")]
    pub hail_start_src: String,

    #[serde(alias = "HAILSTART", with = "as_df_doy")]
    pub hail_start: DateTime<FixedOffset>,

    #[serde(alias = "HAILENDSRC")]
    pub hail_end_src: String,

    #[serde(alias = "HAILEND", with = "as_df_doy")]
    pub hail_end: DateTime<FixedOffset>,

    #[serde(alias = "HAILDURATION", with = "as_f64")]
    pub hail_duration: f64,

    #[serde(alias = "DATELASTUPDATED", with = "as_df_doy")]
    pub date_last_updated: DateTime<FixedOffset>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OverflightResponse {
    #[serde(alias = "marsRelay")]
    pub overflights: Vec<Overflight>,
}

pub async fn fetch_passes() -> Result<Vec<Overflight>> {
    let req = jsonfetch::JsonFetcher::new("https://mars.nasa.gov/mrn_passthru/")?;

    let response: Vec<Overflight> = match req.fetch_str().await {
        Ok(v) => {
            let res: OverflightResponse = serde_json::from_str(v.as_str())?;
            res.overflights
        }
        Err(e) => return Err(anyhow!("Error: {:?}", e)),
    };

    Ok(response)
}
