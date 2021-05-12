use mars_raw_utils::{
    msl
};

fn main() {
    match msl::lmst::get_lmst() {
        Ok(mtime) => {
            println!("{}", mtime.display);
        },
        Err(_e) => {
            eprintln!("Error calculating mission time");
        }
    }
}