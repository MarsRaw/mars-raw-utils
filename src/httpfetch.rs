
use reqwest::{StatusCode, blocking};

use crate::{constants, vprintln, error};
use std::string::String;

pub struct HttpFetcher {
    uri : String,
    numparams: u32
}

// This is rather hacky and minimal.
impl HttpFetcher {

    pub fn new(uri:&str) -> HttpFetcher {
        HttpFetcher{
            uri:String::from(uri),
            numparams:0
        }
    }

    // I mean, seriously. How bad is this?
    pub fn param(&mut self, key:&str, value:&str) {
        let q = if self.numparams == 0 { "?" } else { "&" };
        self.uri = format!("{}{}{}={}", self.uri, q, key, value);
        self.numparams += 1;
    }


    fn fetch(&self) -> error::Result<blocking::Response>{
        vprintln!("Request URI: {}", self.uri);

        let res = blocking::get(self.uri.as_str());

        match res {
            Err(_e) => Err(constants::status::REMOTE_SERVER_ERROR),
            Ok(v) => {
                if v.status() != StatusCode::OK {
                    // Should return a more specific error...
                    Err(constants::status::REMOTE_SERVER_ERROR)
                } else {
                    Ok(v)
                }
            }
        }
    }

    pub fn fetch_text(&self) -> error::Result<std::string::String> {
        let res = self.fetch();
        if let Ok(v) = res {
            if let Ok(t) = v.text() {
                Ok(t)
            } else {
                Err(constants::status::REMOTE_SERVER_ERROR)
            }
        } else {
            Err(constants::status::REMOTE_SERVER_ERROR)
        }
    }

    pub fn fetch_bin(&self) -> error::Result<Vec<u8>> {
        let res = self.fetch();
        if let Ok(v) = res {
            if let Ok(b) = v.bytes() {
                Ok(b.to_vec())
            } else {
                Err(constants::status::REMOTE_SERVER_ERROR)
            }
        } else {
            Err(constants::status::REMOTE_SERVER_ERROR)
        }
    }
}

pub fn simple_fetch_text(url:&str) -> error::Result<std::string::String> {
    let fetcher = HttpFetcher::new(&url);
    fetcher.fetch_text()
}

pub fn simple_fetch_bin(url:&str) -> error::Result<Vec<u8>> {
    let fetcher = HttpFetcher::new(&url);
    fetcher.fetch_bin()
}