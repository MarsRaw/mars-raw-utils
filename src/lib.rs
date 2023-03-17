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

/// Routines for creating stereo anaglyph images
pub mod anaglyph;

/// Support for calibration file loading
pub mod calibfile;

/// Calibration entrypoint
pub mod calibrate;

/// Support for calibration specification profiles
pub mod calprofile;

/// Image linearization and mosaic compositing
pub mod composite;

/// Constant values
pub mod constants;

/// Pixel value decompanding/decompression
pub mod decompanding;

/// Image color channel decorrelation
pub mod decorr;

/// Processing for ENV sequence change detection/dust devils.
pub mod diffgif;

/// Extensions for `RgbImage` to add basic 2d polygon rendering
pub mod drawable;

/// Basic enumerations
pub mod enums;

/// Image flat field processing
pub mod flatfield;

/// Focus stack processing
pub mod focusmerge;

/// Remote data retrieval via HTTP
pub mod httpfetch;

/// Extensions to `RgbImage` to support Mars mission image data
pub mod image;

/// Image inpainting
pub mod inpaintmask;

/// JSON API interactions
pub mod jsonfetch;

/// Routines for Mars2020 Perseverance/Ingenuity processing
pub mod m20;

/// Routines for Mars Exploration Rover Opportunity/Spirit processing
pub mod mer;

/// Base image metadata structures and parsing
pub mod metadata;

/// Routines for Mars Science Laboratory Curiosity Rover processing
pub mod msl;

/// Routines for InSight image processing
pub mod nsyt;

/// Filesystem utilities
pub mod path;

/// Single-point import for most utilized MRU API
pub mod prelude;

/// Utilities for outputting verbose and error text
pub mod print;

/// Time and date support
pub mod time;

/// General utilities
pub mod util;
