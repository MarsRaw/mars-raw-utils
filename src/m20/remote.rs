use crate::{
    constants, 
    jsonfetch, 
    util::*,
    m20::metadata::*,
    m20::latest,
    metadata::convert_to_std_metadata,
    path
};

use sciimg::error;

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


fn print_image(output_path:&str, image:&Image) {
    let p = format!("{}/{}", output_path, path::basename(&image.image_files.full_res));

    println!("{:54} {:25} {:>6} {:27} {:27} {:>6} {:>6} {:7} {:10}", 
                    image.imageid, 
                    image.camera.instrument,
                    format!("{:>6}", image.sol),
                    image.date_taken_utc,//[..16],
                    image.date_taken_mars,
                    format!("{:>6}", image.site),
                    format!("{:>6}", image.drive),
                    if image.sample_type == "Thumbnail" { constants::status::YES } else { constants::status::NO },
                    if path::file_exists(&p) { constants::status::YES } else { constants::status::NO }
                );
}

fn process_results(results:&M20ApiResults, thumbnails:bool, list_only:bool, search:&str, only_new:bool, output_path:&str) -> error::Result<i32> {
    
    let mut valid_img_count = 0;

    for image in results.images.iter() {
        // If this image is a thumbnail and we're ignoring those, then ignore it.
        if image.sample_type == "Thumbnail" && ! thumbnails {
            continue;
        }

        // If we're searching for a substring and this image doesn't match, skip it.
        if !search.is_empty() && image.imageid.find(&search) == None {
            continue;
        }

        valid_img_count += 1;
        print_image(output_path, image);

        if !list_only {
            match fetch_image(&image.image_files.full_res, only_new, Some(output_path)) {
                Ok(_) => (),
                Err(e) => return Err(e)
            };

            let image_base_name = path::basename(image.image_files.full_res.as_str());
            match save_image_json(&image_base_name, &convert_to_std_metadata(image), only_new, Some(output_path)){
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
        ("WATSON", vec!["SHERLOC_WATSON"]),
        ("HELI_NAV", vec!["HELI_NAV"]),
        ("HELI_RTE", vec!["HELI_RTE"]),
        ("CACHECAM", vec!["CACHECAM"]),
        ("PIXL", vec!["PIXL_MCC"]),
        ("SKYCAM", vec!["SKYCAM"])
    ].iter().cloned().collect()}
}




fn submit_query(cameras:&[String], num_per_page:i32, page:Option<i32>, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool) -> error::Result<String> {
    let joined_cameras = cameras.join("|");

    let mut category = "mars2020";
    if cameras.contains(&String::from("HELI_NAV")) || cameras.contains(&String::from("HELI_RTE")) {
        category = "mars2020,ingenuity";
    }

    let mut params = vec![
        stringvec("feed", "raw_images"),
        stringvec("category", category),
        stringvec("feedtype", "json"),
        stringvec("ver", "1.2"),
        stringvec_b("num", format!("{}", num_per_page)),
        stringvec("order", "sol desc"),
        stringvec_b("search", joined_cameras),
        stringvec_b("condition_2", format!("{}:sol:gte", minsol)),
        stringvec_b("condition_3", format!("{}:sol:lte", maxsol))
    ];

    if let Some(p) = page {
        params.push(stringvec_b("page", format!("{}", p)));
    }

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

    req.fetch_str()
}

pub fn fetch_page(cameras:&[String], num_per_page:i32, page:i32, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool, list_only:bool, search:&str, only_new:bool, output_path:&str) -> error::Result<i32> {
    match submit_query(&cameras, num_per_page, Some(page), minsol, maxsol, thumbnails, movie_only) {
        Ok(v) => {
            let res: M20ApiResults = serde_json::from_str(v.as_str()).unwrap();
            process_results(&res, thumbnails, list_only, search, only_new, output_path)
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

pub fn fetch_stats(cameras:&[String], minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool) -> error::Result<M20RemoteStats> {
    match submit_query(&cameras, 0, Some(0), minsol, maxsol, thumbnails, movie_only) {
        Ok(v) => {
            let res: M20ApiResults = serde_json::from_str(v.as_str()).unwrap();
            Ok(M20RemoteStats{
                error_message:String::from(""),
                total_results:res.total_results as i32,
                page:res.page as i32,
                total_images:res.total_images as i32
            })
        },
        Err(e) => Err(e)
    }
}

pub fn fetch_all(cameras:&[String], num_per_page:i32, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool, list_only:bool, search:&str, only_new:bool, output_path:&str) -> error::Result<i32> {

    let stats = match fetch_stats(&cameras, minsol, maxsol, thumbnails, movie_only) {
        Ok(s) => s,
        Err(e) => return Err(e)
    };

    let pages = (stats.total_results as f32 / num_per_page as f32).ceil() as i32;

    let mut count = 0;
    for page in 0..pages {
        match fetch_page(&cameras, num_per_page, page, minsol, maxsol, thumbnails, movie_only, list_only, search, only_new, output_path) {
            Ok(c) => {
                count += c;
            },
            Err(e) => return Err(e)
        };
    }

    // There's a weird mismatch in the number of results reported by the API and the number
    // we're counting in the results...  (ex: MCZ_RIGHT, Sol 58, movie frames)
    //println!("{:?}, pages: {}", stats ,pages);
    Ok(count)
}

pub fn remote_fetch(cameras:&[String], num_per_page:i32, page:Option<i32>, minsol:i32, maxsol:i32, thumbnails:bool, movie_only:bool, list_only:bool, search:&str, only_new:bool, output_path:&str) -> error::Result<i32> {
    match page {
        Some(p) => {
            fetch_page(&cameras, num_per_page, p, minsol, maxsol, thumbnails, movie_only, list_only, search, only_new, output_path)
        },
        None => {
            fetch_all(&cameras, num_per_page, minsol, maxsol, thumbnails, movie_only, list_only, search, only_new, output_path)
        }
    }
}



pub fn fetch_latest() -> error::Result<latest::LatestData> {
    let uri = constants::url::M20_LATEST_WEBSERVICE_URL;

    let req = jsonfetch::JsonFetcher::new(uri);
    match req.fetch_str() {
        Ok(v) => {
            let res: latest::LatestData = serde_json::from_str(v.as_str()).unwrap();
            Ok(res)
        },
        Err(e) => Err(e)
    }
}