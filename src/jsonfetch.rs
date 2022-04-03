
use serde_json::{
    Value
};
use crate::{
    constants,  
    httpfetch::HttpFetcher,
    util::string_is_valid_f64
};

use sciimg::error;

use string_builder::Builder;

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
            Err(_e) => Err(constants::status::REMOTE_SERVER_ERROR),
            Ok(v) => Ok(serde_json::from_str(&v).unwrap())
        }
    }



    pub fn fetch_str(&self) -> error::Result<String> {
        let json_text = self.fetcher.fetch_text();

        match json_text {
            Err(_e) => Err(constants::status::REMOTE_SERVER_ERROR),
            Ok(v) => Ok(v)
        }
    }
}


fn vec_to_str(v:&[f64]) -> String {
    let mut b = Builder::default();

    for item in v {
        b.append(format!("{},", item));
    }

    let mut s = b.string().unwrap();
    if !s.is_empty() {
        s.remove(s.len()-1);
    }
    

    format!("({})", s)
}

fn str_to_vec(s:&str) -> error::Result<Vec<f64>> {
    let mut tuple_vec:Vec<f64> = Vec::new();
    let mut s0 = String::from(s);
    s0.remove(0);s0.remove(s0.len()-1);
    let split = s0.split(',');
    for n in split {
        let n_t = n.trim();
        if string_is_valid_f64(n_t) {
            tuple_vec.push(n_t.parse::<f64>().unwrap());
        } else {
            eprintln!("Encoutered invalid float value string: {}", n_t);
            return Err(constants::status::INVALID_FLOAT_VALUE);
        }
        
    }
    Ok(tuple_vec)
}

pub mod cahvor_format {

    use serde::{
        self,
        Deserialize,
        Deserializer,
        Serializer
    };

    use sciimg::cahvor;

    use crate::jsonfetch::{
        str_to_vec,
        vec_to_str
    };

    use sciimg::vector::Vector;

    pub fn serialize<S>(
        cahvor_opt: &Option<cahvor::Cahvor>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match cahvor_opt {
            None => {
                serializer.serialize_unit()
            },
            Some(cahvor) => {
                let c = vec_to_str(&cahvor.c.to_vec());
                let a = vec_to_str(&cahvor.a.to_vec());
                let h = vec_to_str(&cahvor.h.to_vec());
                let v = vec_to_str(&cahvor.v.to_vec());
                let o = vec_to_str(&cahvor.o.to_vec());
                let r = vec_to_str(&cahvor.r.to_vec());
                match cahvor.mode {
                    cahvor::Mode::Cahvor => {
                        let s = format!("{};{};{};{};{};{}", c, a, h, v, o, r);
                        serializer.serialize_str(&s)
                    },
                    cahvor::Mode::Cahv => {
                        let s = format!("{};{};{};{}", c, a, h, v);
                        serializer.serialize_str(&s)
                    }
                }
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<cahvor::Cahvor>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r :Result<&str, D::Error> = Deserialize::deserialize(deserializer);
        match r {
            Err(_) => Ok(None),
            Ok(s) => {

                let s0 = String::from(s);
                
                let split = s0.split(';');
                let mut parts:Vec<Vec<f64>> = Vec::new();

                for n in split {
                    match n.find('(') {
                        None => (),
                        Some(_i) => {
                            let v:Vec<f64> = str_to_vec(&n).unwrap();
                            parts.push(v);
                        }
                    }
                }

                Ok(Some(cahvor::Cahvor{
                    mode: if parts.len() == 4 { cahvor::Mode::Cahv } else { cahvor::Mode::Cahvor },
                    c: if parts.len() >= 1 { Vector::from_vec(&parts[0]).unwrap() } else { Vector::default() },
                    a: if parts.len() >= 2 { Vector::from_vec(&parts[1]).unwrap() } else { Vector::default() },
                    h: if parts.len() >= 3 { Vector::from_vec(&parts[2]).unwrap() } else { Vector::default() },
                    v: if parts.len() >= 4 { Vector::from_vec(&parts[3]).unwrap() } else { Vector::default() },
                    o: if parts.len() >= 5 { Vector::from_vec(&parts[4]).unwrap() } else { Vector::default() },
                    r: if parts.len() >= 6 { Vector::from_vec(&parts[5]).unwrap() } else { Vector::default() }
                }))
            }
        }
    }
}

pub mod tuple_format {

    use serde::{
        self,
        Deserialize,
        Deserializer,
        Serializer
    };

    use crate::jsonfetch::{
        str_to_vec,
        vec_to_str
    };

    pub fn serialize<S>(
        tuple_vec_opt: &Option<Vec<f64>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match tuple_vec_opt {
            None => {
                serializer.serialize_unit()
            },
            Some(v) => {
                let s = vec_to_str(&v);
                serializer.serialize_str(s.as_ref())
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<f64>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r :Result<&str, D::Error> = Deserialize::deserialize(deserializer);
        match r {
            Err(_) => Ok(None),
            Ok(s) => {
                match s {
                    "UNK" => Ok(None),
                    _ => {
                        let tuple_vec = str_to_vec(s).unwrap();
                        Ok(Some(tuple_vec))
                    }
                }
            }
        }
    }
}
