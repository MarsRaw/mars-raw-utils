
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
        let json_text = self.fetcher.fetch_text();

        match json_text {
            Err(_e) => return Err(constants::status::REMOTE_SERVER_ERROR),
            Ok(v) => Ok(json::parse(&v).unwrap())
        }
    }
}