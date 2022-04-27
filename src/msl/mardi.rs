use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    path,
    decompanding,
    util,
    calprofile,
    print::{
        print_done,
        print_fail,
        print_warn
    }
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
                print_fail(&format!("{} ({})", path::basename(input_file), filename_suffix));
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
        print_warn(&format!("{} ({})", path::basename(input_file), filename_suffix));
        return;
    }

    let mut raw = MarsImage::open(String::from(input_file), enums::Instrument::MslMARDI);

    let mut data_max = 255.0;

    if ! no_ilt {
        vprintln!("Decompanding...");
        raw.decompand(&decompanding::get_ilt_for_instrument(enums::Instrument::MslMARDI));
        data_max = decompanding::get_max_for_instrument(enums::Instrument::MslMARDI) as f32;
    }

    vprintln!("Flatfielding...");
    raw.flatfield();
    
    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar);

    vprintln!("Cropping...");
    raw.image.crop(24, 6, 1599, 1188);

    vprintln!("Normalizing...");
    raw.image.normalize_to_16bit_with_max(data_max);

    vprintln!("Writing to disk...");
    raw.save(&out_file);

    print_done(&format!("{} ({})", path::basename(input_file), filename_suffix));
}