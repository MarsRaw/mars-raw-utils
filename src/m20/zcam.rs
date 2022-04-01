use crate::{
    vprintln, 
    image::MarsImage, 
    enums, 
    path,
    decompanding,
    util,
    calprofile,
    error,
    calibfile
};

use sciimg::{
    cahvor::Cahvor
};

pub const MASTCAMZ_PIXEL_SIZE_MM:f32 = 0.0074;
pub const FOCAL_STOPS:[f32; 7] = [26.0, 34.0, 48.0, 63.0, 79.0, 100.0, 110.0];
pub const MOTOR_COUNT_STOPS:[u16; 7] = [0, 2448, 3834, 5196, 6720, 8652, 9600];

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

pub fn focal_length_from_file_name(filename:&str) -> error::Result<f32> {

    let bn = path::basename(filename);

    if bn.len() < 48 {
        return Err("Filename is invalid M20/MCZ format");
    }

    let subs = &bn[45..48];
    if util::string_is_valid_f32(subs) {
        Ok(subs.parse::<f32>().unwrap())
    } else {
        eprintln!("Found invalid focal length value: {}", subs);
        Err("Invalid value")
    }   
}

fn focal_length_from_cahvor(cahvor:&Option<Cahvor>) -> error::Result<f32> {
    match cahvor {
        Some(c) => Ok(c.f() as f32 * MASTCAMZ_PIXEL_SIZE_MM),
        None => Err("No CAHVOR data")
    }
}

fn close_enough(a:f32, b:f32) -> bool {
    (a - b).abs() < 5.0
}

fn motor_stop_from_focal_length(fl:f32) -> u16 {
    for i in 1..FOCAL_STOPS.len() {
        if close_enough(fl, FOCAL_STOPS[i]) {
            return MOTOR_COUNT_STOPS[i];
        }
    }
    MOTOR_COUNT_STOPS[0]
}

pub fn process_file(input_file:&str, red_scalar:f32, green_scalar:f32, blue_scalar:f32, no_ilt:bool, only_new:bool, filename_suffix:&String) {
    let out_file = util::append_file_name(input_file, filename_suffix);
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

    // I'm not wild about this
    let focal_length = match &raw.metadata {
        Some(md) => {
            
            let fl_res = focal_length_from_cahvor(&md.camera_model_component_list);
            if let Ok(fl) = fl_res {
                Ok(fl)
            } else {
                focal_length_from_file_name(input_file)
            }
            
        },
        None => {
            focal_length_from_file_name(input_file)
        }
    };
    

    match focal_length {
        Ok(fl) => {
            // Do flat fielding
            vprintln!("Flatfielding...");
            vprintln!("Determined camera focal length at {}mm", fl);

            let calfile = calibfile::get_calibration_file_for_instrument(instrument, enums::CalFileType::FlatField).unwrap();
            
            let motor_stop = motor_stop_from_focal_length(fl);
            let file_path = calfile.replace("-motorcount-", motor_stop.to_string().as_str());

            vprintln!("Using flat file: {}", file_path);

            if path::file_exists(&file_path) {
                let flat = MarsImage::open(file_path, instrument);
                raw.flatfield_with_flat(&flat);
            } else {
                eprintln!("Flat file not found: {}", file_path);
                panic!("Flat file not found!");
            }
            
        },
        Err(e) => {
            vprintln!("Could not determine focal length: {}", e)
        }
    };
    //
    //raw.flatfield();


    vprintln!("Inpainting...");
    raw.apply_inpaint_fix();

    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar);

    vprintln!("Normalizing...");
    raw.image.normalize_to_16bit_with_max(data_max);

    if raw.image.width == 1648 {
        vprintln!("Cropping...");
        raw.image.crop(29, 9, 1590, 1182);
    }

    vprintln!("Writing to disk...");
    
    raw.save(&out_file);
}
