// use reqwest::blocking::Client;
// use reqwest::{blocking, StatusCode};

use crate::vprintln;
use reqwest::Client;
use reqwest::Url;
use std::time::Duration;

// Convention is to seperate your stuff from stuff another viewer is likely to know, there's a new
// style team in rust who'll eventaully decide on what is/isn't correct.
use anyhow::Result; // you'll find a lot, if not most these days of CLI apps or bins in general are running anyhow::Results in their return types.
use bytes::Bytes;
use reqwest::Response;
use std::string::String;

pub struct HttpFetcher {
    //uri: String, // Use reqwest::Url.
    client: Client,
    uri: Url,
    timeout: std::time::Duration, //would normally just leave this in the ClientBuilder..
    numparams: u32,
}

const DEFAULT_TIMEOUT: u64 = 60;

// This is rather hacky and minimal.
impl HttpFetcher {
    pub fn new(uri: &str) -> Result<HttpFetcher> {
        Ok(HttpFetcher {
            uri: uri.parse::<Url>()?,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT),
            numparams: 0,
            client: Client::builder()
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
                .build()?,
        })
    }

    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout = Duration::from_secs(seconds);
    }

    // I mean, seriously. How bad is this?
    //checkout Url https://docs.rs/reqwest/latest/reqwest/struct.Url.html, lots of handy builder methods.
    pub fn param(&mut self, key: &str, value: &str) -> Result<()> {
        let q = if self.numparams == 0 { "?" } else { "&" };
        self.uri = format!("{}{}{}={}", self.uri, q, key, value).parse::<Url>()?;
        self.numparams += 1;

        Ok(()) // the () unit type gets optimised away by the compiler.
    }

    // I'd probably just use the .get(), building my client in main.
    // The reqwest::Client is wrapped in an Arc, so you can clone it cheaply, it's designed for reuse --not so much to be instantiated for every single request you want to make.
    async fn fetch(&self) -> Result<Response, reqwest::Error> {
        vprintln!("Request URI: {}", self.uri);
        self.client.get(self.uri.as_str()).send().await
    }

    pub async fn into_bytes(&self) -> Result<Bytes, reqwest::Error> {
        self.fetch().await?.bytes().await
    }

    pub async fn into_string(&self) -> Result<String, reqwest::Error> {
        self.fetch().await?.text().await
    }
}

pub async fn simple_fetch_bin(uri: &str) -> Result<Vec<u8>> {
    let resp = HttpFetcher::new(uri)?.fetch().await?;
    Ok(resp.bytes().await?.to_vec())
}

pub async fn simple_fetch_text(uri: &str) -> Result<String> {
    match String::from_utf8(simple_fetch_bin(uri).await?) {
        Ok(s) => Ok(s),
        Err(why) => Err(why.into()),
    }
}
