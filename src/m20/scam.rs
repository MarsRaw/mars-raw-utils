use crate::{
    vprintln, 
    rgbimage, 
    enums, 
    path, 
    imagebuffer,
    calibfile
};


pub fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, _no_ilt:bool, only_new:bool) {
    let out_file = input_file.replace(".jpg", "-rjcal.png").replace(".JPG", "-rjcal.png")
                             .replace(".png", "-rjcal.png").replace(".PNG", "-rjcal.png");
    if path::file_exists(&out_file) && only_new {
        vprintln!("Output file exists, skipping. ({})", out_file);
        return;
    }
    
    let mut raw = rgbimage::RgbImage::open(String::from(input_file), enums::Instrument::M20SuperCam).unwrap();
    
    vprintln!("Loading image mask");
    let mask = imagebuffer::ImageBuffer::from_file(calibfile::get_calibration_file_for_instrument(enums::Instrument::M20SuperCam, enums::CalFileType::Mask).unwrap().as_str()).unwrap();
    raw.apply_mask(&mask);

    let data_max = 255.0;

    
    if input_file.find("ECM") != None && raw.is_grayscale() {
        vprintln!("Image appears to be grayscale, applying debayering...");
        raw.debayer().unwrap();
    }

    // Gonna start with standard rectangular flat field, but should really
    // mask it to just the round light-collecting area of the image.
    vprintln!("Flatfielding...");
    raw.flatfield().unwrap();

    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar).unwrap();

    vprintln!("Normalizing...");
    raw.normalize_to_16bit_with_max(data_max).unwrap();

    vprintln!("Writing to disk...");
    raw.save(&out_file).unwrap();
}


