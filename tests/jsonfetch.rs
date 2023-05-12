use mars_raw_utils::jsonfetch;

#[tokio::test]
async fn test_json_fetch() {
    let jf = jsonfetch::JsonFetcher::new("http://echo.jsontest.com/key/value/one/two").unwrap();
    let res = jf.fetch().await.unwrap();
    assert_eq!(res["one"], "two");
    assert_eq!(res["key"], "value");
}

#[tokio::test]
async fn test_json_fetch_with_params() {
    let mut jf = jsonfetch::JsonFetcher::new("http://validate.jsontest.com/").unwrap();
    jf.param("json", "{\"foo\":\"bar\"}");
    let res = jf.fetch().await.unwrap();
    assert_eq!(res["object_or_array"], "object");
    assert_eq!(res["size"], 1);
}
