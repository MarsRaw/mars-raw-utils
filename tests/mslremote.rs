use mars_raw_utils::msl::{remote::fetch_latest, remote::remote_fetch};
use mars_raw_utils::remotequery::RemoteQuery;
#[tokio::test]
async fn test_msl_latest() {
    fetch_latest().await.expect("Failed to fetch latest data");
}

#[tokio::test]
async fn test_msl_instrument_fetches() {
    let instruments = vec![
        "MAST_LEFT",
        "MAST_RIGHT",
        "MARDI",
        "MAHLI",
        "NAV_RIGHT_A",
        "NAV_LEFT_A",
        "NAV_RIGHT_B",
        "NAV_LEFT_B",
        "FHAZ_RIGHT_A",
        "FHAZ_LEFT_A",
        "FHAZ_RIGHT_B",
        "FHAZ_LEFT_B",
        "RHAZ_RIGHT_A",
        "RHAZ_LEFT_A",
        "RHAZ_RIGHT_B",
        "RHAZ_LEFT_B",
        "CHEMCAM_RMI",
    ];

    for i in instruments {
        remote_fetch(
            &RemoteQuery {
                cameras: vec![i.into()],
                num_per_page: 5,
                page: Some(0),
                minsol: 3119,
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
        .await
        .unwrap();
    }
}
