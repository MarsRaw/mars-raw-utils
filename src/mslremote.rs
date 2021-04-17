
use crate::{
    constants, 
    vprintln, 
    jsonfetch, 
    httpfetch, 
    path
};

use json::{JsonValue};
use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

fn image_exists_on_filesystem(image:&JsonValue) -> bool {
    let image_url = &image["url"].as_str().unwrap();
    let bn = path::basename(image_url);
    path::file_exists(bn.as_str())
}

fn print_header() {
    println!("{:37} {:15} {:6} {:20} {:27} {:6} {:6} {:7} {:10}", 
                    "ID", 
                    "Instrument",
                    "Sol",
                    "Image Date (UTC)",
                    "Image Date (Mars)",
                    "Site",
                    "Drive",
                    "Thumb",
                    "Present"
                );
}


fn null_to_str(item:&JsonValue) -> String {
    if item.is_null() {
        return String::from("");
    } else {
        return format!("{}", item);
    }
}

fn print_image(image:&JsonValue) {
    println!("{:37} {:15} {:6} {:20} {:27} {:6} {:6} {:7} {:10}", 
                    image["imageid"], 
                    image["instrument"],
                    format!("{:6}", image["sol"]), // This is such a hack...
                    &image["date_taken"].as_str().unwrap()[..16],
                    null_to_str(&image["extended"]["lmst"]),
                    format!("{:6}", null_to_str(&image["site"])),
                    format!("{:6}", null_to_str(&image["drive"])),
                    if image["is_thumbnail"].as_bool().unwrap() { constants::status::YES } else { constants::status::NO },
                    if image_exists_on_filesystem(&image) { constants::status::YES } else { constants::status::NO }
                );
}


fn fetch_image(image:&JsonValue, only_new:bool) {
    let image_url = &image["url"].as_str().unwrap();
    let bn = path::basename(image_url);

    if image_exists_on_filesystem(&image) && only_new {
        vprintln!("Output file {} exists, skipping", bn);
        return;
    }

    // Dude, error checking!!
    let image_data = httpfetch::simple_fetch_bin(image_url).unwrap();
    
    let path = Path::new(bn.as_str());

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}", why),
        Ok(file) => file,
    };

    file.write_all(&image_data[..]).unwrap();
}


fn process_results(json_res:&JsonValue, thumbnails:bool, list_only:bool, search:&str, only_new:bool) {

    print_header();
    vprintln!("{} images found", json_res["items"].len());
    for i in 0..json_res["items"].len() {
        let image = &json_res["items"][i];
        
        // If this image is a thumbnail and we're ignoring those, then ignore it.
        if image["is_thumbnail"].as_bool().unwrap() && ! thumbnails {
            continue;
        }

        // If we're searching for a substring and this image doesn't match, skip it.
        if search != "" && image["imageid"].as_str().unwrap().find(&search) == None {
            continue;
        }

        print_image(image);

        if !list_only {
            fetch_image(image, only_new);
        }
        
    }
}

#[allow(dead_code)]
pub struct MSLInstrumentMap {
    pub map: HashMap<&'static str, Vec<&'static str>>
}

pub fn make_instrument_map() -> MSLInstrumentMap {
    MSLInstrumentMap{map: 
        [
            ("HAZ_FRONT", vec!["FHAZ_RIGHT_A", "FHAZ_LEFT_A", "FHAZ_RIGHT_B", "FHAZ_LEFT_B"]), 
            ("HAZ_REAR", vec!["RHAZ_RIGHT_A", "RHAZ_LEFT_A", "RHAZ_RIGHT_B", "RHAZ_LEFT_B"]), 
            ("NAV_LEFT", vec!["NAV_LEFT_A", "NAV_LEFT_B"]),
            ("NAV_RIGHT", vec!["NAV_RIGHT_A", "NAV_RIGHT_B"]),
            ("CHEMCAM", vec!["CHEMCAM_RMI"]),
            ("MARDI", vec!["MARDI"]),
            ("MAHLI", vec!["MAHLI"]),
            ("MASTCAM", vec!["MAST_LEFT", "MAST_RIGHT"])
        ].iter().cloned().collect()}
}


pub fn remote_fetch(cameras:Vec<String>, num_per_page:i32, page:i32, minsol:i32, maxsol:i32, thumbnails:bool, list_only:bool, search:&str, only_new:bool) {

    let joined_cameras = cameras.join("|");
    let num_per_page_s = format!("{}", num_per_page);
    let page_s = format!("{}", (page - 1));
    let minsol_s = format!("{}:sol:gte", minsol);
    let maxsol_s = format!("{}:sol:lte", maxsol);

    let params = vec![
        vec!["condition_1", "msl:mission"],
        vec!["per_page", num_per_page_s.as_str()],
        vec!["page", page_s.as_str()],
        vec!["order", "sol desc,instrument_sort asc,sample_type_sort asc, date_taken desc"],
        vec!["search", joined_cameras.as_str()],
        vec!["condition_2", minsol_s.as_str()],
        vec!["condition_3", maxsol_s.as_str()]
    ];

    let uri = constants::url::MSL_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri);

    for p in params {
        req.param(p[0], p[1]);
    }

    match req.fetch() {
        Ok(v) => process_results(&v, thumbnails, list_only, search, only_new),
        Err(_e) => eprintln!("Error fetching data from remote server")
    }

}