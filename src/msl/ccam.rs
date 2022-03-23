use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    path,
    calibfile,
    util,
    calprofile
};

use sciimg::imagebuffer;

pub fn process_with_profiles(input_file:&str, only_new:bool, filename_suffix:&String, profile_names:&Vec<&str>) {

    if profile_names.len() > 0 {
        for f in profile_names.iter() {
            process_with_profile(input_file, only_new, filename_suffix, Some(&f.to_string()));
        }
    } else {
        process_with_profile(input_file, only_new, filename_suffix, None);
    }

}

pub fn process_with_profile(input_file:&str, only_new:bool, filename_suffix:&String, profile_name_opt:Option<&String>) {

    if let Some(profile_name) = profile_name_opt {

        match calprofile::load_calibration_profile(&profile_name.to_string()) {
            Ok(profile) => {
                process_file(input_file, only_new, &profile.filename_suffix);
            },
            Err(why) => {
                eprintln!("Error loading calibration profile: {}", why);
                panic!("Error loading calibration profile");
            }
        }
    } else {
        process_file(input_file, only_new, &filename_suffix);
    }

}

pub fn process_file(input_file:&str, only_new:bool, filename_suffix:&String) {
    let out_file = util::append_file_name(input_file, &filename_suffix);
    if path::file_exists(&out_file) && only_new {
        vprintln!("Output file exists, skipping. ({})", out_file);
        return;
    }

    let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::MslChemCam);

    vprintln!("Loading image mask");
    let mask = imagebuffer::ImageBuffer::from_file(calibfile::get_calibration_file_for_instrument(enums::Instrument::MslChemCam, enums::CalFileType::Mask).unwrap().as_str()).unwrap();
    raw.apply_mask(&mask);

    let data_max = 255.0;

    if input_file.find("EDR") != None {
        vprintln!("Image appears to be in standard contrast");
        
        vprintln!("Flatfielding...");
        raw.flatfield();

    } else if input_file.find("EDR") != None {
        vprintln!("Image appears to be in enhanced contrast");
        // ... Don't do flatfielding, these appear to already been applied.
        // ... Do something about that
    }

    vprintln!("Normalizing...");
    raw.image.normalize_to_16bit_with_max(data_max);

    vprintln!("Writing to disk...");
    raw.save(&out_file);

}