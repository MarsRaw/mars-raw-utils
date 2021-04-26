
use crate::{
    constants, 
    jsonfetch, 
    error,
    util::*
};

use json::{
    JsonValue
};

use std::collections::HashMap;


pub fn print_header() {
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
    let image_url = &image["url"].as_str().unwrap();

    println!("{:37} {:15} {:6} {:20} {:27} {:6} {:6} {:7} {:10}", 
                    image["imageid"], 
                    image["instrument"],
                    format!("{:6}", image["sol"]), // This is such a hack...
                    &image["date_taken"].as_str().unwrap()[..16],
                    null_to_str(&image["extended"]["lmst"]),
                    format!("{:6}", null_to_str(&image["site"])),
                    format!("{:6}", null_to_str(&image["drive"])),
                    if image["is_thumbnail"].as_bool().unwrap() { constants::status::YES } else { constants::status::NO },
                    if image_exists_on_filesystem(&image_url) { constants::status::YES } else { constants::status::NO }
                );
}


fn process_results(json_res:&JsonValue, thumbnails:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32>  {
    let mut valid_img_count = 0;
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

        valid_img_count += 1;
        print_image(image);

        if !list_only {
            let image_url = &image["url"].as_str().unwrap();
            match fetch_image(image_url, only_new) {
                Ok(_) => (),
                Err(e) => return Err(e)
            };
        }
        
    }

    Ok(valid_img_count)
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


fn submit_query(cameras:&Vec<String>, num_per_page:i32, page:Option<i32>, minsol:i32, maxsol:i32) -> error::Result<json::JsonValue> {

    let mut params = vec![
        stringvec("condition_1", "msl:mission"),
        stringvec_b("per_page", format!("{}", num_per_page)),
        stringvec("order", "sol desc,instrument_sort asc,sample_type_sort asc, date_taken desc"),
        stringvec_b("search", cameras.join("|")),
        stringvec_b("condition_2", format!("{}:sol:gte", minsol)),
        stringvec_b("condition_3", format!("{}:sol:lte", maxsol))
    ];

    match page {
        Some(p) => {
            params.push(stringvec_b("page", format!("{}", p)));
        },
        None => ()
    };

    let uri = constants::url::MSL_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri);

    for p in params {
        req.param(p[0].as_str(), p[1].as_str());
    }

    req.fetch()
}


pub fn fetch_page(cameras:&Vec<String>, num_per_page:i32, page:i32, minsol:i32, maxsol:i32, thumbnails:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32> {
    match submit_query(&cameras, num_per_page, Some(page), minsol, maxsol) {
        Ok(v) => {
            process_results(&v, thumbnails, list_only, search, only_new)
        },
        Err(e) => Err(e)
    }
}

#[derive(Debug, Clone)]
pub struct MslRemoteStats {
    pub more: bool,
    pub total: i32,
    pub page: i32,
    pub per_page: i32
}

pub fn fetch_stats(cameras:&Vec<String>, minsol:i32, maxsol:i32) -> error::Result<MslRemoteStats> {
    match submit_query(&cameras, 0, Some(0), minsol, maxsol) {
        Ok(v) => {
            Ok(MslRemoteStats{
                more:v["more"].as_bool().unwrap(),
                total:v["total"].as_i32().unwrap(),
                page:v["page"].as_i32().unwrap(),
                per_page:v["per_page"].as_i32().unwrap()
            })
        },
        Err(e) => Err(e)
    }
}

pub fn fetch_all(cameras:&Vec<String>, num_per_page:i32, minsol:i32, maxsol:i32, thumbnails:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32> {

    let stats = match fetch_stats(&cameras, minsol, maxsol) {
        Ok(s) => s,
        Err(e) => return Err(e)
    };

    let pages = (stats.total as f32 / num_per_page as f32).ceil() as i32;

    let mut count = 0;
    for page in 0..pages {
        match fetch_page(&cameras, num_per_page, page, minsol, maxsol, thumbnails, list_only, search, only_new) {
            Ok(c) => {
                count = count + c;
            },
            Err(e) => return Err(e)
        };
    }

    //println!("{:?}, pages: {}", stats ,pages);
    Ok(count)
}


pub fn remote_fetch(cameras:&Vec<String>, num_per_page:i32, page:Option<i32>, minsol:i32, maxsol:i32, thumbnails:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32> {
    match page {
        Some(p) => {
            fetch_page(&cameras, num_per_page, p, minsol, maxsol, thumbnails, list_only, search, only_new)
        },
        None => {
            fetch_all(&cameras, num_per_page, minsol, maxsol, thumbnails, list_only, search, only_new)
        }
    }
}