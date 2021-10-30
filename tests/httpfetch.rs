use mars_raw_utils::{
    httpfetch
};

use serde_json;

#[test]
#[ignore]
fn test_text_fetch() {

    let hf = httpfetch::HttpFetcher::new("http://echo.jsontest.com/key/value/one/two");
    let res = hf.fetch_text().unwrap();

    assert_eq!(res, "{
   \"one\": \"two\",
   \"key\": \"value\"
}
");
}


#[test]
#[ignore]
fn test_text_fetch_with_params() {

    let mut hf = httpfetch::HttpFetcher::new("http://validate.jsontest.com/");
    hf.param("json", "{\"foo\":\"bar\"}");
    let res = hf.fetch_text().unwrap();

    let j : serde_json::Value = serde_json::from_str(&res).unwrap();
    assert_eq!(j["object_or_array"], "object");
    assert_eq!(j["size"], 1);
}

#[test]
#[ignore]
fn test_bin_fetch() {

    let hf = httpfetch::HttpFetcher::new("http://echo.jsontest.com/key/value/one/two");
    let res = hf.fetch_bin().unwrap();

    assert_eq!(std::str::from_utf8(&res[..]).unwrap(), "{
   \"one\": \"two\",
   \"key\": \"value\"
}
");
}

#[test]
#[ignore]
fn test_simple_text_fetch() {

    let res = httpfetch::simple_fetch_text("http://echo.jsontest.com/key/value/one/two").unwrap();

    assert_eq!(res, "{
   \"one\": \"two\",
   \"key\": \"value\"
}
");
}

#[test]
#[ignore]
fn test_simple_bin_fetch() {

    let res = httpfetch::simple_fetch_bin("http://echo.jsontest.com/key/value/one/two").unwrap();

    assert_eq!(std::str::from_utf8(&res[..]).unwrap(), "{
   \"one\": \"two\",
   \"key\": \"value\"
}
");
}