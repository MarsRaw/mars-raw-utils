use crate::{
    vprintln, 
    rgbimage, 
    enums, 
    path, 
    decompanding
};


pub fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, no_ilt:bool, only_new:bool) {
    let out_file = input_file.replace(".jpg", "-rjcal.png").replace(".JPG", "-rjcal.png");
    if path::file_exists(&out_file) && only_new {
        vprintln!("Output file exists, skipping. ({})", out_file);
        return;
    }

    let mut raw = rgbimage::RgbImage::open(String::from(input_file), enums::Instrument::MslMAHLI).unwrap();

    if raw.width == 1632 && raw.height == 1200 {
        vprintln!("Cropping...");
        raw.crop(32, 16, 1584, 1184).unwrap();
    }
    
    vprintln!("Inpainting...");
    raw.apply_inpaint_fix().unwrap();

    let mut data_max = 255.0;

    if ! no_ilt {
        vprintln!("Decompanding...");
        raw.decompand().unwrap();
        data_max = decompanding::get_max_for_instrument(enums::Instrument::MslMAHLI) as f32;
    }

    vprintln!("Flatfielding...");
    raw.flatfield().unwrap();

    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar).unwrap();

    vprintln!("Normalizing...");
    raw.normalize_to_16bit_with_max(data_max).unwrap();

    vprintln!("Writing to disk...");
    raw.save(&out_file).unwrap();
}