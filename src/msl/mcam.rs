use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    path,
    util,
    decompanding,
    constants
};

pub fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, color_noise_reduction:i32, no_ilt:bool, only_new:bool) {
    let out_file = util::append_file_name(input_file, constants::OUTPUT_FILENAME_APPEND);
    if path::file_exists(&out_file) && only_new {
        vprintln!("Output file exists, skipping. ({})", out_file);
        return;
    }

    let mut instrument = enums::Instrument::MslMastcamLeft;

    if util::filename_char_at_pos(&input_file, 5) == 'R' {
        instrument = enums::Instrument::MslMastcamRight;
        vprintln!("Processing for Mastcam Right");
    } else {
        vprintln!("Processing for Mastcam Left") ;
    }

    let mut raw = MarsImage::open(String::from(input_file), instrument);

    let mut data_max = 255.0;
    
    if ! no_ilt {
        vprintln!("Decompanding...");
        raw.decompand(&decompanding::get_ilt_for_instrument(instrument));
        data_max = decompanding::get_max_for_instrument(instrument) as f32;
    }

    if /*util::filename_char_at_pos(&input_file, 22) == 'E' &&*/ raw.image.is_grayscale() {
        vprintln!("Image appears to be grayscale, applying debayering...");
        raw.debayer();
    }


    if raw.image.width == 1536 {
        raw.image.crop(161, 0, 1328, raw.image.height);
    }

    // Only inpaint with the same size as the mask until we can reliably determine
    // subframing sensor location.
    if raw.image.width == 1328 && raw.image.height == 1184 {
        vprintln!("Inpainting...");
        raw.apply_inpaint_fix();
    }
    
    vprintln!("Flatfielding...");
    raw.flatfield();

    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar);

    if color_noise_reduction > 0 {
        vprintln!("Color noise reduction...");
        raw.image.reduce_color_noise(color_noise_reduction);
    }
    
    vprintln!("Normalizing...");
    raw.image.normalize_to_16bit_with_max(data_max);

    vprintln!("Cropping...");
    raw.image.crop(3, 3, raw.image.width - 6, raw.image.height - 6);


    vprintln!("Writing to disk...");
    raw.save(&out_file);
}

