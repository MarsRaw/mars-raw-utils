
pub const _16_BIT_MAX : f32 = std::u16::MAX as f32;

pub const STRIP_HEIGHT : usize = 128;
pub const STRIP_WIDTH : usize = 1648;

pub const DEFAULT_RED_WEIGHT : f32 = 0.902;
pub const DEFAULT_GREEN_WEIGHT : f32 = 1.0;
pub const DEFAULT_BLUE_WEIGHT : f32 = 1.8889;

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
}


// Parameters
pub mod param {
    pub const PARAM_VERBOSE : &str = "v";
    pub const PARAM_OUTPUT : &str = "output";
    pub const PARAM_OUTPUT_SHORT : &str = "o";
    pub const PARAM_DARK : &str = "dark";
    pub const PARAM_DARK_SHORT : &str = "d";
    pub const PARAM_FLAT : &str = "flat";
    pub const PARAM_FLAT_SHORT : &str = "f";
    pub const PARAM_INPUTS : &str = "inputs";
    pub const PARAM_INPUTS_SHORT : &str = "i";
}

pub mod cal {

    // These can't stay hard coded like this...
    pub const M20_INPAINT_MASK_RIGHT_PATH : &str = "src/cal/M20_MCZ_RIGHT_INPAINT_MASK_V1.png";
    pub const M20_INPAINT_MASK_LEFT_PATH : &str = "src/cal/M20_MCZ_LEFT_INPAINT_MASK_V1.png";

    pub const MSL_MAHLI_INPAINT_MASK_PATH : &str = "src/cal/MSL_MAHLI_INPAINT_Sol2904_V1.png";
    pub const MSL_MAHLI_FLAT_PATH  : &str = "src/cal/MSL_MAHLI_FLAT_Sol2904_V1.png";

}

