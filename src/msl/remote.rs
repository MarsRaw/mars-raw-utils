use crate::{
    constants, jsonfetch,
    metadata::convert_to_std_metadata,
    msl::latest::{Latest, LatestData},
    msl::metadata::*,
    print::do_println,
    remotequery::RemoteQuery,
    util::*,
};

use sciimg::path;

use anyhow::{anyhow, Result};

pub fn print_header() {
    do_println(&format!(
        "{:37} {:15} {:6} {:20} {:27} {:6} {:6} {:7} {:10}",
        "ID",
        "Instrument",
        "Sol",
        "Image Date (UTC)",
        "Image Date (Mars)",
        "Site",
        "Drive",
        "Thumb",
        "Present"
    ));
}

fn null_to_str<T: std::fmt::Display>(o: &Option<T>) -> String {
    match o {
        None => String::from(""),
        Some(v) => {
            format!("{}", v)
        }
    }
}

fn print_image(output_path: &str, image: &ImageRecord) {
    let p = format!("{}/{}", output_path, path::basename(&image.url));

    do_println(&format!(
        "{:37} {:15} {:<6} {:20} {:27} {:6} {:6} {:7} {:10}",
        image.imageid,
        image.instrument,
        image.sol, // This is such a hack...
        &image.date_taken[..16],
        null_to_str(&image.extended.lmst),
        null_to_str(&image.site),
        null_to_str(&image.drive),
        if image.is_thumbnail {
            constants::status::YES
        } else {
            constants::status::NO
        },
        if path::file_exists(&p) {
            constants::status::YES
        } else {
            constants::status::NO
        }
    ));
}

async fn process_results<B: Fn(&ImageRecord)>(
    results: &MslApiResults,
    query: &RemoteQuery,
    on_image_downloaded: B,
) -> usize {
    let mut valid_img_count = 0;
    let images = results.items.iter().filter(|image| {
        !(image.is_thumbnail && !query.thumbnails
            || !query.search.is_empty() && !query.search.iter().any(|i| image.imageid.contains(i)))
    });
    for (idx, image) in images.enumerate() {
        valid_img_count = idx;
        print_image(query.output_path.as_str(), image);
        on_image_downloaded(image);
        if !query.list_only {
            _ = fetch_image(&image.url, query.only_new, Some(query.output_path.as_str())).await;
            let image_base_name = path::basename(image.url.as_str());
            _ = save_image_json(
                &image_base_name,
                &convert_to_std_metadata(image),
                query.only_new,
                Some(query.output_path.as_str()),
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
                vec!["FHAZ_RIGHT_A", "FHAZ_LEFT_A", "FHAZ_RIGHT_B", "FHAZ_LEFT_B"],
            ),
            (
                "HAZ_REAR",
                vec!["RHAZ_RIGHT_A", "RHAZ_LEFT_A", "RHAZ_RIGHT_B", "RHAZ_LEFT_B"],
            ),
            ("NAV_LEFT", vec!["NAV_LEFT_A", "NAV_LEFT_B"]),
            ("NAV_RIGHT", vec!["NAV_RIGHT_A", "NAV_RIGHT_B"]),
            ("CHEMCAM", vec!["CHEMCAM_RMI"]),
            ("MARDI", vec!["MARDI"]),
            ("MAHLI", vec!["MAHLI"]),
            ("MASTCAM", vec!["MAST_LEFT", "MAST_RIGHT"]),
        ]
        .iter()
        .cloned()
        .collect(),
    }
}

async fn submit_query(query: &RemoteQuery) -> Result<String> {
    let mut params = vec![
        stringvec("condition_1", "msl:mission"),
        stringvec_b("per_page", format!("{}", query.num_per_page)),
        stringvec(
            "order",
            "sol desc,instrument_sort asc,sample_type_sort asc, date_taken desc",
        ),
        stringvec_b("search", query.cameras.join("|")),
        stringvec_b("condition_2", format!("{}:sol:gte", query.minsol)),
        stringvec_b("condition_3", format!("{}:sol:lte", query.maxsol)),
    ];

    if let Some(p) = query.page {
        params.push(stringvec_b("page", format!("{}", p)));
    }

    let uri = constants::url::MSL_RAW_WEBSERVICE_URL;

    if let Ok(mut req) = jsonfetch::JsonFetcher::new(uri) {
        for p in params {
            req.param(p[0].as_str(), p[1].as_str());
        }
        return req.fetch_str().await;
    }

    Err(anyhow!("Unable to submit query."))
}

pub async fn fetch_page<A: Fn(usize), B: Fn(&ImageRecord)>(
    query: &RemoteQuery,
    on_total_known: A,
    on_image_downloaded: B,
) -> Result<usize> {
    if let Ok(v) = submit_query(query).await {
        let res: Result<MslApiResults, serde_json::Error> = serde_json::from_str(&v);
        match res {
            Ok(res) => {
                on_total_known(res.total as usize);
                Ok(process_results(&res, query, on_image_downloaded).await)
            }
            // NOTE: The anyhow! macro is glorious for making on-the-fly errors in application code, there's a sister libary called thiserror which is for making explicit libary code error types.
            Err(e) => Err(anyhow!("Serde parsing from_str failed. {}", e)),
        }
    } else {
        Err(anyhow!("Query submission failed"))
    }
}

#[derive(Debug, Clone)]
pub struct MslRemoteStats {
    pub more: bool,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
}

pub async fn fetch_stats(query: &RemoteQuery) -> Result<MslRemoteStats> {
    match submit_query(query).await {
        Ok(v) => {
            let res: MslApiResults = serde_json::from_str(v.as_str()).unwrap();
            Ok(MslRemoteStats {
                more: res.more,
                total: res.total as i32,
                page: res.page as i32,
                per_page: res.per_page as i32,
            })
        }
        Err(e) => Err(e),
    }
}

pub async fn fetch_latest() -> Result<LatestData> {
    let uri = constants::url::MSL_LATEST_WEBSERVICE_URL;

    let req = jsonfetch::JsonFetcher::new(uri)?;
    match req.fetch_str().await {
        Ok(v) => {
            let res: Latest = serde_json::from_str(v.as_str())?;
            if res.success {
                Ok(res.latest_data)
            } else {
                Err(anyhow!("Server error"))
            }
        }
        Err(_e) => Err(anyhow!("JsonFetcher fetch_str() failed")),
    }
}

pub async fn fetch_all<A: Fn(usize) + Copy, B: Fn(&ImageRecord) + Copy>(
    query: &RemoteQuery,
    on_total_known: A,
    on_image_downloaded: B,
) -> Result<usize> {
    let stats = match fetch_stats(query).await {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    on_total_known(stats.total as usize);
    let pages = (stats.total as f32 / query.num_per_page as f32).ceil() as i32;

    let mut count = 0;
    for page in 0..pages {
        let mut q: RemoteQuery = query.clone();
        q.page = Some(page);
        match fetch_page(&q, on_total_known, on_image_downloaded).await {
            Ok(c) => {
                count += c;
            }
            Err(e) => return Err(e),
        };
    }

    Ok(count)
}

pub async fn remote_fetch<A: Fn(usize) + Copy, B: Fn(&ImageRecord) + Copy>(
    query: &RemoteQuery,
    on_total_known: A,
    on_image_downloaded: B,
) -> Result<usize> {
    if query.page.is_some() {
        fetch_page(query, on_total_known, on_image_downloaded).await
    } else {
        fetch_all(query, on_total_known, on_image_downloaded).await
    }
}
