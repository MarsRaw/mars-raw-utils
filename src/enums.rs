

// Supported missions
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mission {
    MSL,
    MARS2020
}

// Supported instruments
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instrument {
    MslMAHLI,
    MslMastcamLeft,
    MslMastcamRight,
    M20MastcamZLeft,
    M20MastcamZRight,
    None
}


