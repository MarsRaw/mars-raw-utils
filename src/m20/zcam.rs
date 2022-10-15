use crate::{
    calibfile, calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument,
    image::MarsImage, inpaintmask, path, util, vprintln,
};

use sciimg::prelude::*;

pub const MASTCAMZ_PIXEL_SIZE_MM: f32 = 0.0074;
pub const FOCAL_STOPS: [f32; 7] = [26.0, 34.0, 48.0, 63.0, 79.0, 100.0, 110.0];
pub const MOTOR_COUNT_STOPS: [u16; 7] = [0, 2448, 3834, 5196, 6720, 8652, 9600];

pub fn focal_length_from_file_name(filename: &str) -> error::Result<f32> {
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

fn focal_length_from_cahvor(cahvor: &CameraModel) -> error::Result<f32> {
    if cahvor.is_valid() {
        Ok(cahvor.f() as f32) // Reconcile the type difference.
    } else {
        Err("No CAHVOR data")
    }
}

fn close_enough(a: f32, b: f32) -> bool {
    (a - b).abs() < 5.0
}

fn motor_stop_from_focal_length(fl: f32) -> u16 {
    for i in 1..FOCAL_STOPS.len() {
        if close_enough(fl, FOCAL_STOPS[i]) {
            return MOTOR_COUNT_STOPS[i];
        }
    }
    MOTOR_COUNT_STOPS[0]
}

#[derive(Copy, Clone)]
pub struct M20MastcamZ {}

impl Calibration for M20MastcamZ {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(
            instrument,
            Instrument::M20MastcamZLeft | Instrument::M20MastcamZRight
        )
    }

    fn process_file(
        &self,
        input_file: &str,
        cal_context: &CalProfile,
        only_new: bool,
    ) -> error::Result<CompleteContext> {
        let out_file = util::append_file_name(input_file, cal_context.filename_suffix.as_str());
        if path::file_exists(&out_file) && only_new {
            vprintln!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context);
        }

        let mut warn = false;
        let mut instrument = Instrument::M20MastcamZLeft;

        let bn = path::basename(input_file);
        if bn.chars().nth(1).unwrap() == 'R' {
            instrument = Instrument::M20MastcamZRight;
            vprintln!("Processing for Mastcam-Z Right");
        } else {
            vprintln!("Processing for Mastcam-Z Left");
        }

        let mut raw = MarsImage::open(String::from(input_file), instrument);

        let mut data_max = 255.0;

        if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            raw.decompand(&decompanding::get_ilt_for_instrument(instrument));
            data_max = decompanding::get_max_for_instrument(instrument) as f32;
        }

        // Looks like 'ECM' in the name seems to indicate that it still have the bayer pattern
        // Update: Not always. Added a check to determine whether or not is is grayscale.
        // It's not perfect so please validate results. Gonna keep the 'ECM' check for now.
        if input_file.contains("ECM") && raw.image.is_grayscale() {
            vprintln!("Image appears to be grayscale, applying debayering...");
            raw.debayer();
        }

        // I'm not wild about this
        let focal_length: error::Result<f32> = match focal_length_from_file_name(input_file) {
            Ok(fl) => Ok(fl),
            Err(_) => {
                match &raw.metadata {
                    Some(md) => {
                        let fl_res = focal_length_from_cahvor(&md.camera_model_component_list);
                        if let Ok(fl) = fl_res {
                            Ok(fl)
                        } else {
                            //print_fail(&format!("{} ({})", path::basename(input_file), &cal_context.filename_suffix.to_str()));
                            panic!("Unable to determine zcam focal length")
                        }
                    }
                    None => {
                        //print_fail(&format!("{} ({})", path::basename(input_file), filename_suffix));
                        panic!("Unable to determine zcam focal length")
                    }
                }
            }
        };

        match focal_length {
            Ok(fl) => {
                // Do flat fielding
                vprintln!("Flatfielding...");
                vprintln!("Determined camera focal length at {}mm", fl);

                let calfile = calibfile::get_calibration_file_for_instrument(
                    instrument,
                    enums::CalFileType::FlatField,
                )
                .unwrap();

                let motor_stop = motor_stop_from_focal_length(fl);
                let motor_stop_str = format!("{:04}", motor_stop);
                let file_path = calfile.replace("-motorcount-", motor_stop_str.as_str());

                vprintln!("Using flat file: {}", file_path);

                if path::file_exists(&file_path) {
                    let mut flat = MarsImage::open(file_path, instrument);

                    if let Some(md) = &raw.metadata {
                        if let Some(rect) = &md.subframe_rect {
                            flat.crop(
                                rect[0] as usize - 1,
                                rect[1] as usize - 1,
                                rect[2] as usize,
                                rect[3] as usize,
                            );
                        }
                    }

                    raw.flatfield_with_flat(&flat);
                } else {
                    eprintln!("Flat file not found: {}", file_path);
                    // print_fail(&format!("{} ({})", path::basename(input_file), filename_suffix));
                    panic!("Flat file not found!");
                }
            }
            Err(e) => {
                warn = true;
                vprintln!("Could not determine focal length: {}", e)
            }
        };

        vprintln!("Inpainting...");
        let mut inpaint_mask = inpaintmask::load_mask(instrument).unwrap();
        if let Some(md) = &raw.metadata {
            if let Some(rect) = &md.subframe_rect {
                inpaint_mask = inpaint_mask
                    .get_subframe(
                        rect[0] as usize - 1,
                        rect[1] as usize - 1,
                        rect[2] as usize,
                        rect[3] as usize,
                    )
                    .unwrap();
            }
        }
        raw.apply_inpaint_fix_with_mask(&inpaint_mask);

        vprintln!("Applying color weights...");
        raw.apply_weight(
            cal_context.red_scalar,
            cal_context.green_scalar,
            cal_context.blue_scalar,
        );

        vprintln!("Normalizing...");
        raw.image.normalize_to_16bit_with_max(data_max);

        if raw.image.width == 1648 && raw.image.height == 1200 {
            vprintln!("Cropping...");
            raw.image.crop(29, 9, 1590, 1182);
        }

        vprintln!("Writing to disk...");

        raw.save(&out_file);

        match warn {
            true => cal_warn(cal_context),
            false => cal_ok(cal_context),
        }
    }
}
