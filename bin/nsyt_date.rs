use mars_raw_utils::{
    nsyt
};

fn main() {
    match nsyt::lmst::get_lmst() {
        Ok(mtime) => {
            println!("{}", mtime.lmst_display);
        },
        Err(_e) => {
            eprintln!("Error calculating mission time");
        }
    }
}