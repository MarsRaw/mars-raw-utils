use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    path, 
    util,
    calprofile
};


pub fn process_with_profiles(input_file:&str, hpc_threshold:f32, hpc_window_size:i32, only_new:bool, filename_suffix:&String, profile_names:&Vec<&str>) {

    if profile_names.len() > 0 {
        for f in profile_names.iter() {
            process_with_profile(input_file, hpc_threshold, hpc_window_size, only_new, filename_suffix, Some(&f.to_string()));
        }
    } else {
        process_with_profile(input_file, hpc_threshold, hpc_window_size, only_new, filename_suffix, None);
    }

}

pub fn process_with_profile(input_file:&str, hpc_threshold:f32, hpc_window_size:i32, only_new:bool, filename_suffix:&String, profile_name_opt:Option<&String>) {

    if let Some(profile_name) = profile_name_opt {

        match calprofile::load_calibration_profile(&profile_name.to_string()) {
            Ok(profile) => {
                process_file(input_file, profile.hot_pixel_detection_threshold, hpc_window_size, only_new, &profile.filename_suffix);
            },
            Err(why) => {
                eprintln!("Error loading calibration profile: {}", why);
                panic!("Error loading calibration profile");
            }
        }
    } else {
        process_file(input_file, hpc_threshold, hpc_window_size, only_new, &filename_suffix);
    }

}
pub fn process_file(input_file:&str, hpc_threshold:f32, hpc_window_size:i32, only_new:bool, filename_suffix:&String) {
    let out_file = util::append_file_name(input_file, &filename_suffix);
    if path::file_exists(&out_file) && only_new {
        vprintln!("Output file exists, skipping. ({})", out_file);
        return;
    }
    
    let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::M20SkyCam);

    vprintln!("Flatfielding...");
    raw.flatfield();

    if hpc_threshold > 0.0 {
        vprintln!("Hot pixel correction with variance threshold {}...", hpc_threshold);
        raw.hot_pixel_correction(hpc_window_size, hpc_threshold);
    }

    vprintln!("Normalizing...");
    raw.image.normalize_to_16bit_with_max(255.0);

    // Trim off border pixels
    vprintln!("Cropping border pixels...");
    let crop_to_width = raw.image.width - 34;
    let crop_to_height = raw.image.height - 2;
    raw.image.crop(18, 1, crop_to_width, crop_to_height);

    vprintln!("Writing to disk...");
    raw.save(&out_file);
}
