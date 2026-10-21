use dateparser::DateTimeUtc;
use mars_raw_utils::prelude::*;
use mars_raw_utils::psyche::fetch::PsycheFetch as PsycheFetchClient;
use mars_raw_utils::remotequery::RemoteQuery;

#[tokio::test]
#[should_panic]
async fn test_psyche_latest() {
    remotequery::get_latest(&PsycheFetchClient::new())
        .await
        .expect("Failed to fetch latest data");
}

#[tokio::test]
async fn test_psyche_instrument_fetches() {
    let instruments = vec!["A", "B"];
    for i in instruments {
        eprintln!("Testing fetch for {}", i);
        _ = remotequery::perform_fetch(
            &PsycheFetchClient::new(),
            &RemoteQuery {
                cameras: vec![i.into()],
                num_per_page: 5,
                page: Some(0),
                minsol: 0,
                maxsol: 0,
                thumbnails: false,
                movie_only: false,
                list_only: true,
                search: vec![],
                only_new: false,
                product_types: vec![],
                output_path: String::from(""),
                filter: None,
                filter_num: None,
                filter_wavelenth: None,
                start_date: Some("2026-05-16".parse::<DateTimeUtc>().unwrap().0),
                end_date: Some("2026-05-16".parse::<DateTimeUtc>().unwrap().0),
            },
            |_| {},
            |_| {},
            None,
        )
        .await;
    }
}
