use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

use crate::serializers::{as_f32, as_i32, as_i64};
use crate::{constants, httpfetch};
use chrono::serde::ts_milliseconds;
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type SpacecraftHashMap = HashMap<String, Spacecraft>;
pub type DishHashMap = HashMap<String, DsnConfigDish>;
pub type SiteHashMap = HashMap<String, Site>;

//////////////////
/// DSN Status
//////////////////

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename = "station")]
pub struct Station {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@friendlyName")]
    pub friendly_name: String,

    #[serde(rename = "@timeUTC", with = "ts_milliseconds")]
    pub time_utc: DateTime<Utc>,

    #[serde(rename = "@timeZoneOffset")]
    pub timezone_offset: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "upSignal")]
pub struct UpSignal {
    #[serde(rename = "@active")]
    pub active: bool,

    #[serde(rename = "@signalType")]
    pub signal_type: String,

    #[serde(rename = "@dataRate", with = "as_f32")]
    pub data_rate: f32,

    #[serde(rename = "@frequency", with = "as_i32")]
    pub frequency: i32,

    #[serde(rename = "@band")]
    pub band: String,

    #[serde(rename = "@power", with = "as_f32")]
    pub power: f32,

    #[serde(rename = "@spacecraft")]
    pub spacecraft: String,

    #[serde(rename = "@spacecraftID", with = "as_i32")]
    pub spacecraft_id: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "downSignal")]
pub struct DownSignal {
    #[serde(rename = "@active")]
    pub active: bool,

    #[serde(rename = "@signalType")]
    pub signal_type: String,

    #[serde(rename = "@dataRate", with = "as_f32")]
    pub data_rate: f32,

    #[serde(rename = "@frequency", with = "as_i32")]
    pub frequency: i32,

    #[serde(rename = "@band")]
    pub band: String,

    #[serde(rename = "@power", with = "as_f32")]
    pub power: f32,

    #[serde(rename = "@spacecraft")]
    pub spacecraft: String,

    #[serde(rename = "@spacecraftID", with = "as_i32")]
    pub spacecraft_id: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "target")]
pub struct Target {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@id", with = "as_i32")]
    pub id: i32,

    #[serde(rename = "@uplegRange", with = "as_i64")]
    pub upleg_range: i64,

    #[serde(rename = "@downlegRange", with = "as_i64")]
    pub downleg_range: i64,

    #[serde(rename = "@rtlt", with = "as_i32")]
    pub rtlt: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename = "dish")]
pub struct Dish {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@azimuthAngle", with = "as_i32")]
    pub azimuth_angle: i32,

    #[serde(rename = "@elevationAngle", with = "as_i32")]
    pub elevation_angle: i32,

    #[serde(rename = "@windSpeed", with = "as_i32")]
    pub wind_speed: i32,

    #[serde(rename = "@isMSPA")]
    pub is_mspa: bool,

    #[serde(rename = "@isArray")]
    pub is_array: bool,

    #[serde(rename = "@isDDOR")]
    pub is_ddor: bool,

    #[serde(rename = "@activity")]
    pub activity: String,

    #[serde(rename = "upSignal", default)]
    pub up_signal: Vec<UpSignal>,

    #[serde(rename = "downSignal", default)]
    pub down_signal: Vec<DownSignal>,

    #[serde(rename = "target", default)]
    pub target: Vec<Target>,
}

#[derive(Debug, Deserialize)]
pub enum DsnItem {
    #[serde(rename = "station")]
    Station(Station),
    #[serde(rename = "dish")]
    Dish(Dish),
    #[serde(other)]
    Other, // e.g. <timestamp>, so unknown tags don't error
}

#[derive(Debug, Deserialize)]
#[serde(rename = "dsn")]
pub struct Dsn {
    #[serde(rename = "$value", default)]
    pub items: Vec<DsnItem>,
}

