
pub const DEFAULT_RED_WEIGHT : f32 = 1.0;
pub const DEFAULT_GREEN_WEIGHT : f32 = 1.0;
pub const DEFAULT_BLUE_WEIGHT : f32 = 1.0;

pub const OUTPUT_FILENAME_APPEND : &str = "rjcal";

pub mod url {
    pub const MSL_RAW_WEBSERVICE_URL : &str = "https://mars.nasa.gov/api/v1/raw_image_items/";
    pub const MSL_LATEST_WEBSERVICE_URL : &str = "https://mars.nasa.gov/api/v1/raw_image_items/msl/latest/";
    pub const M20_RAW_WEBSERVICE_URL : &str = "https://mars.nasa.gov/rss/api/";
    pub const M20_LATEST_WEBSERVICE_URL : &str = "https://mars.nasa.gov/rss/api/?feed=raw_images&category=mars2020,ingenuity&feedtype=json&ver=1.2&latest=true";
    pub const NSYT_RAW_WEBSERVICE_URL : &str = "https://mars.nasa.gov/api/v1/raw_image_items/";
    pub const NSYT_LATEST_WEBSERVICE_URL : &str = "https://mars.nasa.gov/api/v1/raw_image_items/insight/latest/";
}

pub mod time {
    pub const MSL_SURFACE_SCLK: f64 = 434589485.0;
    pub const MSL_UNIX_COUNT_OFFSET: f64 = 1381317960.0;
    pub const MSL_SURFACE_SEC_OFFSET : f64 = 1344174599.0;
    pub const MSL_RATE_ADJUSTMENT: f64 = 1.000009438;

    pub const MSL_LONGITUDE: f64 = 137.4417;
    pub const MSL_SOL_OFFSET: f64 = -49268.0;

    // These values need validation
    pub const M2020_LONGITUDE: f64 = 77.43;
    pub const M2020_SOL_OFFSET: f64 = -52303.0;

    pub const NSYT_LONGITUDE: f64 = 135.6234;
    pub const NSYT_SOL_OFFSET: f64 = -51510.0;
    
    pub const LEAP_SEC : f64 = 2.0;
    pub const MARS_SEC_ADJUSTMENT : f64 = 1.02749125;
    pub const TAI_OFFSET: f64 = 37.0;
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
    pub const INVALID_FLOAT_VALUE: &str = "Invalid float value";
    pub const UNSUPPORTED_INSTRUMENT: &str = "Unsupported instrument";
    pub const EVEN_NUMBER_REQUIRED: &str = "Value error: Even number required";
    pub const REMOTE_SERVER_ERROR: &str = "Remote server error";
    pub const YES : &str = "Yes";
    pub const NO : &str = "No";
    pub const DOWNLOADING : &str = "Downloading";
    pub const INVALID_CALIBRATION_FILE_ID : &str = "Invalid calibration file";
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

    pub const PARAM_LEVELS_WHITE_LEVEL : &str = "whitelevel";
    pub const PARAM_LEVELS_WHITE_LEVEL_SHORT : &str = "w";

    pub const PARAM_LEVELS_BLACK_LEVEL : &str = "blacklevel";
    pub const PARAM_LEVELS_BLACK_LEVEL_SHORT : &str = "b";

    pub const PARAM_GAMMA : &str = "gamma";
    pub const PARAM_GAMMA_SHORT : &str = "g";

    pub const PARAM_DELAY : &str = "delay";
    pub const PARAM_DELAY_SHORT : &str = "d";
    
    pub const PARAM_LOWPASS : &str = "lowpass";
    pub const PARAM_LOWPASS_SHORT : &str = "l";

    pub const PARAM_PRODUCT_TYPE : &str = "prodtype";
    pub const PARAM_PRODUCT_TYPE_SHORT : &str = "p";
}


