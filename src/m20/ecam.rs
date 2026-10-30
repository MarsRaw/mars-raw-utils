use crate::{
    calibfile,
    calibrate::*,
    calprofile::CalProfile,
    decompanding,
    enums::{self, Instrument},
    marsimage::MarsImage,
    memcache::load_image,
    metadata::MissionMetadata,
    util,
};
use anyhow::{anyhow, Error, Result};
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
                ))
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
                ))
            }
        };

        let subframe_rect = if let Some(md) = raw.metadata.as_rover() {
            md.subframe_rect.clone()
        } else {
            return Err(anyhow!(
                "Received nonsurface metadata when surface (rover) metadata was expected"
            ));
        };

        let scale_factor = if let Some(md) = raw.metadata.as_rover() {
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

        // vprintln!("Loading image mask");
        // let mut mask = match calibfile::get_calibration_file_for_instrument(
        //     instrument,
        //     enums::CalFileType::Mask,
        // ) {
        //     Ok(s) => {
        //         let mask_file_path = s.replace("-scalefactor-", scale_factor_str.as_str());
        //         info!(
        //             "Mask file path for scale factor {}: {}",
        //             scale_factor, mask_file_path
        //         );
        //         load_image(&mask_file_path).unwrap()
        //     }
        //     Err(why) => {
        //         warn!(
        //             "Flat file not determined for instrument {:?}: {:?}",
        //             instrument, why
        //         );
        //         Image::new_empty().unwrap()
        //     }
        // };

        if let Some(rect) = &subframe_rect {
            flat.crop(
                (rect[0] as usize - 1) / scale_factor as usize,
                (rect[1] as usize - 1) / scale_factor as usize,
                (rect[2] as usize) / scale_factor as usize,
                (rect[3] as usize) / scale_factor as usize,
            );

            info!("Flat cropped to {}x{}", flat.width, flat.height);
        }

        // if !mask.is_empty() {
        //     let mask_adjusted = create_adjusted_mask(mask.get_band(0));
        //     raw.image
        //         .set_band(&raw.image.get_band(0).multiply(&mask_adjusted).unwrap(), 0);
        //     raw.image
        //         .set_band(&raw.image.get_band(1).multiply(&mask_adjusted).unwrap(), 1);
        //     raw.image
        //         .set_band(&raw.image.get_band(2).multiply(&mask_adjusted).unwrap(), 2);
        // }
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

        // Trim off border pixels

        if cal_context.auto_subframing {
            if scale_factor == 1 {
                if let Some(rect) = &subframe_rect {
                    //rect[0] += 1.0;
                    let new_rect = vec![rect[0] + 2.0, rect[1] + 2.0, rect[2] - 2.0, rect[3] - 2.0];
                    match &mut raw.metadata.mission {
                        MissionMetadata::Surface(md) => md.subframe_rect = Some(new_rect),
                        _ => {
                                return Err(anyhow!(
                                    "Received nonsurface metadata when surface (rover) metadata was expected"
                                ))
                            }
                     }
                }
            }
            let crop_to_width = raw.image.width - 4;
            let crop_to_height = raw.image.height - 4;
            raw.crop(2, 2, crop_to_width, crop_to_height);
        }

        //
        //

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
