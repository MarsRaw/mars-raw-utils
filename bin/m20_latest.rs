use mars_raw_utils::{
    m20
};

use std::process;

fn main() {

    let latest : m20::latest::LatestData = match m20::remote::fetch_latest() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error fetching latest data from M20 remote server: {}", e);
            process::exit(1);
        }
    };

    println!("Latest data: {}", latest.latest);
    println!("Latest sol: {}", latest.latest_sol);
    println!("Latest sols: {:?}", latest.latest_sols);
    println!("New Count: {}", latest.new_count);
    println!("Sol Count: {}", latest.sol_count);
    println!("Total: {}", latest.total);
}