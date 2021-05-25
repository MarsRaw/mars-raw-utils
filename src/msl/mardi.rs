use crate::{
    vprintln, 
    rgbimage, 
    enums, 
    path,
    decompanding,
    constants,
    util
};

pub fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, no_ilt:bool, only_new:bool) {
    let out_file = util::append_file_name(input_file, constants::OUTPUT_FILENAME_APPEND);
    if path::file_exists(&out_file) && only_new {
        vprintln!("Output file exists, skipping. ({})", out_file);
        return;
    }

    let mut raw = rgbimage::RgbImage::open(String::from(input_file), enums::Instrument::MslMARDI).unwrap();

    let mut data_max = 255.0;

    if ! no_ilt {
        vprintln!("Decompanding...");
        raw.decompand().unwrap();
        data_max = decompanding::get_max_for_instrument(enums::Instrument::MslMARDI) as f32;
    }

    vprintln!("Flatfielding...");
    raw.flatfield().unwrap();
    
    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar).unwrap();

    vprintln!("Cropping...");
    raw.crop(24, 6, 1599, 1188).unwrap();

    vprintln!("Normalizing...");
    raw.normalize_to_16bit_with_max(data_max).unwrap();

    vprintln!("Writing to disk...");
    raw.save(&out_file).unwrap();
}