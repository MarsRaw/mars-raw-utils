use crate::jsonfetch;
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_this_or_that::{as_f64, as_i64};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Overflight {
    #[serde(alias = "OVERFLIGHTID")]
    pub overflight_id: String,

    #[serde(alias = "SPACECRAFTORBITER")]
    pub spacecraft_orbiter: String,

    #[serde(alias = "SPACECRAFTLANDER")]
    pub spacecraft_lander: String,

    #[serde(alias = "STARTRISEYEAR", deserialize_with = "as_i64")]
    pub start_rise_year: i64,

    #[serde(alias = "STARTRISEDAYOFYEAR", deserialize_with = "as_i64")]
    pub start_rise_day_of_year: i64,

    #[serde(alias = "OVERFLIGHTPASSNUMBER", deserialize_with = "as_i64")]
    pub overflight_pass_number: i64,

    #[serde(alias = "MAXIMUMELEVATION", deserialize_with = "as_f64")]
    pub maximum_elevation: f64,

    #[serde(alias = "MAXIMUMELEVATIONTIME", with = "doy_date_format")]
    pub maximum_elevation_time: DateTime<FixedOffset>,

    #[serde(alias = "MAXIMUMELEVATIONRANGE", deserialize_with = "as_f64")]
    pub maximum_elevation_range: f64,

    #[serde(alias = "STARTTIME", with = "doy_date_format")]
    pub start_time: DateTime<FixedOffset>,

    #[serde(alias = "ENDTIME", with = "doy_date_format")]
    pub end_time: DateTime<FixedOffset>,

    #[serde(alias = "RISESETDURATION", deserialize_with = "as_f64")]
    pub rise_set_duration: f64,

    #[serde(alias = "REQUESTTYPE")]
    pub request_type: String,

    #[serde(alias = "REQUESTCATEGORY")]
    pub request_category: String,

    #[serde(alias = "REQUESTFORWARDLINKDATARATE", deserialize_with = "as_i64")]
    pub request_forward_link_data_rate: i64,

    #[serde(alias = "REQUESTRETURNLINKDATARATE", deserialize_with = "as_i64")]
    pub request_return_link_data_rate: i64,

    #[serde(alias = "REQUESTDATAVOLUMERETURNED", deserialize_with = "as_i64")]
    pub request_data_volume_returned: i64,

    #[serde(alias = "REQUESTADR_ENABLE_FLAG")]
    pub request_adr_enable_flag: String,

    #[serde(alias = "ACKTYPE")]
    pub ack_type: String,

    #[serde(alias = "ACKSUPPORTPLAN")]
    pub ack_support_plan: String,

    #[serde(alias = "ACKFORWARDLINKDATARATE", deserialize_with = "as_i64")]
    pub ack_forward_link_data_rate: i64,

    #[serde(alias = "ACKRETURNLINKDATARATE", deserialize_with = "as_i64")]
    pub ack_return_link_data_rate: i64,

    #[serde(alias = "ACKADR_ENABLE_FLAG")]
    pub ack_adr_enable_flag: String,

    #[serde(
        alias = "ORBITERSCORECARDFORWARDLINKDATARATE",
        deserialize_with = "as_i64"
    )]
    pub orbiter_scorecard_forward_link_data_rate: i64,

    #[serde(
        alias = "ORBITERSCORECARDRETURNLINKDATARATE",
        deserialize_with = "as_i64"
    )]
    pub orbiter_scorecard_return_link_data_rate: i64,

    #[serde(
        alias = "ORBITERSCORECARDDATAVOLUMERETURNED",
        deserialize_with = "as_i64"
    )]
    pub orbiter_scorecard_data_volume_returned: i64,

    #[serde(alias = "LINKTYPE")]
    pub link_type: String,

    #[serde(alias = "HAILSTARTSRC")]
    pub hail_start_src: String,

    #[serde(alias = "HAILSTART", with = "doy_date_format")]
    pub hail_start: DateTime<FixedOffset>,

    #[serde(alias = "HAILENDSRC")]
    pub hail_end_src: String,

    #[serde(alias = "HAILEND", with = "doy_date_format")]
    pub hail_end: DateTime<FixedOffset>,

    #[serde(alias = "HAILDURATION", deserialize_with = "as_i64")]
    pub hail_duration: i64,

    #[serde(alias = "DATELASTUPDATED", with = "doy_date_format")]
    pub date_last_updated: DateTime<FixedOffset>,
}

// https://serde.rs/custom-date-format.html
mod doy_date_format {
    use chrono::{DateTime, FixedOffset, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%jT%H:%M:%S%.3f %z";

    pub fn serialize<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        println!("Deserialize this: {}", s);
        if s.is_empty() {
            Ok(Utc::now().fixed_offset())
        } else {
            DateTime::parse_from_str(&format!("{} +0000", s), FORMAT)
                .map_err(serde::de::Error::custom)
        }
    }
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
