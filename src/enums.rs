

// Supported missions
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mission {
    MSL,
    MARS2020,
    INSIGHT
}

// Supported instruments
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instrument {
    MslMAHLI,
    MslMastcamLeft,
    MslMastcamRight,
    MslNavCamRight, // Limiting to RCE-B camera for ECAM. For now.
    MslNavCamLeft,
    MslFrontHazLeft,
    MslFrontHazRight,
    MslRearHazLeft,
    MslRearHazRight,
    MslChemCam,
    M20MastcamZLeft,
    M20MastcamZRight,
    M20NavcamLeft,
    M20NavcamRight,
    M20FrontHazLeft,
    M20FrontHazRight,
    M20RearHazLeft,
    M20RearHazRight,
    M20Watson,
    M20SuperCam,
    NsytICC,
    NsytIDC,
    None
}

// Image data value range. Doesn't enforce actual
// value data types in the structs
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ImageMode {
    U8BIT,
    U12BIT,
    U16BIT
}



impl ImageMode {

    pub fn maxvalue(mode:ImageMode) -> f32 {
        match mode {
            ImageMode::U8BIT => 255.0,
            ImageMode::U12BIT => 2033.0, // In terms of the ILT
            ImageMode::U16BIT => 65535.0
        }
    }
}