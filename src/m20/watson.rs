use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    path,
    util,
    decompanding,
    calprofile,
    inpaintmask,
    flatfield
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
    
    let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20Watson);
    


    let mut data_max = 255.0;

    if ! no_ilt {
       vprintln!("Decompanding...");
       raw.decompand(&decompanding::ILT);
       data_max = decompanding::get_max_for_instrument(enums::Instrument::M20Watson) as f32;
    }

    vprintln!("Flatfielding...");
    let mut flat = flatfield::load_flat(enums::Instrument::M20Watson).unwrap();
    if raw.image.width == 1584 && raw.image.height == 1184 {
        flat.image.crop(32, 16, 1584, 1184);
    }
    raw.flatfield_with_flat(&flat);
    
    vprintln!("Inpainting...");
    let mut inpaint_mask = inpaintmask::load_mask(enums::Instrument::M20Watson).unwrap();
    if raw.image.width == 1584 && raw.image.height == 1184 {
        inpaint_mask = inpaint_mask.get_subframe(32, 16, 1584, 1184).unwrap();
    }
    raw.apply_inpaint_fix_with_mask(&inpaint_mask);

    if input_file.find("ECM") != None && raw.image.is_grayscale() {
        vprintln!("Image appears to be grayscale, applying debayering...");
        raw.debayer();
    }

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