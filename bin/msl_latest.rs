use mars_raw_utils::prelude::*;

#[macro_use]
extern crate clap;
use std::process;
use clap::{Arg, App};

fn main() {
    let matches = App::new(crate_name!())
                    .version(crate_version!())
                    .author(crate_authors!())
                .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                    .short(constants::param::PARAM_VERBOSE)
                    .help("Show verbose output"))
                .arg(Arg::with_name(constants::param::PARAM_LIST)
                    .short(constants::param::PARAM_LIST_SHORT)
                    .long(constants::param::PARAM_LIST)
                    .value_name(constants::param::PARAM_LIST)
                    .help("List sols with new images only")
                    .required(false)
                    .takes_value(false))
                .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let list_new_sols = matches.is_present(constants::param::PARAM_LIST);

    let latest : msl::latest::LatestData = match msl::remote::fetch_latest() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error fetching latest data from MSL remote server: {}", e);
            process::exit(1);
        }
    };

    if list_new_sols {
        latest.latest_sols.iter().for_each(|s| {
            println!("{}", s);
        });
    } else {
        println!("Latest data: {}", latest.latest);
        println!("Latest sol: {}", latest.latest_sol);
        println!("Latest sols: {:?}", latest.latest_sols);
        println!("New Count: {}", latest.new_count);
        println!("Sol Count: {}", latest.sol_count);
        println!("Total: {}", latest.total);
    }
}