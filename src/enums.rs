

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
    MslMARDI,
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
    M20Pixl,
    M20HeliNav,
    M20HeliRte,
    NsytICC,
    NsytIDC,
    None
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CalFileType {
    FlatField,
    InpaintMask,
    Mask
}




