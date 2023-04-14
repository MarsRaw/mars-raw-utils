use crate::{
    constants, jsonfetch, m20::latest, m20::metadata::*, metadata::convert_to_std_metadata,
    remotequery::RemoteQuery, util::*,
};
use sciimg::path;

use anyhow::{anyhow, Result};

pub fn print_header() {
    println!(
        "{:54} {:25} {:6} {:27} {:27} {:6} {:6} {:7} {:10}",
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

fn print_image(output_path: &str, image: &ImageRecord) {
    let p = format!(
        "{}/{}",
        output_path,
        path::basename(&image.image_files.full_res)
    );

    println!(
        "{:54} {:25} {:>6} {:27} {:27} {:>6} {:>6} {:7} {:10}",
        image.imageid,
        image.camera.instrument,
        image.sol,
        image.date_taken_utc, //[..16],
        image.date_taken_mars,
        image.site,
        image.drive,
        if image.sample_type == "Thumbnail" {
            constants::status::YES
        } else {
            constants::status::NO
        },
        if path::file_exists(&p) {
            constants::status::YES
        } else {
            constants::status::NO
        }
    );
}

async fn process_results(results: &M20ApiResults, query: &RemoteQuery) -> usize {
    let mut valid_img_count = 0;
    let images = results.images.iter().filter(|image| {
        !(image.sample_type == "Thumbnail" && !query.thumbnails
            || !query.search.is_empty() && !query.search.iter().any(|i| image.imageid.contains(i)))
    });
    for (idx, image) in images.enumerate() {
        valid_img_count = idx;
        print_image(query.output_path.as_ref(), image);
        if !query.list_only {
            _ = fetch_image(
                &image.image_files.full_res,
                query.only_new,
                Some(query.output_path.as_ref()),
            )
            .await;
            let image_base_name = path::basename(image.image_files.full_res.as_str());
            _ = save_image_json(
                &image_base_name,
                &convert_to_std_metadata(image),
                query.only_new,
                Some(query.output_path.as_ref()),
            );
        }
    }
    valid_img_count
}

pub fn make_instrument_map() -> InstrumentMap {
    InstrumentMap {
        map: [
            (
                "HAZ_FRONT",
                vec![
                    "FRONT_HAZCAM_LEFT_A",
                    "FRONT_HAZCAM_LEFT_B",
                    "FRONT_HAZCAM_RIGHT_A",
                    "FRONT_HAZCAM_RIGHT_B",
                ],
            ),
            ("SUPERCAM", vec!["SUPERCAM_RMI"]),
            ("HAZ_REAR", vec!["REAR_HAZCAM_LEFT", "REAR_HAZCAM_RIGHT"]),
            ("NAVCAM", vec!["NAVCAM_LEFT", "NAVCAM_RIGHT"]),
            ("MASTCAM", vec!["MCZ_LEFT", "MCZ_RIGHT"]),
            (
                "EDLCAM",
                vec![
                    "EDL_DDCAM",
                    "EDL_PUCAM1",
                    "EDL_PUCAM2",
                    "EDL_RUCAM",
                    "EDL_RDCAM",
                    "LCAM",
                ],
            ),
            ("WATSON", vec!["SHERLOC_WATSON"]),
            ("HELI_NAV", vec!["HELI_NAV"]),
            ("HELI_RTE", vec!["HELI_RTE"]),
            ("CACHECAM", vec!["CACHECAM"]),
            ("PIXL", vec!["PIXL_MCC"]),
            ("SKYCAM", vec!["SKYCAM"]),
            ("SHERLOC", vec!["SHERLOC_ACI"]),
        ]
        .iter()
        .cloned()
        .collect(),
    }
}

async fn submit_query(query: &RemoteQuery) -> Result<String> {
    let joined_cameras = query.cameras.join("|");

    let mut category = "mars2020";
    if query.cameras.contains(&String::from("HELI_NAV"))
        || query.cameras.contains(&String::from("HELI_RTE"))
    {
        category = "mars2020,ingenuity";
    }

    let mut params = vec![
        stringvec("feed", "raw_images"),
        stringvec("category", category),
        stringvec("feedtype", "json"),
        stringvec("ver", "1.2"),
        stringvec_b("num", format!("{}", query.num_per_page)),
        stringvec("order", "sol desc"),
        stringvec_b("search", joined_cameras),
        stringvec_b("condition_2", format!("{}:sol:gte", query.minsol)),
        stringvec_b("condition_3", format!("{}:sol:lte", query.maxsol)),
    ];

    if let Some(p) = query.page {
        params.push(stringvec_b("page", format!("{}", p)));
    }

    let mut extended: Vec<String> = vec![];
    if query.thumbnails {
        extended.push("sample_type::thumbnail".into());
    } else {
        extended.push("sample_type::full".into());
    }

    if query.movie_only {
        extended.push("product_id::ecv".into());
    }

    query
        .product_types
        .iter()
        .for_each(|p| extended.push(format!("product_id::{}", p)));

    if !extended.is_empty() {
        params.push(stringvec_b("extended", extended.join(",")));
    }

    let uri = constants::url::M20_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri)?;

    for p in params {
        req.param(p[0].as_str(), p[1].as_str());
    }

    Ok(req.fetch_str().await?)
}

pub async fn fetch_page(query: &RemoteQuery) -> Result<usize> {
    match submit_query(query).await {
        Ok(v) => {
            let res: M20ApiResults = serde_json::from_str(v.as_str())?;
            Ok(process_results(&res, query).await)
        }
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone)]
pub struct M20RemoteStats {
    pub error_message: String,
    pub total_results: i32,
    pub page: i32,
    pub total_images: i32,
}

pub async fn fetch_stats(query: &RemoteQuery) -> Result<M20RemoteStats> {
    match submit_query(query).await {
        Ok(v) => {
            let res: M20ApiResults = serde_json::from_str(v.as_str()).unwrap();
            Ok(M20RemoteStats {
                error_message: String::from(""),
                total_results: res.total_results as i32,
                page: res.page as i32,
                total_images: res.total_images as i32,
            })
        }
        Err(e) => Err(e),
    }
}

pub async fn fetch_all(query: &RemoteQuery) -> Result<usize> {
    let stats = match fetch_stats(query).await {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let pages = (stats.total_results as f32 / query.num_per_page as f32).ceil() as i32;

    let mut count = 0;
    for page in 0..pages {
        let mut q: RemoteQuery = query.clone();
        q.page = Some(page);
        match fetch_page(&q).await {
            Ok(c) => {
                count += c;
            }
            Err(e) => return Err(e),
        };
    }

    // There's a weird mismatch in the number of results reported by the API and the number
    // we're counting in the results...  (ex: MCZ_RIGHT, Sol 58, movie frames)
    //println!("{:?}, pages: {}", stats ,pages);
    Ok(count)
}

pub async fn remote_fetch(query: &RemoteQuery) -> Result<usize> {
    if query.page.is_some() {
        fetch_page(query).await
    } else {
        fetch_all(query).await
    }
}

pub async fn fetch_latest() -> Result<latest::LatestData> {
    let uri = constants::url::M20_LATEST_WEBSERVICE_URL;

    let req = jsonfetch::JsonFetcher::new(uri)?;
    match req.fetch_str().await {
        Ok(v) => {
            let res: latest::LatestData = serde_json::from_str(v.as_str()).unwrap();
            Ok(res)
        }
        Err(e) => Err(anyhow!("Serde parsing from_str failed. {}", e)),
    }
}
