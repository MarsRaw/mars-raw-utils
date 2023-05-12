use crate::{
    constants, jsonfetch, metadata::convert_to_std_metadata, nsyt::latest, nsyt::metadata::*,
    remotequery::RemoteQuery, util::*,
};
use sciimg::path;

use anyhow::{anyhow, Result};

pub fn print_header() {
    println!(
        "{:37} {:15} {:6} {:20} {:27} {:7} {:10}",
        "ID", "Instrument", "Sol", "Image Date (UTC)", "Image Date (Mars)", "Thumb", "Present"
    );
}

fn print_image(output_path: &str, image: &ImageRecord) {
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

async fn process_results(results: &NsytApiResults, query: &RemoteQuery) -> usize {
    let mut valid_img_count = 0;
    let images = results.items.iter().filter(|image| {
        !(image.is_thumbnail && !query.thumbnails
            || !query.search.is_empty() && !query.search.iter().any(|i| image.imageid.contains(i)))
    });
    // let iter_count = images.clone().into_iter().count();
    for (idx, image) in images.enumerate() {
        valid_img_count = idx;
        print_image(query.output_path.as_str(), image);
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
        map: [("IDC", vec!["idc"]), ("ICC", vec!["icc"])]
            .iter()
            .cloned()
            .collect(),
    }
}

async fn submit_query(query: &RemoteQuery) -> Result<String> {
    let mut params = vec![
        stringvec("condition_1", "insight:mission"),
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

    let uri = constants::url::NSYT_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri).unwrap(); //TODO: thiserror crate

    for p in params {
        req.param(p[0].as_str(), p[1].as_str());
    }

    req.fetch_str().await
}

pub async fn fetch_page(query: &RemoteQuery) -> Result<usize> {
    match submit_query(query).await {
        Ok(v) => match serde_json::from_str(&v) {
            Ok(res) => Ok(process_results(&res, query).await),
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

pub async fn fetch_stats(query: &RemoteQuery) -> Result<NsytRemoteStats> {
    match submit_query(query).await {
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

pub async fn fetch_all(query: &RemoteQuery) -> Result<usize> {
    let stats = match fetch_stats(query).await {
        Ok(s) => Ok(s),
        Err(e) => Err(anyhow!("unable to fetch statistics:\n{}", e)),
    }?;

    let pages = (stats.total as f32 / query.num_per_page as f32).ceil() as i32;

    let mut count = 0;
    for page in 0..pages {
        let mut q: RemoteQuery = query.clone();
        q.page = Some(page);
        _ = match fetch_page(&q).await {
            Ok(c) => {
                count += c;
                Ok(())
            }
            Err(e) => Err(anyhow!("{}", e)),
        };
    }

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
    let url = constants::url::NSYT_LATEST_WEBSERVICE_URL;

    let req = jsonfetch::JsonFetcher::new(url)?;
    let res: latest::Latest = serde_json::from_str(&req.fetch_str().await?)?;
    if !res.success {
        return Err(anyhow!("unable to fetch latest."));
    }
    Ok(res.latest_data)
}
