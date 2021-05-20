
use serde_json::{
    Value
};
use crate::{
    constants, 
    error, 
    httpfetch::HttpFetcher
};

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


fn vec_to_str(v:&Vec<f64>) -> String {
    let mut b = Builder::default();

    for item in v {
        b.append(format!("{},", item));
    }

    let mut s = b.string().unwrap();
    if s.len() > 0 {
        s.remove(s.len()-1);
    }
    

    format!("({})", s)
}

fn str_to_vec(s:&str) -> error::Result<Vec<f64>> {
    let mut tuple_vec:Vec<f64> = Vec::new();
    let mut s0 = String::from(s);
    s0.remove(0);s0.remove(s0.len()-1);
    let split = s0.split(",");
    for n in split {
        tuple_vec.push(n.parse::<f64>().unwrap());
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

    use crate::cahvor::Cahvor;

    use crate::jsonfetch::{
        str_to_vec,
        vec_to_str
    };

    use crate::vector::Vector;

    pub fn serialize<S>(
        cahvor_opt: &Option<Cahvor>,
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

                // Find a more reasonable way to pass the damn vec to vec_to_str...
                let c = vec_to_str(&cahvor.c.to_owned().unwrap().to_vec());
                let a = vec_to_str(&cahvor.a.to_owned().unwrap().to_vec());
                let h = vec_to_str(&cahvor.h.to_owned().unwrap().to_vec());
                let v = vec_to_str(&cahvor.v.to_owned().unwrap().to_vec());

                match cahvor.o {
                    Some(_) => {
                        let o = vec_to_str(&cahvor.o.to_owned().unwrap().to_vec());
                        let r = vec_to_str(&cahvor.r.to_owned().unwrap().to_vec());
                        
                        let s = format!("{};{};{};{};{};{}", c, a, h, v, o, r);
        
                        serializer.serialize_str(&s)
                    },
                    None => {
                        let s = format!("{};{};{};{}", c, a, h, v);
                        serializer.serialize_str(&s)
                    }
                }
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Cahvor>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r :Result<&str, D::Error> = Deserialize::deserialize(deserializer);
        match r {
            Err(_) => Ok(None),
            Ok(s) => {

                let s0 = String::from(s);
                
                let split = s0.split(";");
                let mut parts:Vec<Vec<f64>> = Vec::new();

                for n in split {
                    match n.find("(") {
                        None => (),
                        Some(_i) => {
                            let v:Vec<f64> = str_to_vec(&n).unwrap();
                            parts.push(v);
                        }
                    }
                }

                Ok(Some(Cahvor{
                    c: Some(Vector::from_vec(&parts[0]).unwrap()),
                    a: Some(Vector::from_vec(&parts[1]).unwrap()),
                    h: Some(Vector::from_vec(&parts[2]).unwrap()),
                    v: Some(Vector::from_vec(&parts[3]).unwrap()),
                    o: if parts.len() >= 5 { Some(Vector::from_vec(&parts[4]).unwrap()) } else { None },
                    r: if parts.len() >= 6 { Some(Vector::from_vec(&parts[5]).unwrap()) } else { None }
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



pub mod vector_format {
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

    use crate::vector::Vector;

    pub fn serialize<S>(
        vector_opt: &Option<Vector>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match vector_opt {
            None => {
                serializer.serialize_unit()
            },
            Some(v) => {
                let s = vec_to_str(&v.to_vec());
                serializer.serialize_str(s.as_ref())
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vector>, D::Error>
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
                        let vec = Vector::from_vec(&tuple_vec).unwrap();
                        Ok(Some(vec))
                    }
                }
            }
        }
    }

}



// fn from_triplet<'de, D>(deserializer: D) -> Result<Option<Triplet>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s: &str = Deserialize::deserialize(deserializer)?;
//     match sscanf::scanf!(s, "{{{},{},{}}}", f64, f64, f64) {
//         None => Ok(None),
//         Some(parsed) => {
//             Ok(Some(Triplet{x:parsed.0, y:parsed.1, z:parsed.2}))
//         }
//     }
// }