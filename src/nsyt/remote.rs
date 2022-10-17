use crate::{
    constants, jsonfetch, metadata::convert_to_std_metadata, nsyt::latest, nsyt::metadata::*, path,
    util::*,
};

use anyhow::{anyhow, Result};

pub fn print_header() {
    println!(
        "{:37} {:15} {:6} {:20} {:27} {:7} {:10}",
        "ID", "Instrument", "Sol", "Image Date (UTC)", "Image Date (Mars)", "Thumb", "Present"
    );
}

fn print_image(output_path: &str, image: &Image) {
    let p = format!("{}/{}", output_path, path::basename(&image.url));

    println!(
        "{:37} {:15} {:<6} {:20} {:27} {:7} {:10}",
        image.imageid,
        image.instrument.to_uppercase(),
        image.sol,
        &image.date_taken[..16],
        image.extended.localtime,
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
    );
}

fn search_empty_or_has_match(image_id: &str, search: &[String]) -> bool {
    if search.is_empty() {
        return true;
    }

    for i in search.iter() {
        if image_id.contains(i) {
            return true;
        }
    }
    false
}
async fn process_results(
    results: &NsytApiResults,
    thumbnails: bool,
    list_only: bool,
    search: &[String],
    only_new: bool,
    output_path: &str,
) -> i32 {
    let mut valid_img_count = 0;
    for image in results.items.iter() {
        // If this image is a thumbnail and we're ignoring those, then ignore it.
        if image.is_thumbnail && !thumbnails {
            continue;
        }

        // If we're searching for a substring and this image doesn't match, skip it.
        if !search_empty_or_has_match(&image.imageid, search) {
            continue;
        }
        valid_img_count += 1; //ITM is an anti-pattern. TODO: enumerate(), and have the 'e' fall out.
        print_image(output_path, image);

        if !list_only {
            _ = fetch_image(&image.url, only_new, Some(output_path)).await;
            let image_base_name = path::basename(image.url.as_str());
            _ = save_image_json(
                &image_base_name,
                &convert_to_std_metadata(image),
                only_new,
                Some(output_path),
            );
        }
    }

    valid_img_count
}

pub fn make_instrument_map() -> InstrumentMap {
    InstrumentMap {
        map: [("IDC", vec!["idc"]), ("ICC", vec!["icc"])]
            .iter()
            .cloned()
            .collect(),
    }
}

async fn submit_query(
    cameras: &[String],
    num_per_page: i32,
    page: Option<i32>,
    minsol: i32,
    maxsol: i32,
) -> Result<String, reqwest::Error> {
    let mut params = vec![
        stringvec("condition_1", "insight:mission"),
        stringvec_b("per_page", format!("{}", num_per_page)),
        stringvec(
            "order",
            "sol desc,instrument_sort asc,sample_type_sort asc, date_taken desc",
        ),
        stringvec_b("search", cameras.join("|")),
        stringvec_b("condition_2", format!("{}:sol:gte", minsol)),
        stringvec_b("condition_3", format!("{}:sol:lte", maxsol)),
    ];

    if let Some(p) = page {
        params.push(stringvec_b("page", format!("{}", p)));
    }

    let uri = constants::url::NSYT_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri).unwrap(); //TODO: thiserror crate

    for p in params {
        req.param(p[0].as_str(), p[1].as_str());
    }

    req.fetch_str().await
}

pub async fn fetch_page(
    cameras: &[String],
    num_per_page: i32,
    page: i32,
    minsol: i32,
    maxsol: i32,
    thumbnails: bool,
    list_only: bool,
    search: &[String],
    only_new: bool,
    output_path: &str,
) -> Result<i32> {
    match submit_query(cameras, num_per_page, Some(page), minsol, maxsol).await {
        Ok(v) => match serde_json::from_str(&v) {
            Ok(res) => {
                Ok(
                    process_results(&res, thumbnails, list_only, search, only_new, output_path)
                        .await,
                )
            }
            // NOTE: The anyhow! macro is glorious for making on-the-fly errors in application code, there's a sister libary called thiserror which is for making explicit libary code error types.
            Err(e) => Err(anyhow!("Serde parsing from_str failed. {}", e)),
        },
        Err(e) => Err(anyhow!("Serde parsing from_str failed. {}", e)),
    }
}

#[derive(Debug, Clone)]
pub struct NsytRemoteStats {
    pub more: bool,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
}

pub async fn fetch_stats(cameras: &[String], minsol: i32, maxsol: i32) -> Result<NsytRemoteStats> {
    match submit_query(cameras, 0, Some(0), minsol, maxsol).await {
        Ok(v) => {
            let res: NsytApiResults = serde_json::from_str(v.as_str()).unwrap();
            Ok(NsytRemoteStats {
                more: res.more,
                total: res.total as i32,
                page: res.page as i32,
                per_page: res.per_page as i32,
            })
        }
        Err(_) => Err(anyhow!(
            "Unable to create NsytRemoteStats from submitted query."
        )),
    }
}

pub async fn fetch_all(
    cameras: &[String],
    num_per_page: i32,
    minsol: i32,
    maxsol: i32,
    thumbnails: bool,
    list_only: bool,
    search: &[String],
    only_new: bool,
    output_path: &str,
) -> Result<i32> {
    let stats = match fetch_stats(cameras, minsol, maxsol).await {
        Ok(s) => Ok(s),
        Err(e) => Err(anyhow!("unable to fetch statistics:\n{}", e)),
    }?;

    let pages = (stats.total as f32 / num_per_page as f32).ceil() as i32;

    let mut count = 0;
    for page in 0..pages {
        _ = match fetch_page(
            cameras,
            num_per_page,
            page,
            minsol,
            maxsol,
            thumbnails,
            list_only,
            search,
            only_new,
            output_path,
        )
        .await
        {
            Ok(c) => {
                count += c;
                Ok(())
            }
            Err(e) => Err(anyhow!("{}", e)),
        };
    }

    Ok(count)
}
pub async fn remote_fetch(
    cameras: &[String],
    num_per_page: i32,
    page: Option<i32>,
    minsol: i32,
    maxsol: i32,
    thumbnails: bool,
    list_only: bool,
    search: &[String],
    only_new: bool,
    output_path: &str,
) -> Result<i32> {
    match page {
        Some(p) => {
            fetch_page(
                cameras,
                num_per_page,
                p,
                minsol,
                maxsol,
                thumbnails,
                list_only,
                search,
                only_new,
                output_path,
            )
            .await
        }
        None => {
            fetch_all(
                cameras,
                num_per_page,
                minsol,
                maxsol,
                thumbnails,
                list_only,
                search,
                only_new,
                output_path,
            )
            .await
        }
    }
}

pub async fn fetch_latest() -> Result<latest::LatestData> {
    let uri = constants::url::NSYT_LATEST_WEBSERVICE_URL;

    let req = jsonfetch::JsonFetcher::new(uri)?;
    let res: latest::Latest = serde_json::from_str(&req.fetch_str().await?)?;
    if !res.success {
        return Err(anyhow!("unable to fetch latest."));
    }
    Ok(res.latest_data)
}
