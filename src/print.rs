
static mut IS_VERBOSE: bool = false;


pub fn set_verbose(v:bool) {
    unsafe {
        IS_VERBOSE = v;
    }
}

pub fn is_verbose() -> bool {
    unsafe {
        return IS_VERBOSE;
    }
}

#[macro_export]
macro_rules! vprintln {
    () => (if crate::print::is_verbose() { std::print!("\n"); });
    ($($arg:tt)*) => {
        if crate::print::is_verbose() { 
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! veprintln {
    () => (if crate::print::is_verbose() { std::print!("\n"); });
    ($($arg:tt)*) => {
        if crate::print::is_verbose() { 
            eprintln!($($arg)*);
        }
    };
}
