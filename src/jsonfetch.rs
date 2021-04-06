
use reqwest::{StatusCode, blocking};

use json;
use crate::{constants, print, vprintln, error};
use std::string::String;

pub struct JsonFetcher {
    uri : String,
    numparams: u32
}

impl JsonFetcher {

    pub fn new(uri:&str) -> JsonFetcher {
        JsonFetcher{
            uri:String::from(uri),
            numparams:0
        }
    }

    pub fn param(&mut self, key:&str, value:&str) {
        let q = if self.numparams == 0 { "?" } else { "&" };
        self.uri = format!("{}{}{}={}", self.uri, q, key, value);
        self.numparams += 1;
    }


    pub fn fetch(&self) -> error::Result<json::JsonValue> {
        vprintln!("Request URI: {}", self.uri);
        let res = blocking::get(self.uri.as_str()).unwrap();

        // check response code, etc... Handle errors better...
        assert_eq!(res.status(), StatusCode::OK);

        let json_text = res.text().unwrap();
        //vprintln!("{}", json_text);
        let parsed_json = json::parse(&json_text).unwrap();
        
        Ok(parsed_json)
    }
}