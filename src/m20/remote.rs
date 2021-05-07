use crate::{
    constants, 
    jsonfetch, 
    error,
    util::*
};
use json::{
    JsonValue
};

pub fn print_header() {
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
    let image_url = &image["image_files"]["full_res"].as_str().unwrap();
    println!("{:54} {:25} {:6} {:27} {:27} {:6} {:6} {:7} {:10}", 
                    image["imageid"], 
                    image["camera"]["instrument"],
                    format!("{:6}", image["sol"]), // This is such a hack...
                    &image["date_taken_utc"].as_str().unwrap()[..16],
                    image["date_taken_mars"],
                    format!("{:6}", image["site"]),
                    format!("{:6}", image["drive"]),
                    if image["sample_type"] == "Thumbnail" { constants::status::YES } else { constants::status::NO },
                    if image_exists_on_filesystem(&image_url) { constants::status::YES } else { constants::status::NO }
                );
}

fn process_results(json_res:&JsonValue, thumbnails:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32> {
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
            let image_url = &image["image_files"]["full_res"].as_str().unwrap();
            match fetch_image(&image_url, only_new) {
                Ok(_) => (),
                Err(e) => return Err(e)
            };
            match save_image_json(image_url, &image, only_new){
                Ok(_) => (),
                Err(e) => return Err(e)
            };
        }
        
    }

    Ok(valid_img_count)
}

pub fn make_instrument_map() -> InstrumentMap {
    InstrumentMap{map: 
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




fn submit_query(cameras:&Vec<String>, num_per_page:i32, page:Option<i32>, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool) -> error::Result<json::JsonValue> {
    let joined_cameras = cameras.join("|");

    let mut params = vec![
        stringvec("feed", "raw_images"),
        stringvec("category", "mars2020"),
        stringvec("feedtype", "json"),
        stringvec_b("num", format!("{}", num_per_page)),
        stringvec("order", "sol desc"),
        stringvec_b("search", joined_cameras),
        stringvec_b("condition_2", format!("{}:sol:gte", minsol)),
        stringvec_b("condition_3", format!("{}:sol:lte", maxsol))
    ];
    match page {
        Some(p) => {
            params.push(stringvec_b("page", format!("{}", p)));
        },
        None => ()
    };

    if thumbnails {
        params.push(stringvec("extended", "sample_type::thumbnail,"));
    } else if movie_only {
        params.push(stringvec("extended", "sample_type::full,product_id::ecv,"));
    } else {
        params.push(stringvec("extended", "sample_type::full,"));
    }

    let uri = constants::url::M20_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri);

    for p in params {
        req.param(p[0].as_str(), p[1].as_str());
    }

    req.fetch()
}

pub fn fetch_page(cameras:&Vec<String>, num_per_page:i32, page:i32, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32> {
    match submit_query(&cameras, num_per_page, Some(page), minsol, maxsol, thumbnails, movie_only) {
        Ok(v) => {
            process_results(&v, thumbnails, list_only, search, only_new)
        },
        Err(e) => Err(e)
    }
}

#[derive(Debug, Clone)]
pub struct M20RemoteStats {
    pub error_message: String,
    pub total_results: i32,
    pub page: i32,
    pub total_images: i32
}

pub fn fetch_stats(cameras:&Vec<String>, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool) -> error::Result<M20RemoteStats> {
    match submit_query(&cameras, 0, Some(0), minsol, maxsol, thumbnails, movie_only) {
        Ok(v) => {
            Ok(M20RemoteStats{
                error_message:v["error_message"].to_string(),
                total_results:v["total_results"].as_i32().unwrap(),
                page:v["page"].as_i32().unwrap(),
                total_images:v["total_images"].as_i32().unwrap()
            })
        },
        Err(e) => Err(e)
    }
}

pub fn fetch_all(cameras:&Vec<String>, num_per_page:i32, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32> {

    let stats = match fetch_stats(&cameras, minsol, maxsol, thumbnails, movie_only) {
        Ok(s) => s,
        Err(e) => return Err(e)
    };

    let pages = (stats.total_results as f32 / num_per_page as f32).ceil() as i32;

    let mut count = 0;
    for page in 0..pages {
        match fetch_page(&cameras, num_per_page, page, minsol, maxsol, thumbnails, movie_only, list_only, search, only_new) {
            Ok(c) => {
                count = count + c;
            },
            Err(e) => return Err(e)
        };
    }

    // There's a weird mismatch in the number of results reported by the API and the number
    // we're counting in the results...  (ex: MCZ_RIGHT, Sol 58, movie frames)
    //println!("{:?}, pages: {}", stats ,pages);
    Ok(count)
}

pub fn remote_fetch(cameras:&Vec<String>, num_per_page:i32, page:Option<i32>, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32> {
    match page {
        Some(p) => {
            fetch_page(&cameras, num_per_page, p, minsol, maxsol, thumbnails, movie_only, list_only, search, only_new)
        },
        None => {
            fetch_all(&cameras, num_per_page, minsol, maxsol, thumbnails, movie_only, list_only, search, only_new)
        }
    }
}
