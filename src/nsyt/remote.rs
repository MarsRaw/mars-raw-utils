use crate::{
    constants, 
    jsonfetch, 
    error,
    util::*,
    nsyt::metadata::*,
    nsyt::latest,
    metadata::convert_to_std_metadata,
    path
};


pub fn print_header() {
    println!("{:37} {:15} {:6} {:20} {:27} {:7} {:10}", 
                    "ID", 
                    "Instrument",
                    "Sol",
                    "Image Date (UTC)",
                    "Image Date (Mars)",
                    "Thumb",
                    "Present"
                );
}

fn print_image(image:&Image) {
    println!("{:37} {:15} {:<6} {:20} {:27} {:7} {:10}", 
                    image.imageid, 
                    image.instrument.to_uppercase(),
                    format!("{:<6}", image.sol), // This is such a hack...
                    &image.date_taken[..16],
                    image.extended.localtime,
                    if image.is_thumbnail { constants::status::YES } else { constants::status::NO },
                    if image_exists_on_filesystem(&image.url) { constants::status::YES } else { constants::status::NO }
                );
}

fn process_results(results:&NsytApiResults, thumbnails:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32>  {
    let mut valid_img_count = 0;
    for image in results.items.iter() {
        // If this image is a thumbnail and we're ignoring those, then ignore it.
        if image.is_thumbnail && ! thumbnails {
            continue;
        }

        // If we're searching for a substring and this image doesn't match, skip it.
        if !search.is_empty() && image.imageid.find(&search) == None {
            continue;
        }

        valid_img_count += 1;
        print_image(&image);

        if !list_only {
            match fetch_image(&image.url, only_new) {
                Ok(_) => (),
                Err(e) => return Err(e)
            };
            let image_base_name = path::basename(image.url.as_str());
            match save_image_json(&image_base_name, &convert_to_std_metadata(image), only_new){
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
            ("IDC", vec!["idc"]),
            ("ICC", vec!["icc"]),
        ].iter().cloned().collect()}
}

fn submit_query(cameras:&[String], num_per_page:i32, page:Option<i32>, minsol:i32, maxsol:i32) -> error::Result<String> {

    let mut params = vec![
        stringvec("condition_1", "insight:mission"),
        stringvec_b("per_page", format!("{}", num_per_page)),
        stringvec("order", "sol desc,instrument_sort asc,sample_type_sort asc, date_taken desc"),
        stringvec_b("search", cameras.join("|")),
        stringvec_b("condition_2", format!("{}:sol:gte", minsol)),
        stringvec_b("condition_3", format!("{}:sol:lte", maxsol))
    ];

    if let Some(p) = page {
        params.push(stringvec_b("page", format!("{}", p)));
    }

    let uri = constants::url::NSYT_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri);

    for p in params {
        req.param(p[0].as_str(), p[1].as_str());
    }

    req.fetch_str()
}

pub fn fetch_page(cameras:&[String], num_per_page:i32, page:i32, minsol:i32, maxsol:i32, thumbnails:bool, list_only:bool, search:&str, only_new:bool) -> error::Result<i32> {
    match submit_query(&cameras, num_per_page, Some(page), minsol, maxsol) {
        Ok(v) => {
            let res: NsytApiResults = serde_json::from_str(v.as_str()).unwrap();
            process_results(&res, thumbnails, list_only, search, only_new)
        },
        Err(e) => Err(e)
    }
}

#[derive(Debug, Clone)]
pub struct NsytRemoteStats {
    pub more: bool,
    pub total: i32,
    pub page: i32,
    pub per_page: i32
}

pub fn fetch_stats(cameras:&Vec<String>, minsol:i32, maxsol:i32) -> error::Result<NsytRemoteStats> {
    match submit_query(&cameras, 0, Some(0), minsol, maxsol) {
        Ok(v) => {
            let res: NsytApiResults = serde_json::from_str(v.as_str()).unwrap();
            Ok(NsytRemoteStats{
                more:res.more,
                total:res.total as i32,
                page:res.page as i32,
                per_page:res.per_page as i32
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


pub fn fetch_latest() -> error::Result<latest::LatestData> {
    let uri = constants::url::NSYT_LATEST_WEBSERVICE_URL;

    let req = jsonfetch::JsonFetcher::new(uri);
    match req.fetch_str() {
        Ok(v) => {
            let res: latest::Latest = serde_json::from_str(v.as_str()).unwrap();
            if res.success {
                Ok(res.latest_data)
            } else {
                Err("Server error")
            }
        },
        Err(e) => Err(e)
    }
}
