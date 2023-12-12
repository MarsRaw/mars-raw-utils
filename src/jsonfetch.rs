use crate::httpfetch::HttpFetcher;
use anyhow::Result;
use serde_json::Value;

pub struct JsonFetcher {
    fetcher: HttpFetcher,
}

impl JsonFetcher {
    pub fn new(uri: &str) -> Result<JsonFetcher> {
        Ok(JsonFetcher {
            fetcher: match HttpFetcher::new(uri) {
                Ok(it) => it,
                Err(err) => return Err(err),
            },
        })
    }

    pub fn param(&mut self, key: &str, value: &str) {
        _ = self.fetcher.param(key, value);
    }

    pub async fn fetch(&self) -> Result<Value> {
        let json_text = self.fetcher.into_string().await?; //as_string() is also a common name for this.
        Ok(serde_json::from_str(&json_text.text)?)
    }

    pub async fn fetch_str(&self) -> Result<String> {
        Ok(self.fetcher.into_string().await?.text)
    }
}
