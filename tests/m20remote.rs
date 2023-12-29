use mars_raw_utils::m20::fetch::M20Fetch as M20FetchClient;
use mars_raw_utils::prelude::*;
use mars_raw_utils::remotequery::RemoteQuery;

#[tokio::test]
async fn test_m20_latest() {
    remotequery::get_latest(&M20FetchClient::new())
        .await
        .expect("Failed to fetch latest data");
}

#[tokio::test]
async fn test_m20_instrument_fetches() {
    let instruments = vec![
        "FRONT_HAZCAM_LEFT_A",
        "FRONT_HAZCAM_LEFT_B",
        "FRONT_HAZCAM_RIGHT_A",
        "FRONT_HAZCAM_RIGHT_B",
        "HAZ_FRONT",
        "SUPERCAM_RMI",
        "REAR_HAZCAM_LEFT",
        "REAR_HAZCAM_RIGHT",
        "NAVCAM_LEFT",
        "NAVCAM_RIGHT",
        "MCZ_LEFT",
        "MCZ_RIGHT",
        "EDL_DDCAM",
        "EDL_PUCAM1",
        "EDL_PUCAM2",
        "EDL_RUCAM",
        "EDL_RDCAM",
        "LCAM",
        "SHERLOC_WATSON",
    ];

    for i in instruments {
        eprintln!("Testing fetch for {}", i);
        _ = remotequery::perform_fetch(
            &M20FetchClient::new(),
            &RemoteQuery {
                cameras: vec![String::from(i)],
                num_per_page: 5,
                page: Some(0),
                minsol: 70,
                maxsol: 70,
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
