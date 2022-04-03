use mars_raw_utils::prelude::*;

use std::process;

fn main() {

    let latest : nsyt::latest::LatestData = match nsyt::remote::fetch_latest() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error fetching latest data from InSight remote server: {}", e);
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