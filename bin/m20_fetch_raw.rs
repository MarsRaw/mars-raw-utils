
use mars_raw_utils::{
    constants, 
    print, 
    jsonfetch, 
    httpfetch, 
    path, 
    util
};
use json::{
    JsonValue
};
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

fn process_results(json_res:&JsonValue, thumbnails:bool, list_only:bool, search:&str) {
    print_header();

    let mut valid_img_count = 0;
    for i in 0..json_res["images"].len() {
        let image = &json_res["images"][i];
        
        // If this image is a thumbnail and we're ignoring those, then ignore it.
        if image["sample_type"] == "Thumbnail" && ! thumbnails {
            continue;
        }

        // If we're searching for a substring and this image doesn't match, skip it.
        if search != "" && image["imageid"].as_str().unwrap().find(&search) == None {
            continue;
        }

        valid_img_count += 1;
        print_image(image);

        if !list_only {
            fetch_image(image);
        }
        
    }

    println!("{} images found", valid_img_count);
}

fn fetch_image(image:&JsonValue) {
    let image_url = &image["image_files"]["full_res"].as_str().unwrap();
    let bn = path::basename(image_url);

    let image_data = httpfetch::simple_fetch_bin(image_url).unwrap();

    let path = Path::new(bn.as_str());

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
                    .takes_value(false)
                    .required(false)) 
                .arg(Arg::with_name("movie")
                    .short("e")
                    .long("movie")
                    .value_name("movie")
                    .help("Only movie frames")
                    .takes_value(false)
                    .required(false)) 
                .arg(Arg::with_name("thumbnails")
                    .short("t")
                    .long("thumbnails")
                    .value_name("thumbnails")
                    .help("Download thumbnails in the results")
                    .takes_value(false)
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


    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let mut num_per_page = 100;
    let mut page = 1;
    let mut minsol = 1000000;
    let mut maxsol = -1;
    let mut sol = -1;
    let mut thumbnails = false;
    let mut search = "";
    let mut list_only = false;
    let mut movie_only = false;

    let mut cameras: Vec<&str> = Vec::default();
    if matches.is_present("camera") {
        cameras = matches.values_of("camera").unwrap().collect();
    }
    
    if matches.is_present("thumbnails") {
        thumbnails = true;
    }

    if matches.is_present("movie") {
        movie_only = true;
    }

    if matches.is_present("list") {
        list_only = true;
    }

    if matches.is_present("seqid") {
        search =  matches.value_of("seqid").unwrap();
    }

    if matches.is_present("num") {
        let s = matches.value_of("num").unwrap();
        if util::string_is_valid_f32(&s) {
            num_per_page = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if matches.is_present("page") {
        let s = matches.value_of("page").unwrap();
        if util::string_is_valid_f32(&s) {
            page = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if matches.is_present("minsol") {
        let s = matches.value_of("minsol").unwrap();
        if util::string_is_valid_f32(&s) {
            minsol = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if matches.is_present("maxsol") {
        let s = matches.value_of("maxsol").unwrap();
        if util::string_is_valid_f32(&s) {
            maxsol = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if matches.is_present("sol") {
        let s = matches.value_of("sol").unwrap();
        if util::string_is_valid_f32(&s) {
            sol = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if sol >= 0 {
        minsol = sol;
        maxsol = sol;
    }

    let joined_cameras = cameras.join("|");
    let num_per_page_s = format!("{}", num_per_page);
    let page_s = format!("{}", (page - 1));
    let minsol_s = format!("{}:sol:gte", minsol);
    let maxsol_s = format!("{}:sol:lte", maxsol);

    let mut params = vec![
        vec!["feed", "raw_images"],
        vec!["category", "mars2020"],
        vec!["feedtype", "json"],
        vec!["num", num_per_page_s.as_str()],
        vec!["page", page_s.as_str()],
        vec!["order", "sol desc"],
        vec!["search", joined_cameras.as_str()],
        vec!["condition_2", minsol_s.as_str()],
        vec!["condition_3", maxsol_s.as_str()]
    ];

    if thumbnails {
        params.push(vec!["extended", "sample_type::thumbnail,"]);
    } else if movie_only {
        params.push(vec!["extended", "sample_type::full,product_id::ecv,"]);
    } else {
        params.push(vec!["extended", "sample_type::full,"]);
    }

    let uri = constants::url::M20_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri);

    for p in params {
        req.param(p[0], p[1]);
    }

    match req.fetch() {
        Ok(v) => process_results(&v, thumbnails, list_only, search),
        Err(_e) => eprintln!("Error fetching data from remote server")
    }

}
