
pub const DEFAULT_RED_WEIGHT : f32 = 1.0;
pub const DEFAULT_GREEN_WEIGHT : f32 = 1.0;
pub const DEFAULT_BLUE_WEIGHT : f32 = 1.0;

pub mod url {
    pub const MSL_RAW_WEBSERVICE_URL : &str = "https://mars.nasa.gov/api/v1/raw_image_items/";
    pub const M20_RAW_WEBSERVICE_URL : &str = "https://mars.nasa.gov/rss/api/";
}

pub mod time {
    pub const MSL_SURFACE_SCLK: f32 = 434589485.0;
    pub const MSL_UNIX_COUNT_OFFSET: f32 = 1381317960.0;
    pub const MSL_SURFACE_SEC_OFFSET : f32 = 1344174599.0;

    pub const RATE_ADJUSTMENT: f32 = 1.000009438;
    pub const LEAP_SEC : f32 = 2.0;
    pub const MARS_SEC_ADJUSTMENT : f32 = 1.02749125;
}

// Strings
pub mod status {
    pub const EMPTY : &str = "";
    pub const OK : &str = "ok";
    pub const STRUCT_IS_EMPTY : &str = "Structure is empty";
    pub const INVALID_PIXEL_COORDINATES : &str = "Invalid pixel coordinates";
    pub const PARENT_NOT_EXISTS_OR_UNWRITABLE : &str = "Parent does not exist or cannot be written";
    pub const FILE_NOT_FOUND: &str = "File not found";
    pub const ARRAY_SIZE_MISMATCH : &str = "Array size mismatch";
    pub const NOT_IMPLEMENTED : &str = "Not yet implemented";
    pub const DIMENSIONS_DO_NOT_MATCH_VECTOR_LENGTH : &str = "Image dimensions do not match supplied vector length";
    pub const ERROR_PARSING_JSON: &str = "Error parsing JSON";
    pub const INVALID_ENUM_VALUE: &str = "Invalid enum value";
    pub const INVALID_RAW_VALUE: &str = "Invalid raw image value";
    pub const UNSUPPORTED_INSTRUMENT: &str = "Unsupported instrument";
    pub const EVEN_NUMBER_REQUIRED: &str = "Value error: Even number required";
    pub const REMOTE_SERVER_ERROR: &str = "Remote server error";
    pub const YES : &str = "Yes";
    pub const NO : &str = "No";
    pub const DOWNLOADING : &str = "Downloading";
}


// Parameters
pub mod param {
    pub const PARAM_VERBOSE : &str = "v";
    pub const PARAM_OUTPUT : &str = "output";
    pub const PARAM_OUTPUT_SHORT : &str = "o";
    pub const PARAM_INPUTS : &str = "inputs";
    pub const PARAM_INPUTS_SHORT : &str = "i";

    pub const PARAM_RED_WEIGHT : &str = "red";
    pub const PARAM_RED_WEIGHT_SHORT : &str = "R";

    pub const PARAM_GREEN_WEIGHT : &str = "green";
    pub const PARAM_GREEN_WEIGHT_SHORT : &str = "G";

    pub const PARAM_BLUE_WEIGHT : &str = "blue";
    pub const PARAM_BLUE_WEIGHT_SHORT : &str = "B";

    pub const PARAM_COLOR_NOISE_REDUCTION : &str = "color_noise_reduction";
    pub const PARAM_COLOR_NOISE_REDUCTION_SHORT : &str = "c";

    // Don't apply ILT
    pub const PARAM_RAW_COLOR : &str = "raw";
    pub const PARAM_RAW_COLOR_SHORT : &str = "r";

    // Hot pixel correction threshold
    pub const PARAM_HPC_THRESHOLD : &str = "hpc_threshold";
    pub const PARAM_HPC_THRESHOLD_SHORT : &str = "t";

    // Hot pixel correction window size
    pub const PARAM_HPC_WINDOW_SIZE : &str = "hpc_window";
    pub const PARAM_HPC_WINDOW_SIZE_SHORT : &str = "w";

    pub const PARAM_ONLY_NEW : &str = "new";
    pub const PARAM_ONLY_NEW_SHORT : &str = "n";

    pub const PARAM_SCALE_FACTOR : &str = "factor";
    pub const PARAM_SCALE_FACTOR_SHORT : &str = "f";
}

pub mod cal {

    // These can't stay hard coded like this...
    pub const M20_INPAINT_MASK_RIGHT_PATH : &str = "{DATADIR}/M20_MCZ_RIGHT_INPAINT_MASK_V1.png";
    pub const M20_INPAINT_MASK_LEFT_PATH : &str = "{DATADIR}/M20_MCZ_LEFT_INPAINT_MASK_V1.png";

    pub const MSL_MAHLI_INPAINT_MASK_PATH : &str = "{DATADIR}/MSL_MAHLI_INPAINT_Sol2904_V1.png";
    pub const MSL_MAHLI_FLAT_PATH : &str = "{DATADIR}/MSL_MAHLI_FLAT_Sol2904_V1.png";

    pub const M20_WATSON_INPAINT_MASK_PATH : &str = "{DATADIR}/M20_WATSON_INPAINT_MASK_V1.png";
    pub const M20_WATSON_FLAT_PATH : &str = "{DATADIR}/M20_WATSON_FLAT_V0.png";

    pub const M20_SCAM_FLAT_BAYER_PATH : &str = "{DATADIR}/M20_SCAM_FLAT_BAYER_Sol77_V2.png";
    pub const M20_SCAM_FLAT_RGB_PATH : &str = "{DATADIR}/M20_SCAM_FLAT_RGB_Sol77_V2.png";
    pub const M20_SCAM_MASK_PATH : &str = "{DATADIR}/M20_SCAM_MASK_Sol1_V1.png";

    // Limiting to navcams on RCE-B
    pub const MSL_NCAM_RIGHT_INPAINT_PATH : &str = "{DATADIR}/MSL_NRB_INPAINT_Sol3052_V1.png";
    pub const MSL_NCAM_RIGHT_FLAT_PATH : &str = "{DATADIR}/MSL_NRB_FLAT_V1.png";
    pub const MSL_NCAM_LEFT_FLAT_PATH : &str = "{DATADIR}/MSL_NLB_FLAT_V1.png";

    pub const MSL_FHAZ_RIGHT_FLAT_PATH : &str = "{DATADIR}/MSL_FRB_FLAT_V1.png";
    pub const MSL_FHAZ_LEFT_FLAT_PATH : &str = "{DATADIR}/MSL_FLB_FLAT_V1.png";

    pub const MSL_RHAZ_RIGHT_FLAT_PATH : &str = "{DATADIR}/MSL_RRB_FLAT_V1.png";
    pub const MSL_RHAZ_LEFT_FLAT_PATH : &str = "{DATADIR}/MSL_RLB_FLAT_V1.png";

    pub const MSL_MCAM_LEFT_INPAINT_PATH : &str = "{DATADIR}/MSL_MCAM_LEFT_INPAINT_Sol3082_V1.png";
    pub const MSL_MCAM_RIGHT_INPAINT_PATH : &str = "{DATADIR}/MSL_MCAM_RIGHT_INPAINT_Sol3101_V1.png";
}

