// #![feature(associated_type_bounds)]

use reqwest::Client;
use std::time::Duration;
extern crate clap;

lazy_static! {
    pub static ref CLIENT: reqwest::Client =
        Client::builder().timeout(Duration::from_secs(60)).build().expect("Error creating client, this is most unexpected and is likely a mistake by a developer working on this project.");
}

#[macro_use]
extern crate lazy_static;

pub mod anaglyph;
pub mod calibfile;
pub mod calibrate;
pub mod calprofile;
pub mod composite;
pub mod constants;
pub mod decompanding;
pub mod diffgif;
pub mod drawable;
pub mod enums;
pub mod flatfield;
pub mod focusmerge;
pub mod httpfetch;
pub mod image;
pub mod inpaintmask;
pub mod jsonfetch;
pub mod m20;
pub mod mer;
pub mod metadata;
pub mod msl;
pub mod nsyt;
pub mod path;
pub mod prelude;
pub mod print;
pub mod time;
pub mod util;
