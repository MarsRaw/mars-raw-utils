use crate::{
    constants, 
    jsonfetch, 
    httpfetch, 
    path, 
    vprintln
};
use json::{
    JsonValue
};
use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

fn image_exists_on_filesystem(image:&JsonValue) -> bool {
    let image_url = &image["image_files"]["full_res"].as_str().unwrap();
    let bn = path::basename(image_url);
    path::file_exists(bn.as_str())
}

fn print_header() {
    println!("{:54} {:25} {:6} {:27} {:27} {:6} {:6} {:7} {:10}", 
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

fn print_image(image:&JsonValue) {
    println!("{:54} {:25} {:6} {:27} {:27} {:6} {:6} {:7} {:10}", 
                    image["imageid"], 
                    image["camera"]["instrument"],
                    format!("{:6}", image["sol"]), // This is such a hack...
                    &image["date_taken_utc"].as_str().unwrap()[..16],
                    image["date_taken_mars"],
                    format!("{:6}", image["site"]),
                    format!("{:6}", image["drive"]),
                    if image["sample_type"] == "Thumbnail" { constants::status::YES } else { constants::status::NO },
                    if image_exists_on_filesystem(&image) { constants::status::YES } else { constants::status::NO }
                );
}

fn process_results(json_res:&JsonValue, thumbnails:bool, list_only:bool, search:&str, only_new:bool) {
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
            fetch_image(image, only_new);
        }
        
    }

    println!("{} images found", valid_img_count);
}

fn fetch_image(image:&JsonValue, only_new:bool) {
    let image_url = &image["image_files"]["full_res"].as_str().unwrap();
    let bn = path::basename(image_url);

    if image_exists_on_filesystem(&image) && only_new {
        vprintln!("Output file {} exists, skipping", bn);
        return;
    }

    let path = Path::new(bn.as_str());
    
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}", why),
        Ok(file) => file,
    };

    let image_data = httpfetch::simple_fetch_bin(image_url).unwrap();
    file.write_all(&image_data[..]).unwrap();
}

#[allow(dead_code)]
pub struct M20InstrumentMap {
    pub map: HashMap<&'static str, Vec<&'static str>>
}

pub fn make_instrument_map() -> M20InstrumentMap {
    M20InstrumentMap{map: 
    [
        ("HAZ_FRONT", vec!["FRONT_HAZCAM_LEFT_A", "FRONT_HAZCAM_LEFT_B", "FRONT_HAZCAM_RIGHT_A", "FRONT_HAZCAM_RIGHT_B"]),
        ("SUPERCAM", vec!["SUPERCAM_RMI"]),
        ("HAZ_REAR", vec!["REAR_HAZCAM_LEFT", "REAR_HAZCAM_RIGHT"]),
        ("NAVCAM", vec!["NAVCAM_LEFT", "NAVCAM_RIGHT"]),
        ("MASTCAM", vec!["MCZ_LEFT","MCZ_RIGHT"]),
        ("EDLCAM", vec!["EDL_DDCAM", "EDL_PUCAM1", "EDL_PUCAM2", "EDL_RUCAM", "EDL_RDCAM", "LCAM"]),
        ("WATSON", vec!["SHERLOC_WATSON"])
    ].iter().cloned().collect()}
}

pub fn remote_fetch(cameras:Vec<String>, num_per_page:i32, page:i32, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool, list_only:bool, search:&str, only_new:bool) {

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
        Ok(v) => process_results(&v, thumbnails, list_only, search, only_new),
        Err(_e) => eprintln!("Error fetching data from remote server")
    }
}
