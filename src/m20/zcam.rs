use crate::{
    calibfile, calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument,
    inpaintmask, marsimage::MarsImage, util,
};

use sciimg::prelude::*;

use anyhow::anyhow;
use anyhow::Result;

pub const MASTCAMZ_PIXEL_SIZE_MM: f32 = 0.0074;
pub const FOCAL_STOPS: [f32; 7] = [26.0, 34.0, 48.0, 63.0, 79.0, 100.0, 110.0];
pub const MOTOR_COUNT_STOPS: [u16; 7] = [0, 2448, 3834, 5196, 6720, 8652, 9600];

pub fn focal_length_from_file_name(filename: &str) -> Result<f32> {
    let bn = path::basename(filename);

    if bn.len() < 48 {
        return Err(anyhow!("Filename is invalid M20/MCZ format"));
    }

    let subs = &bn[45..48];
    if util::string_is_valid_f32(subs) {
        Ok(subs.parse::<f32>().unwrap())
    } else {
        eprintln!("Found invalid focal length value: {}", subs);
        Err(anyhow!("Invalid value"))
    }
}

fn focal_length_from_cahvor(cahvor: &CameraModel) -> Result<f32> {
    if cahvor.is_valid() {
        Ok(cahvor.f() as f32) // Reconcile the type difference.
    } else {
        Err(anyhow!("No CAHVOR data"))
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
    ) -> Result<CompleteContext> {
        let out_file = util::append_file_name(input_file, cal_context.filename_suffix.as_str());
        if path::file_exists(&out_file) && only_new {
            vprintln!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context, &out_file);
        }

        let mut warn = false;
        let instrument;

        let bn = path::basename(input_file);
        if bn.chars().nth(1).unwrap() == 'R' {
            instrument = Instrument::M20MastcamZRight;
            vprintln!("Processing for Mastcam-Z Right");
        } else {
            instrument = Instrument::M20MastcamZLeft;
            vprintln!("Processing for Mastcam-Z Left");
        }

        let mut raw = MarsImage::open(input_file, instrument);

        let data_max = if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            let lut = decompanding::get_ilt_for_instrument(instrument).unwrap();
            raw.decompand(&lut);
            lut.max() as f32
        } else {
            255.0
        };

        // Looks like 'ECM' in the name seems to indicate that it still have the bayer pattern
        // Update: Not always. Added a check to determine whether or not is is grayscale.
        // It's not perfect so please validate results. Gonna keep the 'ECM' check for now.
        if input_file.contains("ECM") && raw.image.is_grayscale() {
            vprintln!("Image appears to be grayscale, applying debayering...");
            raw.debayer_with_method(cal_context.debayer_method);
        }

        // I'm not wild about this
        let focal_length: Result<f32> = match focal_length_from_file_name(input_file) {
            Ok(fl) => Ok(fl),
            Err(_) => {
                if let Ok(fl) = focal_length_from_cahvor(&raw.metadata.camera_model_component_list)
                {
                    Ok(fl)
                } else {
                    //print_fail(&format!("{} ({})", path::basename(input_file), &cal_context.filename_suffix.to_str()));
                    panic!("Unable to determine zcam focal length")
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
                    let mut flat = MarsImage::open(&file_path, instrument);

                    if let Some(rect) = &raw.metadata.subframe_rect {
                        flat.crop(
                            rect[0] as usize - 1,
                            rect[1] as usize - 1,
                            rect[2] as usize,
                            rect[3] as usize,
                        );
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
        if let Some(rect) = &raw.metadata.subframe_rect {
            inpaint_mask = inpaint_mask
                .get_subframe(
                    rect[0] as usize - 1,
                    rect[1] as usize - 1,
                    rect[2] as usize,
                    rect[3] as usize,
                )
                .unwrap();
        }

        raw.apply_inpaint_fix_with_mask(&inpaint_mask);

        vprintln!("Applying color weights...");
        raw.apply_weight(
            cal_context.red_scalar,
            cal_context.green_scalar,
            cal_context.blue_scalar,
        );

        vprintln!(
            "Current image width: {}, height: {}",
            raw.image.width,
            raw.image.height
        );

        vprintln!("Cropping...");

        if cal_context.auto_subframing {
            if let Some(rect) = &raw.metadata.subframe_rect {
                let new_rect = vec![
                    rect[0] + 29.0,
                    rect[1] + 9.0,
                    rect[2] - 58.0,
                    rect[3] - 18.0,
                ];
                raw.metadata.subframe_rect = Some(new_rect);
            }

            raw.image
                .crop(29, 9, raw.image.width - 29 - 29, raw.image.height - 9 - 9);
        }

        vprintln!(
            "Current image width: {}, height: {}",
            raw.image.width,
            raw.image.height
        );

        if cal_context.srgb_color_correction {
            vprintln!("Applying sRGB color conversion");
            raw.image
                .convert_colorspace(color::ColorSpaceType::RGB, color::ColorSpaceType::sRGB)?;
        }

        if cal_context.decorrelate_color {
            vprintln!("Normalizing with decorrelated colors...");
            raw.image.normalize_to_16bit_decorrelated();
        } else {
            vprintln!("Normalizing with correlated colors...");
            raw.image.normalize_to_16bit_with_max(data_max);
        }

        vprintln!("Writing to {}", out_file);
        raw.update_history();
        match raw.save(&out_file) {
            Ok(_) => match warn {
                true => cal_warn(cal_context, &out_file),
                false => cal_ok(cal_context, &out_file),
            },
            Err(why) => {
                veprintln!("Error saving file: {}", why);
                cal_fail(cal_context, &out_file)
            }
        }
    }
}
