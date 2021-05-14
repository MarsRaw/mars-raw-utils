use mars_raw_utils::{
    m20
};

fn main() {
    match m20::lmst::get_lmst() {
        Ok(mtime) => {
            println!("{}", mtime.lmst_display);
        },
        Err(_e) => {
            eprintln!("Error calculating mission time");
        }
    }
}