
use mars_raw_utils::{constants, print, vprintln, jsonfetch, httpfetch, path};
use json::{JsonValue};
use std::path::Path;
use std::fs::File;
use std::io::Write;

#[macro_use]
extern crate clap;
use std::process;
use clap::{Arg, App};

fn print_header() {
    println!("{:54} {:25} {:6} {:27} {:27} {:6} {:6} {:7}", 
                    "ID", 
                    "Instrument",
                    "Sol",
                    "Image Date (UTC)",
                    "Image Date (Mars)",
                    "Site",
                    "Drive",
                    "Thumb"
                );
}

fn print_image(image:&JsonValue) {
    println!("{:54} {:25} {:6} {:27} {:27} {:6} {:6} {:7}", 
                    image["imageid"], 
                    image["camera"]["instrument"],
                    format!("{:6}", image["sol"]), // This is such a hack...
                    &image["date_taken_utc"].as_str().unwrap()[..16],
                    image["date_taken_mars"],
                    format!("{:6}", image["site"]),
                    format!("{:6}", image["drive"]),
                    image["sample_type"] == "Thumbnail"
                );
}


fn fetch_image(image:&JsonValue) {
    let image_url = &image["image_files"]["full_res"].as_str().unwrap();
    let bn = path::basename(image_url);

    let image_data = httpfetch::simple_fetch_bin(image_url).unwrap();

    let path = Path::new("/home/kgill/Desktop").join(bn.to_string());

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}", why),
        Ok(file) => file,
    };

    file.write_all(&image_data[..]).unwrap();
}

fn main() {
    let matches = App::new(crate_name!())
                    .version(crate_version!())
                    .author(crate_authors!())
                .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                    .short(constants::param::PARAM_VERBOSE)
                    .help("Show verbose output"))
                .arg(Arg::with_name("camera")
                    .short("c")
                    .long("camera")
                    .value_name("camera")
                    .help("M20 Camera Instrument(s)")
                    .required(false)
                    .takes_value(true)
                    .multiple(true))
                .arg(Arg::with_name("sol")
                    .short("s")
                    .long("sol")
                    .value_name("sol")
                    .help("Mission Sol")
                    .required(false)
                    .takes_value(true))    
                .arg(Arg::with_name("minsol")
                    .short("m")
                    .long("minsol")
                    .value_name("minsol")
                    .help("Starting Mission Sol")
                    .required(false)
                    .takes_value(true))  
                .arg(Arg::with_name("maxsol")
                    .short("M")
                    .long("maxsol")
                    .value_name("maxsol")
                    .help("Ending Mission Sol")
                    .required(false)
                    .takes_value(true)) 
                .arg(Arg::with_name("list")
                    .short("l")
                    .long("list")
                    .value_name("list")
                    .help("Don't download, only list results")
                    .required(false)) 
                .arg(Arg::with_name("thumbnails")
                    .short("t")
                    .long("thumbnails")
                    .value_name("thumbnails")
                    .help("Download thumbnails in the results")
                    .required(false)) 
                .arg(Arg::with_name("num")
                    .short("n")
                    .long("num")
                    .value_name("num")
                    .help("Max number of results")
                    .required(false)
                    .takes_value(true))    
                .arg(Arg::with_name("page")
                    .short("p")
                    .long("page")
                    .value_name("page")
                    .help("Results page (starts at 1)")
                    .required(false)
                    .takes_value(true))  
                .arg(Arg::with_name("seqid")
                    .short("S")
                    .long("seqid")
                    .value_name("seqid")
                    .help("Specific sequence id or substring")
                    .required(false)
                    .takes_value(true))  
                .get_matches();


    /*
        parser.add_argument("-c", "--camera", help="MSL Camera Instrument(s)", required=None, default=None, type=str, nargs='+', choices=build_choices_list())
    parser.add_argument("-s", "--sol", help="Mission Sol", required=False, type=int)
    parser.add_argument("-m", "--minsol", help="Starting Mission Sol", required=False, type=int)
    parser.add_argument("-M", "--maxsol", help="Ending Mission Sol", required=False, type=int)
    parser.add_argument("-l", "--list", help="Don't download, only list results", action="store_true")
    parser.add_argument("-r", "--raw", help="Print raw JSON response", action="store_true")
    parser.add_argument("-t", "--thumbnails", help="Download thumbnails in the results", action="store_true")
    parser.add_argument("-n", "--num", help="Max number of results", required=False, type=int, default=100)
    parser.add_argument("-p", "--page", help="Results page (starts at 1)", required=False, type=int, default=1)
    parser.add_argument("-S", "--seqid", help="Specific sequence id", required=False, type=str, default=None)
    */

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let params = vec![
        vec!["feed", "raw_images"],
        vec!["category", "mars2020"],
        vec!["feedtype", "json"],
        vec!["num", "2"],
        vec!["page", "0"],
        vec!["order", "sol desc"],
        vec!["search", "NAVCAM_RIGHT"],
        vec!["condition_2", "43:sol:gte"],
        vec!["condition_3", "46:sol:lte"]
    ];

    let uri = constants::url::M20_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri);

    for p in params {
        req.param(p[0], p[1]);
    }

    let json_res = req.fetch().unwrap();

    print_header();
    for i in 0..json_res["images"].len() {
        let image = &json_res["images"][i];
        
        print_image(image);
        fetch_image(image);
        
    }

}
