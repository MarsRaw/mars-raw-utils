use mars_raw_utils::prelude::*;
use mars_raw_utils::remotequery::RemoteQuery;

#[tokio::test]
async fn test_nsyt_latest() {
    remotequery::get_latest(Mission::InSight)
        .await
        .expect("Failed to fetch latest data");
}

#[tokio::test]
async fn test_nsyt_instrument_fetches() {
    let instruments = vec!["idc", "icc"];
    for i in instruments {
        eprintln!("Testing fetch for {}", i);
        _ = remotequery::perform_fetch(
            Mission::InSight,
            &RemoteQuery {
                cameras: vec![i.into()],
                num_per_page: 5,
                page: Some(0),
                minsol: 1000,
                maxsol: 1000,
                thumbnails: false,
                movie_only: false,
                list_only: true,
                search: vec![],
                only_new: false,
                product_types: vec![],
                output_path: String::from(""),
            },
            |_| {},
            |_| {},
        )
        .await;
    }
}
