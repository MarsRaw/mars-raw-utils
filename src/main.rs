#[macro_use]
extern crate clap;

fn main() {
    println!("{}, Version {}", crate_name!(), crate_version!());
}
