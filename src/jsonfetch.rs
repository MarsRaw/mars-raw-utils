
use serde_json::{
    Value
};
use crate::{constants, error, httpfetch::HttpFetcher};

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

    pub fn fetch(&self) -> error::Result<Value> {
        let json_text = self.fetcher.fetch_text();

        match json_text {
            Err(_e) => return Err(constants::status::REMOTE_SERVER_ERROR),
            Ok(v) => Ok(serde_json::from_str(&v).unwrap())
        }
    }



    pub fn fetch_str(&self) -> error::Result<String> {
        let json_text = self.fetcher.fetch_text();

        match json_text {
            Err(_e) => return Err(constants::status::REMOTE_SERVER_ERROR),
            Ok(v) => Ok(v)
        }
    }
}