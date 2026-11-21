use crate::{
    calibfile,
    calibrate::{Calibration, CompleteContext, cal_fail, cal_ok, cal_warn},
    calprofile::CalProfile,
    decompanding,
    enums::{self, Instrument},
    marsimage::MarsImage,
    memcache::load_image,
    metadata::MissionMetadata::{self, Surface},
    util,
};
use anyhow::{Error, Result, anyhow};
use sciimg::{camera::cahvore::linearity, image::Image, path};

#[derive(Copy, Clone)]
pub struct M20EECam {}

// This really shouldn't be hardcoded
fn get_bias_for_instrument(instrument: Instrument) -> Result<f32> {
    match instrument {
        Instrument::M20NavcamLeft => Ok(133.0),
        Instrument::M20NavcamRight => Ok(132.0),
        Instrument::M20FrontHazLeft => Ok(151.0),
        Instrument::M20FrontHazRight => Ok(148.0),
        Instrument::M20RearHazLeft => Ok(118.0),
        Instrument::M20RearHazRight => Ok(132.0),
        _ => Err(Error::msg(format!(
            "Invalid M20 ECAM instrument: {:?}",
            instrument
        ))),
    }
}

impl Calibration for M20EECam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(
            instrument,
            Instrument::M20FrontHazLeft
                | Instrument::M20FrontHazRight
                | Instrument::M20NavcamLeft
                | Instrument::M20NavcamRight
                | Instrument::M20RearHazLeft
                | Instrument::M20RearHazRight
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
            info!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context, &out_file);
        }

        let mut raw = MarsImage::open(input_file)?;
        let instrument = match raw.metadata.instrument() {
            Ok(instrument) => instrument,
            Err(why) => {
                return Err(anyhow!(
                    "Unable to determine camera instrument from metadata: {:?}",
                    why
                ));
            }
        };

        // Override default linearity
        match &mut raw.metadata.mission {
            MissionMetadata::Surface(md) => {
                md.camera_model_component_list.linearity = linearity::LINEARITY_FISHEYE
            }
            _ => {
                return Err(anyhow!(
                    "Received nonsurface metadata when surface (rover) metadata was expected"
                ));
            }
        };

        let subframe_rect = if let Some(md) = raw.metadata.as_surface() {
            md.subframe_rect.clone()
        } else {
            return Err(anyhow!(
                "Received nonsurface metadata when surface (rover) metadata was expected"
            ));
        };

        let scale_factor = if let Some(md) = raw.metadata.as_surface() {
            md.scale_factor
        } else {
            return Err(anyhow!(
                "Received nonsurface metadata when surface (rover) metadata was expected"
            ));
        };

        let data_max = if cal_context.apply_ilt {
            info!("Decompanding...");
            let lut = decompanding::get_ilt_for_instrument(instrument).unwrap();
            raw.decompand(&lut);
            lut.max() as f32
        } else {
            255.0
        };

        // Looks like 'ECM' in the name seems to indicate that it still have the bayer pattern
        if raw.image.is_grayscale() {
            info!("Debayering...");
            raw.debayer_with_method(cal_context.debayer_method);
        }

        let bias = get_bias_for_instrument(instrument)?;
        info!("Applying Bias Subtraction of {}...", bias);
        raw.image.apply_bias_subtraction(bias);

        info!("Flatfielding...");
        let scale_factor = if scale_factor >= 1 { scale_factor } else { 1 };
        let scale_factor_str = format!("sf{}", scale_factor);

        let mut flat = match calibfile::get_calibration_file_for_instrument(
            instrument,
            enums::CalFileType::FlatField,
        ) {
            Ok(s) => {
                let flat_file_path = s.replace("-scalefactor-", scale_factor_str.as_str());
                info!(
                    "Flat file path for scale factor {}: {}",
                    scale_factor, flat_file_path
                );
                load_image(&flat_file_path).unwrap()
            }
            Err(why) => {
                warn!(
                    "Flat file not determined for instrument {:?}: {:?}",
                    instrument, why
                );
                Image::new_empty().unwrap()
            }
        };

        if let Some(rect) = &subframe_rect {
            flat.crop(
                (rect[0] as usize - 1) / scale_factor as usize,
                (rect[1] as usize - 1) / scale_factor as usize,
                (rect[2] as usize) / scale_factor as usize,
                (rect[3] as usize) / scale_factor as usize,
            );

            info!("Flat cropped to {}x{}", flat.width, flat.height);
        }

        raw.apply_flat(&flat);

        if !raw.image.is_grayscale() {
            info!("Applying color weights...");
            raw.apply_weight(
                cal_context.red_scalar,
                cal_context.green_scalar,
                cal_context.blue_scalar,
            );
        }

        if cal_context.decorrelate_color {
            info!("Normalizing with decorrelated colors...");
            raw.image.normalize_to_16bit_decorrelated();
        } else {
            info!("Normalizing with correlated colors...");
            raw.image.normalize_to_16bit_with_max(data_max);
        }

        vprintln!("Loading image mask");

        let mask_file_path = match calibfile::get_calibration_file_for_instrument(
            instrument,
            enums::CalFileType::Mask,
        ) {
            Ok(s) => Some(s.replace("-scalefactor-", scale_factor_str.as_str())),
            Err(why) => {
                warn!(
                    "Flat file not determined for instrument {:?}: {:?}",
                    instrument, why
                );
                None
            }
        };
        if let Some(mask_file_path) = &mask_file_path {
            if path::file_exists(mask_file_path) && path::file_readable(mask_file_path) {
                info!("Loading mask file from {}", mask_file_path);
                let mut mask = load_image(mask_file_path)?;
                if scale_factor == 2 {
                    let d_w = mask.width / scale_factor as usize;
                    let d_h = mask.height / scale_factor as usize;
                    info!(
                        "Downscaling mask file to {}x{} (raw image dimensions: {}x{})",
                        d_w, d_h, raw.image.width, raw.image.height
                    );
                    mask.resize_to_with_filter(d_w, d_h, sciimg::resize::ResizeFilter::Nearest);
                }

                if let Some(rect) = &subframe_rect {
                    mask.crop(
                        (rect[0] as usize - 1) / scale_factor as usize,
                        (rect[1] as usize - 1) / scale_factor as usize,
                        (rect[2] as usize) / scale_factor as usize,
                        (rect[3] as usize) / scale_factor as usize,
                    );

                    info!("Mask cropped to {}x{}", mask.width, mask.height);
                }

                info!("Applying mask to image");
                raw.apply_alpha(mask.get_band(0));
                raw.image.set_using_alpha(true);
            } else {
                warn!(
                    "Mask file does not exist or is not readable: {}",
                    mask_file_path
                );
            }
        }
        // Trim off border pixels

        if cal_context.auto_subframing {
            if scale_factor == 1
                && let Some(rect) = &subframe_rect
            {
                //rect[0] += 1.0;
                let new_rect = vec![rect[0] + 2.0, rect[1] + 2.0, rect[2] - 2.0, rect[3] - 2.0];
                match &mut raw.metadata.mission {
                    MissionMetadata::Surface(md) => md.subframe_rect = Some(new_rect),
                    _ => {
                        return Err(anyhow!(
                            "Received nonsurface metadata when surface (rover) metadata was expected"
                        ));
                    }
                }
            }
            let crop_to_width = raw.image.width - 4;
            let crop_to_height = raw.image.height - 4;
            raw.crop(2, 2, crop_to_width, crop_to_height);
        }

        //
        //

        // The camera model is prescaled relative to the scaling factor, so technically they are out of sync when scale_factor > 2.
        // We have already done the operations required to deal with scale_factor, so let's just set it to 1 so it
        // matches the scale of the camera model.

        let mut rover_metadata = match raw.metadata.as_surface() {
            Some(md) => md.clone(),
            None => return cal_fail(cal_context, &out_file),
        };
        rover_metadata.scale_factor = 1;
        raw.metadata.mission = Surface(rover_metadata);

        vprintln!("Writing to disk...");
        raw.update_history();

        match raw.save(&out_file) {
            Ok(_) => cal_ok(cal_context, &out_file),
            Err(why) => {
                error!("Error saving file: {}", why);
                cal_fail(cal_context, &out_file)
            }
        }
    }
}
