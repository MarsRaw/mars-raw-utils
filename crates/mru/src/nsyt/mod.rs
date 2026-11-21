/// Calibration routines for the Instrument Context Camera (ICC)
pub mod icc;

/// Calibration routines for the Instrument Deployment Camera (IDC)
pub mod idc;

/// Support for parsing the M20 public API metadata
pub mod metadata;

/// Support for calculating realtime mission time
pub mod missiontime;

/// Support for retrieving images from the public raw image API
pub mod fetch;
