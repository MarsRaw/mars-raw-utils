use std::num::ParseIntError;
use std::str::FromStr;

// Supported missions
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Mission {
    MSL,      // Mars Science Laboratory - Curiosity Rover
    Mars2020, // Perseverance Rover
    InSight,  // InSight Lander
    MerA,     // Mars Exploration Rovers - Spirit Rover
    MerB,     // Mars Exploration Rovers - Opportunity Rover
}

/// Representation of left/right side of a stereo image with an option to simply not care (or unknown).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Eye {
    Right,
    Left,
    DontCare,
}

// Supported instruments
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
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
    M20SkyCam,
    M20HeliNav,
    M20HeliRte,
    M20SherlocAci,
    M20CacheCam,
    M20EdlRdcam,
    NsytICC,
    NsytIDC,
    #[default]
    None,
}

impl FromStr for Instrument {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Instrument, ParseIntError> {
        Ok(match s.to_uppercase().as_str() {
            "MCZ_LEFT" => Instrument::M20MastcamZLeft,
            "MCZ_RIGHT" => Instrument::M20MastcamZRight,
            "FRONT_HAZCAM_LEFT_A" | "FRONT_HAZCAM_LEFT_B" => Instrument::M20FrontHazLeft,
            "FRONT_HAZCAM_RIGHT_A" | "FRONT_HAZCAM_RIGHT_B" => Instrument::M20FrontHazRight,
            "REAR_HAZCAM_LEFT" => Instrument::M20RearHazLeft,
            "REAR_HAZCAM_RIGHT" => Instrument::M20RearHazRight,
            "NAVCAM_LEFT" => Instrument::M20NavcamLeft,
            "NAVCAM_RIGHT" => Instrument::M20NavcamRight,
            "SHERLOC_WATSON" => Instrument::M20Watson,
            "HELI_NAV" => Instrument::M20HeliNav,
            "HELI_RTE" => Instrument::M20HeliRte,
            "PIXL_MCC" => Instrument::M20Pixl,
            "SKYCAM" => Instrument::M20SkyCam,
            "SUPERCAM_RMI" => Instrument::M20SuperCam,
            "SHERLOC_ACI" => Instrument::M20SherlocAci,
            "CACHECAM" => Instrument::M20CacheCam,
            "EDL_RDCAM" => Instrument::M20EdlRdcam,
            "MAST_LEFT" => Instrument::MslMastcamLeft,
            "MAST_RIGHT" => Instrument::MslMastcamRight,
            "MAHLI" => Instrument::MslMAHLI,
            "MARDI" => Instrument::MslMARDI,
            "FHAZ_LEFT_A" | "FHAZ_LEFT_B" => Instrument::MslFrontHazLeft,
            "FHAZ_RIGHT_A" | "FHAZ_RIGHT_B" => Instrument::MslFrontHazRight,
            "RHAZ_LEFT_A" | "RHAZ_LEFT_B" => Instrument::MslRearHazLeft,
            "RHAZ_RIGHT_A" | "RHAZ_RIGHT_B" => Instrument::MslRearHazRight,
            "NAV_LEFT_A" | "NAV_LEFT_B" => Instrument::MslNavCamLeft,
            "NAV_RIGHT_A" | "NAV_RIGHT_B" => Instrument::MslNavCamRight,
            "CHEMCAM_RMI" => Instrument::MslChemCam,

            "IDC" => Instrument::NsytIDC,
            "ICC" => Instrument::NsytICC,
            _ => Instrument::None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CalFileType {
    FlatField,
    InpaintMask,
    Mask,
    Lut,
}
