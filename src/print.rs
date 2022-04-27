
static mut IS_VERBOSE: bool = false;

use chrono::prelude::*;
use colored::*;

const DATETIME_PRINT_FORMAT : &str = "%Y-%m-%d %H:%M:%S%.3f";

pub fn print_datetime() {
    print!("{} ",  Local::now().format(DATETIME_PRINT_FORMAT).to_string());
}

pub fn eprint_datetime() {
    eprint!("{} ",  Local::now().format(DATETIME_PRINT_FORMAT).to_string());
}

pub fn set_verbose(v:bool) {
    unsafe {
        IS_VERBOSE = v;
    }
}

pub fn is_verbose() -> bool {
    unsafe {
        IS_VERBOSE
    }
}

#[macro_export]
macro_rules! vprintln {
    () => (if crate::print::is_verbose() { std::print!("\n"); });
    ($($arg:tt)*) => {
        if crate::print::is_verbose() { 
            crate::print::print_datetime();
            print!("{}:{} ", file!(), line!());
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! veprintln {
    () => (if crate::print::is_verbose() { std::eprint!("\n"); });
    ($($arg:tt)*) => {
        if crate::print::is_verbose() { 
            crate::print::eprint_datetime();
            eprint!("{}:{} ", file!(), line!());
            eprintln!($($arg)*);
        }
    };
}

pub enum CompleteStatus {
    DONE,
    WARN,
    FAIL
}

pub fn print_done(file_base_name:&String) {
    print_complete(file_base_name, CompleteStatus::DONE);
}

pub fn print_warn(file_base_name:&String) {
    print_complete(file_base_name, CompleteStatus::WARN);
}

pub fn print_fail(file_base_name:&String) {
    print_complete(file_base_name, CompleteStatus::FAIL);
}

pub fn print_complete(file_base_name:&String, status:CompleteStatus) {
    println!("{: <80}[ {} ]", 
                    file_base_name,
                    match status {
                        CompleteStatus::DONE => "DONE".green(),
                        CompleteStatus::WARN => "WARN".yellow(),
                        CompleteStatus::FAIL => "FAIL".red()
                    }
    );
}