//////////////////
/// DSN Config
//////////////////
/*
<config>
    <sites>
        <site name="mdscc" friendlyName="Madrid" longitude="-4.2480085" latitude="40.2413554">
            <dish name="DSS63" friendlyName="DSS 63" type="70M"></dish>
            <dish name="DSS65" friendlyName="DSS 65" type="34MHEF"></dish>
            <dish name="DSS53" friendlyName="DSS 53" type="34M"></dish>
            <dish name="DSS54" friendlyName="DSS 54" type="34M"></dish>
            <dish name="DSS55" friendlyName="DSS 55" type="34M"></dish>
            <dish name="DSS56" friendlyName="DSS 56" type="34M"></dish>
        </site>
    </sites>
    <spacecraftMap>
        <spacecraft name="ace" explorerName="sc_ace" friendlyName="Advanced Composition Explorer" thumbnail="true"></spacecraft>
        <spacecraft name="apm1" explorerName="" friendlyName="Astrobotic Peregrine 1 Mission" thumbnail="true"></spacecraft>
    </spacecraftMap>
</config>
*/

#[derive(Debug, Deserialize, Clone)]
#[serde(rename = "spacecraft")]
pub struct Spacecraft {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@explorerName")]
    pub explorer_name: String,

    #[serde(rename = "@friendlyName")]
    pub friendly_name: String,

    #[serde(rename = "@thumbnail")]
    pub thumbnail: Option<bool>,
}
#[derive(Debug, Deserialize)]
#[serde(rename = "spacecraftMap")]
pub struct SpacecraftMap {
    #[serde(rename = "spacecraft")]
    pub spacecraft: Vec<Spacecraft>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename = "dish")]
pub struct DsnConfigDish {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@friendlyName")]
    pub friendly_name: String,

    #[serde(rename = "@type")]
    pub dish_type: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename = "site")]
pub struct Site {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@friendlyName")]
    pub friendly_name: String,

    #[serde(rename = "@longitude")]
    pub longitude: f64,

    #[serde(rename = "@latitude")]
    pub latitude: f64,

    #[serde(rename = "dish")]
    pub dishes: Vec<DsnConfigDish>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename = "sites")]
pub struct Sites {
    #[serde(rename = "site")]
    pub sites: Vec<Site>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "config")]
pub struct DsnConfig {
    #[serde(rename = "sites")]
    pub sites: Sites,

    #[serde(rename = "spacecraftMap")]
    pub spacecraft_map: SpacecraftMap,
}

impl DsnConfig {
    pub fn create_spacecraft_hashmap(&self) -> SpacecraftHashMap {
        self.spacecraft_map
            .spacecraft
            .iter()
            .map(|s| (s.name.to_lowercase(), s.clone()))
            .collect()
    }

    pub fn create_dish_hashmap(&self) -> DishHashMap {
        self.sites
            .sites
            .iter()
            .flat_map(|site| {
                site.dishes
                    .iter()
                    .map(|d| (d.name.to_lowercase(), d.clone()))
            })
            .collect()
    }

    pub fn create_site_hashmap(&self) -> SiteHashMap {
        self.sites
            .sites
            .iter()
            .map(|s| (s.name.to_lowercase(), s.clone()))
            .collect()
    }
}

pub async fn fetch_dsn_status() -> Result<Dsn> {
    let req = httpfetch::HttpFetcher::new(constants::url::DSN_STATUS)?;

    let response_string = match req.into_string().await {
        Ok(response_string) => response_string.text,
        Err(why) => return Err(anyhow!("Remote error fetching dsn status: {:?}", why)),
    };

    let status: Dsn = match from_str(&response_string) {
        Ok(res) => res,
        Err(why) => return Err(anyhow!("Error parsing DSN status XML: {:?}", why)),
    };

    Ok(status)
}

pub async fn fetch_dsn_config() -> Result<DsnConfig> {
    let req = httpfetch::HttpFetcher::new(constants::url::DSN_CONFIG)?;

    let response_string = match req.into_string().await {
        Ok(response_string) => response_string.text,
        Err(why) => return Err(anyhow!("Remote error fetching dsn config: {:?}", why)),
    };

    let dsn_config: DsnConfig = match from_str(&response_string) {
        Ok(res) => res,
        Err(why) => return Err(anyhow!("Error parsing DSN config XML: {:?}", why)),
    };

    Ok(dsn_config)
}
