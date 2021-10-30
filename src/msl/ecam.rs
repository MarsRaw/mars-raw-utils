use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    path,
    util,
    inpaintmask,
    constants
};


// Doesn't support subframed images yet since we won't know what part of the sensor was
// used from the raws alone. If it's in the JSON response from the raw image site, then
// maybe I can embed that data into the jpegs (EXIF) when downloaded using msl_fetch_raws
// and trigger off of that. Still need to think of times when someone downloads the image
// directly from the webpage using their browser as the website often prepends a wonky
// prefix to the image filename.
//
// Also leaving in the ILT parameter until I iron out the cases in which it's needed
// for ECAM. 
pub fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, _no_ilt:bool, hpc_threshold:f32, only_new:bool) {
    let out_file = util::append_file_name(input_file, constants::OUTPUT_FILENAME_APPEND);
    if path::file_exists(&out_file) && only_new {
        vprintln!("Output file exists, skipping. ({})", out_file);
        return;
    }

    let mut instrument = enums::Instrument::MslNavCamRight;

    // Attempt to figure out camera from file name
    if util::filename_char_at_pos(&input_file, 0) == 'N' {         // NAVCAMS
        if util::filename_char_at_pos(&input_file, 1) == 'L' {     // Left
            instrument = enums::Instrument::MslNavCamLeft;
        } else {                                   // Assume Right
            instrument = enums::Instrument::MslNavCamRight;
        }
    } else if util::filename_char_at_pos(&input_file, 0) == 'F' {  // FHAZ
        if util::filename_char_at_pos(&input_file, 1)  == 'L' {     // Left
            instrument = enums::Instrument::MslFrontHazLeft;
        } else {                                   // Assume Right
            instrument = enums::Instrument::MslFrontHazRight;
        }  
    } else if util::filename_char_at_pos(&input_file, 0) == 'R' {  // RHAZ
        if util::filename_char_at_pos(&input_file, 1)  == 'L' {     // Left
            instrument = enums::Instrument::MslRearHazLeft;
        } else {                                   // Assume Right
            instrument = enums::Instrument::MslRearHazRight;
        }
    }

    let mut raw = MarsImage::open(String::from(input_file), instrument);

    // Exclude subframed images for now...
    if inpaintmask::inpaint_supported_for_instrument(instrument) && raw.image.height >= 1022 {
        vprintln!("Inpainting...");
        raw.apply_inpaint_fix();
    } else {
        vprintln!("Inpainting not supported for instrument {:?}", instrument);
    }

    if hpc_threshold > 0.0 {
        vprintln!("Hot pixel correction with variance threshold {}...", hpc_threshold);
        raw.hot_pixel_correction(3, hpc_threshold);
    }
    
    let data_max = 255.0;

    // if ! no_ilt {
    //     vprintln!("Decompanding...");
    //     raw.decompand().unwrap();
    //     data_max = decompanding::get_max_for_instrument(instrument) as f32;
    // }
    
    // Exclude subframed images for now...
    if raw.image.height >= 1022 {
        vprintln!("Flatfielding...");
        raw.flatfield();
    }
    
    
    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar);

    vprintln!("Normalizing...");
    raw.image.normalize_to_16bit_with_max(data_max);

    // Trim off border pixels
    let crop_to_width = raw.image.width - 2;
    let crop_to_height = raw.image.height - 2;
    raw.image.crop(1, 1, crop_to_width, crop_to_height);

    vprintln!("Writing to disk...");
    raw.save(&out_file);
}
