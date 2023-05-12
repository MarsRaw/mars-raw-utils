// use rayon::prelude::*;

use crate::{calprofile::*, enums::Instrument};

use anyhow::Result;
// use sciimg::path;

pub enum CompleteStatus {
    OK,
    WARN,
    FAIL,
}

pub struct CompleteContext {
    pub status: CompleteStatus,
    pub cal_context: CalProfile,
    pub source_filename: String,
}

impl CompleteContext {
    pub fn new(status: CompleteStatus, cal_context: &CalProfile, source_filename: &str) -> Self {
        CompleteContext {
            status,
            cal_context: cal_context.clone(),
            source_filename: source_filename.to_owned(),
        }
    }
}

pub fn cal_warn(cal_context: &CalProfile, source_filename: &str) -> Result<CompleteContext> {
    Ok(CompleteContext::new(
        CompleteStatus::WARN,
        cal_context,
        source_filename,
    ))
}

pub fn cal_ok(cal_context: &CalProfile, source_filename: &str) -> Result<CompleteContext> {
    Ok(CompleteContext::new(
        CompleteStatus::OK,
        cal_context,
        source_filename,
    ))
}

pub fn cal_fail(cal_context: &CalProfile, source_filename: &str) -> Result<CompleteContext> {
    Ok(CompleteContext::new(
        CompleteStatus::FAIL,
        cal_context,
        source_filename,
    ))
}

pub trait Calibration: Sync {
    fn accepts_instrument(&self, instrument: Instrument) -> bool;

    fn process_with_profile(
        &self,
        input_file: &str,
        only_new: bool,
        profile: &CalProfile,
    ) -> Result<CompleteContext> {
        self.process_file(input_file, profile, only_new)
    }

    fn process_file(
        &self,
        input_file: &str,
        cal_context: &CalProfile,
        only_new: bool,
    ) -> Result<CompleteContext>;
}

pub struct CalContainer {
    pub calibrator: Box<dyn Calibration + 'static>,
}

pub fn process_with_profiles<F: Fn(Result<CompleteContext>)>(
    calibrator: &CalContainer,
    input_file: &str,
    only_new: bool,
    profile_names: &[CalProfile],
    on_cal_complete: F,
) {
    for profile in profile_names.iter() {
        on_cal_complete(
            calibrator
                .calibrator
                .process_with_profile(input_file, only_new, profile),
        );
    }
}

// pub fn simple_calibration_with_profiles(
//     calibrator: &CalContainer,
//     input_files: &Vec<&str>,
//     only_new: bool,
//     profiles: &[CalProfile],
// ) {
//     input_files
//         .into_par_iter()
//         .enumerate()
//         .for_each(|(idx, in_file)| {
//             if path::file_exists(in_file) {
//                 vprintln!(
//                     "Processing File: {} (#{} of {})",
//                     in_file,
//                     idx,
//                     input_files.len()
//                 );
//                 process_with_profiles(
//                     calibrator,
//                     in_file,
//                     only_new,
//                     profiles,
//                     |result| match result {
//                         Ok(cc) => print_complete(
//                             &format!(
//                                 "{} ({})",
//                                 path::basename(in_file),
//                                 cc.cal_context.filename_suffix
//                             ),
//                             cc.status,
//                         ),
//                         Err(why) => {
//                             eprintln!("Error: {}", why);
//                             print_fail(&in_file.to_string());
//                         }
//                     },
//                 );
//             } else {
//                 eprintln!("File not found: {}", in_file);
//                 print_fail(&in_file.to_string());
//             }
//         });
// }

// pub fn simple_calibration(
//     calibrator: &CalContainer,
//     input_files: &Vec<&str>,
//     only_new: bool,
//     cal_context: &CalProfile,
// ) -> Result<CompleteContext> {
//     input_files
//         .into_par_iter()
//         .enumerate()
//         .for_each(|(idx, in_file)| {
//             if path::file_exists(in_file) {
//                 vprintln!(
//                     "Processing File: {} (#{} of {})",
//                     in_file,
//                     idx,
//                     input_files.len()
//                 );
//                 calibrator
//                     .calibrator
//                     .process_file(in_file, cal_context, only_new)
//             } else {
//                 eprintln!("File not found: {}", in_file);
//                 Err(CompleteContext::new(
//                     CompleteStatus::FAIL,
//                     cal_context,
//                     in_file,
//                 ))
//                 //print_fail(&in_file.to_string());
//             }
//         });
// }
