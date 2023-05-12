use crate::calibrate::CompleteStatus;

static mut IS_VERBOSE: bool = false;

type FnPrint = dyn Fn(&String) + Send + Sync + 'static;
static mut PRINT: Option<Box<FnPrint>> = None;

use chrono::prelude::*;
use colored::*;
use termsize;

const DATETIME_PRINT_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

pub fn set_print<F: Fn(&String) + Send + Sync + 'static>(f: F) {
    unsafe {
        PRINT = Some(Box::new(f));
    }
}

pub fn do_println(s: &String) {
    unsafe {
        if let Some(p) = &PRINT {
            p(s);
        } else {
            println!("{}", s);
        }
    }
}

pub fn set_verbose(v: bool) {
    unsafe {
        IS_VERBOSE = v;
    }
}

pub fn is_verbose() -> bool {
    unsafe { IS_VERBOSE }
}

pub fn format_datetime() -> String {
    format!("{} ", Local::now().format(DATETIME_PRINT_FORMAT))
}

pub fn eprint_datetime() {
    eprint!("{} ", Local::now().format(DATETIME_PRINT_FORMAT));
}

/// Print to stdout if user specified increased output verbosity
#[macro_export]
macro_rules! vprintln {
    () => (if $crate::print::is_verbose() { std::print!("\n"); });
    ($($arg:tt)*) => {
        if $crate::print::is_verbose() {
            $crate::print::do_println(&format!("{} {}:{} {}", $crate::print::format_datetime(), file!(), line!(), format!($($arg)*)));
        }
    };
}

/// Print to stderr if user specified increased output verbosity
#[macro_export]
macro_rules! veprintln {
    () => (if $crate::print::is_verbose() { std::eprint!("\n"); });
    ($($arg:tt)*) => {
        if $crate::print::is_verbose() {
            eprintln!("{} {}:{} {}", $crate::print::format_datetime(), file!(), line!(), format!($($arg)*));
        }
    };
}

pub fn print_done(file_base_name: &String) {
    do_println(&format_complete(file_base_name, CompleteStatus::OK));
}

pub fn format_done(file_base_name: &String) -> String {
    format_complete(file_base_name, CompleteStatus::OK)
}

pub fn print_warn(file_base_name: &String) {
    do_println(&format_complete(file_base_name, CompleteStatus::WARN));
}

pub fn format_warn(file_base_name: &String) -> String {
    format_complete(file_base_name, CompleteStatus::WARN)
}

pub fn print_fail(file_base_name: &String) {
    do_println(&format_complete(file_base_name, CompleteStatus::FAIL));
}

pub fn format_fail(file_base_name: &String) -> String {
    format_complete(file_base_name, CompleteStatus::FAIL)
}

pub fn print_complete(file_base_name: &String, status: CompleteStatus) {
    do_println(&format_complete(file_base_name, status));
}

pub fn format_complete(file_base_name: &String, status: CompleteStatus) -> String {
    let mut width = 88;

    if let Some(size) = termsize::get() {
        if size.cols < width {
            width = size.cols;
        }
    };

    let mut formatted = format!("{: <80}", file_base_name);
    if formatted.len() > width as usize - 8 {
        formatted = String::from(&formatted[0..(width as usize - 8)]);
    }

    format!(
        "{}[ {} ]",
        formatted,
        match status {
            CompleteStatus::OK => "DONE".green(),
            CompleteStatus::WARN => "WARN".yellow(),
            CompleteStatus::FAIL => "FAIL".red(),
        }
    )
}

pub fn print_experimental() {
    do_println(&format!("{} - Results may vary, bugs will be present, and not all functionality has been implemented", "Experimental Code!".red()));
}
