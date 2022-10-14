use reqwest::blocking::Client;
use reqwest::{blocking, StatusCode};

use crate::{constants, vprintln};
use std::time::Duration;

use sciimg::error;

use std::string::String;

pub struct HttpFetcher {
    uri: String,
    timeout: std::time::Duration,
    numparams: u32,
}

const DEFAULT_TIMEOUT: u64 = 60;

// This is rather hacky and minimal.
impl HttpFetcher {
    pub fn new(uri: &str) -> HttpFetcher {
        HttpFetcher {
            uri: String::from(uri),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT),
            numparams: 0,
        }
    }

    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout = Duration::from_secs(seconds);
    }

    // I mean, seriously. How bad is this?
    pub fn param(&mut self, key: &str, value: &str) {
        let q = if self.numparams == 0 { "?" } else { "&" };
        self.uri = format!("{}{}{}={}", self.uri, q, key, value);
        self.numparams += 1;
    }

    fn fetch(&self) -> error::Result<blocking::Response> {
        vprintln!("Request URI: {}", self.uri);
        vprintln!("Timeout set to {} seconds", self.timeout.as_secs());

        let client = Client::builder().timeout(self.timeout).build().unwrap();
        let res = client.get(self.uri.as_str()).send();

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

pub fn simple_fetch_text_with_timeout(
    url: &str,
    timeout_seconds: u64,
) -> error::Result<std::string::String> {
    let mut fetcher = HttpFetcher::new(&url);
    fetcher.set_timeout(timeout_seconds);
    fetcher.fetch_text()
}

pub fn simple_fetch_text(url: &str) -> error::Result<std::string::String> {
    simple_fetch_text_with_timeout(&url, DEFAULT_TIMEOUT)
}

pub fn simple_fetch_bin_with_timeout(url: &str, timeout_seconds: u64) -> error::Result<Vec<u8>> {
    let mut fetcher = HttpFetcher::new(&url);
    fetcher.set_timeout(timeout_seconds);
    fetcher.fetch_bin()
}

pub fn simple_fetch_bin(url: &str) -> error::Result<Vec<u8>> {
    simple_fetch_bin_with_timeout(&url, DEFAULT_TIMEOUT)
}
