
use json;
use crate::{constants, print, vprintln, error, httpfetch::HttpFetcher};

pub struct JsonFetcher {
    fetcher : HttpFetcher
}

impl JsonFetcher {

    pub fn new(uri:&str) -> JsonFetcher {
        JsonFetcher{
            fetcher:HttpFetcher::new(uri)
        }
    }

    pub fn param(&mut self, key:&str, value:&str) {
        self.fetcher.param(key, value);
    }

    pub fn fetch(&self) -> error::Result<json::JsonValue> {
        let json_text = self.fetcher.fetch_text().unwrap();
        let parsed_json = json::parse(&json_text).unwrap();
        Ok(parsed_json)
    }
}