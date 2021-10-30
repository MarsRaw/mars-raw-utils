use crate::{
    vprintln, 
    image::MarsImage, 
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

    let mut instrument = enums::Instrument::M20MastcamZLeft;

    let bn = path::basename(&input_file);
    if bn.chars().nth(1).unwrap() == 'R' {
        instrument = enums::Instrument::M20MastcamZRight;
        vprintln!("Processing for Mastcam-Z Right");
    } else {
        vprintln!("Processing for Mastcam-Z Left") ;
    }
    
    let mut raw = MarsImage::open(String::from(input_file), instrument);

    let mut data_max = 255.0;

    if ! no_ilt {
        vprintln!("Decompanding...");
        raw.decompand(&decompanding::get_ilt_for_instrument(instrument));
        data_max = decompanding::get_max_for_instrument(instrument) as f32;
    }

    // Looks like 'ECM' in the name seems to indicate that it still have the bayer pattern
    // Update: Not always. Added a check to determine whether or not is is grayscale.
    // It's not perfect so please validate results. Gonna keep the 'ECM' check for now.
    if input_file.find("ECM") != None && raw.image.is_grayscale() {
        vprintln!("Image appears to be grayscale, applying debayering...");
        raw.debayer();
    }





    vprintln!("Inpainting...");
    raw.apply_inpaint_fix();

    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar);

    vprintln!("Normalizing...");
    raw.image.normalize_to_16bit_with_max(data_max);

    if raw.image.width == 1648 {
        vprintln!("Cropping...");
        raw.image.crop(24, 4, 1600, 1192);
    }

    vprintln!("Writing to disk...");
    
    raw.save(&out_file);
}
