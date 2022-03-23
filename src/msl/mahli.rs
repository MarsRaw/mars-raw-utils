use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    path, 
    decompanding,
    util,
    flatfield,
    calprofile
};

pub fn process_with_profiles(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, no_ilt:bool, only_new:bool, filename_suffix:&String, profile_names:&Vec<&str>) {

    if profile_names.len() > 0 {
        for f in profile_names.iter() {
            process_with_profile(input_file, red_scalar, green_scalar, blue_scalar, no_ilt, only_new, filename_suffix, Some(&f.to_string()));
        }
    } else {
        process_with_profile(input_file, red_scalar, green_scalar, blue_scalar, no_ilt, only_new, filename_suffix, None);
    }

}

pub fn process_with_profile(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, no_ilt:bool, only_new:bool, filename_suffix:&String, profile_name_opt:Option<&String>) {

    if let Some(profile_name) = profile_name_opt {

        match calprofile::load_calibration_profile(&profile_name.to_string()) {
            Ok(profile) => {
                process_file(input_file, profile.red_scalar, profile.green_scalar, profile.blue_scalar, !profile.apply_ilt, only_new, &profile.filename_suffix);
            },
            Err(why) => {
                eprintln!("Error loading calibration profile: {}", why);
                panic!("Error loading calibration profile");
            }
        }
    } else {
        process_file(input_file, red_scalar, green_scalar, blue_scalar, no_ilt, only_new, &filename_suffix);
    }

}

pub fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, no_ilt:bool, only_new:bool, filename_suffix:&String) {
    let out_file = util::append_file_name(input_file, &filename_suffix);
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