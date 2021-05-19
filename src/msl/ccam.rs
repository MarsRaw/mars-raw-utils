use crate::{
    vprintln, 
    rgbimage, 
    enums, 
    path,
    calibfile,
    imagebuffer
};


pub fn process_file(input_file:&str, only_new:bool) {
    let out_file = input_file.replace(".png", "-rjcal.png").replace(".PNG", "-rjcal.png")
                             .replace(".jpg", "-rjcal.png").replace(".JPG", "-rjcal.png");
    if path::file_exists(&out_file) && only_new {
        vprintln!("Output file exists, skipping. ({})", out_file);
        return;
    }

    let mut raw = rgbimage::RgbImage::open(String::from(input_file), enums::Instrument::MslChemCam).unwrap();

    vprintln!("Loading image mask");
    let mask = imagebuffer::ImageBuffer::from_file(calibfile::get_calibration_file_for_instrument(enums::Instrument::MslChemCam, enums::CalFileType::Mask).unwrap().as_str()).unwrap();
    raw.apply_mask(&mask);

    let data_max = 255.0;

    if input_file.find("EDR") != None {
        vprintln!("Image appears to be in standard contrast");
        
        vprintln!("Flatfielding...");
        raw.flatfield().unwrap();

    } else if input_file.find("EDR") != None {
        vprintln!("Image appears to be in enhanced contrast");
        // ... Don't do flatfielding, these appear to already been applied.
        // ... Do something about that
    }

    vprintln!("Normalizing...");
    raw.normalize_to_16bit_with_max(data_max).unwrap();

    vprintln!("Writing to disk...");
    raw.save(&out_file).unwrap();

}