use mars_raw_utils::httpfetch;

#[tokio::test]
#[ignore]
async fn test_text_fetch() {
    let hf = httpfetch::HttpFetcher::new("http://echo.jsontest.com/key/value/one/two").unwrap();
    let res = hf.into_string().await.unwrap();

    assert_eq!(
        res,
        "{
   \"one\": \"two\",
   \"key\": \"value\"
}
"
    );
}

#[tokio::test]
#[ignore]
async fn test_text_fetch_with_params() {
    let mut hf = httpfetch::HttpFetcher::new("http://validate.jsontest.com/").unwrap();
    _ = hf.param("json", "{\"foo\":\"bar\"}");
    let res = hf.into_string().await.unwrap();

    let j: serde_json::Value = serde_json::from_str(&res).unwrap();
    assert_eq!(j["object_or_array"], "object");
    assert_eq!(j["size"], 1);
}

#[tokio::test]
#[ignore]
async fn test_bin_fetch() {
    let hf = httpfetch::HttpFetcher::new("http://echo.jsontest.com/key/value/one/two").unwrap();
    let res = hf.into_bytes().await.unwrap();

    assert_eq!(
        std::str::from_utf8(&res[..]).unwrap(),
        "{
   \"one\": \"two\",
   \"key\": \"value\"
}
"
    );
}

// Redundant.
// #[tokio::test]
// #[ignore]
// async fn test_simple_text_fetch() {
//     let res = httpfetch::simple_fetch_text("http://echo.jsontest.com/key/value/one/two").unwrap();

//     assert_eq!(
//         res,
//         "{
//    \"one\": \"two\",
//    \"key\": \"value\"
// }
// "
//     );
// }

#[tokio::test]
#[ignore]
async fn test_simple_bin_fetch() {
    let res = httpfetch::simple_fetch_bin("http://echo.jsontest.com/key/value/one/two")
        .await
        .unwrap();

    assert_eq!(
        std::str::from_utf8(&res[..]).unwrap(),
        "{
   \"one\": \"two\",
   \"key\": \"value\"
}
"
    );
}
