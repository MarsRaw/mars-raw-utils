use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    path, 
    decompanding,
    util,
    constants,
    flatfield
};


pub fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, no_ilt:bool, only_new:bool) {
    let out_file = util::append_file_name(input_file, constants::OUTPUT_FILENAME_APPEND);
    if path::file_exists(&out_file) && only_new {
        vprintln!("Output file exists, skipping. ({})", out_file);
        return;
    }

    let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::MslMAHLI);

    if raw.image.width == 1632 && raw.image.height == 1200 {
        vprintln!("Cropping...");
        raw.image.crop(32, 16, 1584, 1184);
    } else if raw.image.width == 1648 && raw.image.height == 1200 {
        vprintln!("Cropping...");
        raw.image.crop(48, 16, 1584, 1184);
    }
    vprintln!("Image width/height after cropping: {}x{}", raw.image.width, raw.image.height);

    //1648, 1200
    vprintln!("Inpainting...");
    raw.apply_inpaint_fix();

    let mut data_max = 255.0;

    if ! no_ilt {
        vprintln!("Decompanding...");
        raw.decompand(&decompanding::get_ilt_for_instrument(enums::Instrument::MslMAHLI));
        data_max = decompanding::get_max_for_instrument(enums::Instrument::MslMAHLI) as f32;
    }

    vprintln!("Flatfielding...");
    let mut flat = flatfield::load_flat(enums::Instrument::MslMAHLI).unwrap();
    if flat.image.width == 1632 && flat.image.height == 1200 {
        flat.image.crop(32, 16, 1584, 1184);
    } 
    flat.apply_inpaint_fix();

    if flat.image.width > raw.image.width {
        let x = (flat.image.width - raw.image.width) / 2;
        let y = (flat.image.height - raw.image.height) / 2;
        vprintln!("Cropping flat with x/y/width/height: {},{} {}x{}", x, y, raw.image.width, raw.image.height);
        flat.image.crop(x, y, raw.image.width, raw.image.height);
    }

    raw.flatfield_with_flat(&flat);
    

    vprintln!("Cropping...");
    raw.image.crop(0, 3, 1584, 1180);

    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar);

    vprintln!("Normalizing...");
    raw.image.normalize_to_16bit_with_max(data_max);

    vprintln!("Writing to disk...");
    raw.save(&out_file);
}