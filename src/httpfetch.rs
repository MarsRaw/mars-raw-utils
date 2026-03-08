// use reqwest::blocking::Client;
// use reqwest::{blocking, StatusCode};

use reqwest::Client;
use reqwest::Url;
use std::cmp::min;
use std::env;
use std::time::Duration;

// Convention is to seperate your stuff from stuff another viewer is likely to know, there's a new
// style team in rust who'll eventaully decide on what is/isn't correct.
use anyhow::anyhow;
use anyhow::Result; // you'll find a lot, if not most these days of CLI apps or bins in general are running anyhow::Results in their return types.
use futures_util::StreamExt;
use std::string::String;

use crate::constants;
use crate::util::string_is_valid_u64;

pub struct HttpFetcher {
    //uri: String, // Use reqwest::Url.
    client: Client,
    uri: Url,
    timeout: std::time::Duration, //would normally just leave this in the ClientBuilder..
    numparams: u32,
}

const DEFAULT_TIMEOUT: u64 = 60;

fn get_timeout_seconds() -> Result<u64> {
    if let Ok(v) = env::var(constants::env::HTTP_TIMEOUT_VAR) {
        if string_is_valid_u64(v.as_str()) {
            v.parse::<u64>()
                .or(Err(anyhow!("Failed to parse \"{:?}\" to u64", v)))
        } else {
            Ok(DEFAULT_TIMEOUT)
        }
    } else {
        Ok(DEFAULT_TIMEOUT)
    }
}

#[derive(Debug)]
pub struct SimpleHttpResponse {
    pub bytes: Vec<u8>,
    pub status: u16,
}

pub struct SimpleHttpResponseString {
    pub text: String,
    pub status: u16,
}

// This is rather hacky and minimal.
impl HttpFetcher {
    pub fn new(uri: &str) -> Result<HttpFetcher> {
        match get_timeout_seconds() {
            Ok(timeout_seconds) => {
                info!("Using HTTP Timeoout (seconds): {}", timeout_seconds);

                Ok(HttpFetcher {
                    uri: uri.parse::<Url>()?,
                    timeout: Duration::from_secs(timeout_seconds),
                    numparams: 0,
                    client: Client::builder()
                        .timeout(Duration::from_secs(timeout_seconds))
                        .build()?,
                })
            }
            Err(why) => Err(why),
        }
    }

    pub fn set_timeout(&mut self, seconds: u64) {
        info!("Setting http timeout to custom {} seconds", seconds);
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
    async fn fetch(&self) -> Result<SimpleHttpResponse> {
        info!("Request URI: {}", self.uri);
        let res = self.client.get(self.uri.as_str()).send().await?;
        let status = res.status();
        Ok(SimpleHttpResponse {
            bytes: res.bytes().await?.to_vec(),
            status: status.as_u16(),
        })
    }

    async fn fetch_monitored<F: Fn(u64, u64, f32)>(
        &self,
        monitor_callback: F,
    ) -> Result<SimpleHttpResponse> {
        info!("Request URI: {}", self.uri);
        let res = self.client.get(self.uri.as_str()).send().await?;

        let data_len = res.content_length().ok_or(anyhow!(format!(
            "Unknown content length for remote uri {}",
            self.uri
        )))?;

        let mut bytes_retrieved: u64 = 0;
        let mut bytes_buffer: Vec<u8> = Vec::with_capacity(data_len as usize);
        let status = res.status();
        let mut stream = res.bytes_stream();

        while let Some(bytes) = stream.next().await {
            let chunk = bytes.or(Err(anyhow!("Error retrieving data chunk")))?;
            bytes_buffer.append(&mut chunk.to_vec().clone());
            let new = min(bytes_retrieved + (chunk.len() as u64), data_len);
            bytes_retrieved = new;

            monitor_callback(
                data_len,
                bytes_retrieved,
                bytes_retrieved as f32 / data_len as f32,
            );
        }

        Ok(SimpleHttpResponse {
            bytes: bytes_buffer,
            status: status.as_u16(),
        })
    }

    pub async fn into_bytes(&self) -> Result<SimpleHttpResponse> {
        self.fetch().await
    }

    pub async fn into_bytes_monitored<F: Fn(u64, u64, f32)>(
        &self,
        f: F,
    ) -> Result<SimpleHttpResponse> {
        self.fetch_monitored(f).await
    }

    pub async fn into_string(&self) -> Result<SimpleHttpResponseString> {
        let res = self.fetch().await?;
        Ok(SimpleHttpResponseString {
            text: String::from_utf8(res.bytes).expect("Failed to parse response to string"),
            status: res.status,
        })
    }

    pub async fn into_string_monitored<F: Fn(u64, u64, f32)>(
        &self,
        f: F,
    ) -> Result<SimpleHttpResponseString> {
        let res = self.fetch_monitored(f).await?;
        Ok(SimpleHttpResponseString {
            text: String::from_utf8(res.bytes).expect("Failed to parse response to string"),
            status: res.status,
        })
    }
}

pub async fn simple_fetch_bin(uri: &str) -> Result<Vec<u8>> {
    let resp = HttpFetcher::new(uri)?.fetch().await?;
    Ok(resp.bytes)
}

pub async fn simple_fetch_text(uri: &str) -> Result<String> {
    match String::from_utf8(simple_fetch_bin(uri).await?) {
        Ok(s) => Ok(s),
        Err(why) => Err(why.into()),
    }
}

pub async fn simple_fetch_bin_monitored<F: Fn(u64, u64, f32)>(uri: &str, f: F) -> Result<Vec<u8>> {
    let resp = HttpFetcher::new(uri)?.fetch_monitored(f).await?;
    Ok(resp.bytes)
}

pub async fn simple_fetch_text_monitored<F: Fn(u64, u64, f32)>(uri: &str, f: F) -> Result<String> {
    match String::from_utf8(simple_fetch_bin_monitored(uri, f).await?) {
        Ok(s) => Ok(s),
        Err(why) => Err(why.into()),
    }
}

#[test]
fn test_get_timeout_seconds() {
    env::remove_var("MRU_HTTP_TIMEOUT");

    assert_eq!(get_timeout_seconds().unwrap(), DEFAULT_TIMEOUT);

    env::set_var("MRU_HTTP_TIMEOUT", "FOO");
    assert_eq!(get_timeout_seconds().unwrap(), DEFAULT_TIMEOUT);

    env::set_var("MRU_HTTP_TIMEOUT", "-34");
    assert_eq!(get_timeout_seconds().unwrap(), DEFAULT_TIMEOUT);

    env::set_var("MRU_HTTP_TIMEOUT", "34.45");
    assert_eq!(get_timeout_seconds().unwrap(), DEFAULT_TIMEOUT);

    env::set_var("MRU_HTTP_TIMEOUT", "-34.45");
    assert_eq!(get_timeout_seconds().unwrap(), DEFAULT_TIMEOUT);

    env::set_var("MRU_HTTP_TIMEOUT", "120");
    assert_eq!(get_timeout_seconds().unwrap(), 120);

    env::set_var("MRU_HTTP_TIMEOUT", "120_u64");
    assert_eq!(get_timeout_seconds().unwrap(), DEFAULT_TIMEOUT);
}
