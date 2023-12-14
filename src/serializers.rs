use crate::{constants, util::string_is_valid_f64};
use anyhow::{anyhow, Result};
use string_builder::Builder;

fn vec_to_str(v: &[f64]) -> String {
    let mut b = Builder::default();

    for item in v {
        b.append(format!("{},", item));
    }

    let mut s = b.string().unwrap();
    if !s.is_empty() {
        s.remove(s.len() - 1);
    }

    format!("({})", s)
}

fn str_to_vec(s: &str) -> Result<Vec<f64>> {
    let mut tuple_vec: Vec<f64> = Vec::new();
    let mut s0 = String::from(s);
    s0.remove(0);
    s0.remove(s0.len() - 1);
    let split = s0.split(',');
    for n in split {
        let n_t = n.trim();
        if string_is_valid_f64(n_t) {
            tuple_vec.push(n_t.parse::<f64>().unwrap());
        } else {
            error!("Encoutered invalid float value string: {}", n_t);
            return Err(anyhow!(constants::status::INVALID_FLOAT_VALUE));
        }
    }
    Ok(tuple_vec)
}

//////////////////////////////////////////////////
// f64
//////////////////////////////////////////////////

pub mod as_f64 {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(num.to_string().as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(0.0)
        } else {
            s.replace(',', "")
                .parse::<f64>()
                .map_err(serde::de::Error::custom)
        }
    }
}

//////////////////////////////////////////////////
// f64 (Option)
//////////////////////////////////////////////////

pub mod as_f64_opt {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(v) = num {
            serializer.serialize_str(v.to_string().as_str())
        } else {
            serializer.serialize_str("")
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() || s == "--" {
            Ok(None)
        } else {
            match s
                .replace(',', "")
                .parse::<f64>()
                .map_err(serde::de::Error::custom)
            {
                Ok(v) => Ok(Some(v)),
                Err(why) => Err(why),
            }
        }
    }
}

//////////////////////////////////////////////////
// f32
//////////////////////////////////////////////////

pub mod as_f32 {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &f32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(num.to_string().as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() || s == "--" {
            Ok(0.0)
        } else {
            s.replace(',', "")
                .parse::<f32>()
                .map_err(serde::de::Error::custom)
        }
    }
}

//////////////////////////////////////////////////
// f32 (Option)
//////////////////////////////////////////////////

pub mod as_f32_opt {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &Option<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(v) = num {
            serializer.serialize_str(v.to_string().as_str())
        } else {
            serializer.serialize_str("")
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() || s == "--" {
            Ok(None)
        } else {
            match s
                .replace(',', "")
                .parse::<f32>()
                .map_err(serde::de::Error::custom)
            {
                Ok(v) => Ok(Some(v)),
                Err(why) => Err(why),
            }
        }
    }
}

//////////////////////////////////////////////////
// i64
//////////////////////////////////////////////////

pub mod as_i64 {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(num.to_string().as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(0)
        } else {
            s.replace(',', "")
                .parse::<i64>()
                .map_err(serde::de::Error::custom)
        }
    }
}

//////////////////////////////////////////////////
// i64 (Option)
//////////////////////////////////////////////////

pub mod as_i64_opt {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(v) = num {
            serializer.serialize_str(v.to_string().as_str())
        } else {
            serializer.serialize_str("")
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() || s == "--" {
            Ok(None)
        } else {
            match s
                .replace(',', "")
                .parse::<i64>()
                .map_err(serde::de::Error::custom)
            {
                Ok(v) => Ok(Some(v)),
                Err(why) => Err(why),
            }
        }
    }
}

//////////////////////////////////////////////////
// i32
//////////////////////////////////////////////////

pub mod as_i32 {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(num.to_string().as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(0)
        } else {
            s.replace(',', "")
                .parse::<i32>()
                .map_err(serde::de::Error::custom)
        }
    }
}

//////////////////////////////////////////////////
// i32 (Option)
//////////////////////////////////////////////////

pub mod as_i32_opt {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &Option<i32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(v) = num {
            serializer.serialize_str(v.to_string().as_str())
        } else {
            serializer.serialize_str("")
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() || s == "--" {
            Ok(None)
        } else {
            match s
                .replace(',', "")
                .parse::<i32>()
                .map_err(serde::de::Error::custom)
            {
                Ok(v) => Ok(Some(v)),
                Err(why) => Err(why),
            }
        }
    }
}

//////////////////////////////////////////////////
// Day of Year, e.g. 2023-335T13:53:20.000
//////////////////////////////////////////////////

// https://serde.rs/custom-date-format.html
pub mod as_df_doy {
    use chrono::{DateTime, FixedOffset, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%jT%H:%M:%S%.3f %z";

    pub fn serialize<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(Utc::now().fixed_offset())
        } else {
            DateTime::parse_from_str(&format!("{} +0000", s), FORMAT)
                .map_err(serde::de::Error::custom)
        }
    }
}

//////////////////////////////////////////////////
// Simple Date Format, e.g. 2023-12-03
//////////////////////////////////////////////////

// https://serde.rs/custom-date-format.html
pub mod as_df_date {
    use chrono::{DateTime, FixedOffset, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3f %z";

    pub fn serialize<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(Utc::now().fixed_offset())
        } else {
            DateTime::parse_from_str(&format!("{}T00:00:00.000 +0000", s), FORMAT)
                .map_err(serde::de::Error::custom)
        }
    }
}

//////////////////////////////////////////////////
// CAHVORE
//////////////////////////////////////////////////

pub mod as_cahvore {

    use super::str_to_vec;
    use crate::util::string_is_valid_f64;
    use sciimg::prelude::*;
    use sciimg::vector::Vector;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(model_opt: &CameraModel, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !model_opt.is_valid() {
            serializer.serialize_unit()
        } else {
            serializer.serialize_str(&model_opt.serialize())
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<CameraModel, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: Result<&str, D::Error> = Deserialize::deserialize(deserializer);
        match r {
            Err(_) => Ok(CameraModel::default()),
            Ok(s) => {
                let s0 = String::from(s);

                let split = s0.split(';');
                let mut parts: Vec<Vec<f64>> = Vec::new();

                for n in split {
                    match n.find('(') {
                        None => {
                            if string_is_valid_f64(n) {
                                parts.push(vec![n.parse::<f64>().unwrap()]);
                            }
                        }
                        Some(_i) => {
                            parts.push(str_to_vec(n).unwrap());
                        }
                    }
                }

                match parts.len() {
                    4 => {
                        // CAHV
                        Ok(CameraModel::new(Box::new(Cahv {
                            c: if !parts.is_empty() {
                                Vector::from_vec(&parts[0]).unwrap()
                            } else {
                                Vector::default()
                            },
                            a: if parts.len() >= 2 {
                                Vector::from_vec(&parts[1]).unwrap()
                            } else {
                                Vector::default()
                            },
                            h: if parts.len() >= 3 {
                                Vector::from_vec(&parts[2]).unwrap()
                            } else {
                                Vector::default()
                            },
                            v: if parts.len() >= 4 {
                                Vector::from_vec(&parts[3]).unwrap()
                            } else {
                                Vector::default()
                            },
                        })))
                    }
                    6 => {
                        // CAHVOR
                        Ok(CameraModel::new(Box::new(Cahvor {
                            c: if !parts.is_empty() {
                                Vector::from_vec(&parts[0]).unwrap()
                            } else {
                                Vector::default()
                            },
                            a: if parts.len() >= 2 {
                                Vector::from_vec(&parts[1]).unwrap()
                            } else {
                                Vector::default()
                            },
                            h: if parts.len() >= 3 {
                                Vector::from_vec(&parts[2]).unwrap()
                            } else {
                                Vector::default()
                            },
                            v: if parts.len() >= 4 {
                                Vector::from_vec(&parts[3]).unwrap()
                            } else {
                                Vector::default()
                            },
                            o: if parts.len() >= 5 {
                                Vector::from_vec(&parts[4]).unwrap()
                            } else {
                                Vector::default()
                            },
                            r: if parts.len() >= 6 {
                                Vector::from_vec(&parts[5]).unwrap()
                            } else {
                                Vector::default()
                            },
                        })))
                    }
                    9 => {
                        // CAHVORE
                        Ok(CameraModel::new(Box::new(Cahvore {
                            c: if !parts.is_empty() {
                                Vector::from_vec(&parts[0]).unwrap()
                            } else {
                                Vector::default()
                            },
                            a: if parts.len() >= 2 {
                                Vector::from_vec(&parts[1]).unwrap()
                            } else {
                                Vector::default()
                            },
                            h: if parts.len() >= 3 {
                                Vector::from_vec(&parts[2]).unwrap()
                            } else {
                                Vector::default()
                            },
                            v: if parts.len() >= 4 {
                                Vector::from_vec(&parts[3]).unwrap()
                            } else {
                                Vector::default()
                            },
                            o: if parts.len() >= 5 {
                                Vector::from_vec(&parts[4]).unwrap()
                            } else {
                                Vector::default()
                            },
                            r: if parts.len() >= 6 {
                                Vector::from_vec(&parts[5]).unwrap()
                            } else {
                                Vector::default()
                            },
                            e: if parts.len() >= 7 {
                                Vector::from_vec(&parts[6]).unwrap()
                            } else {
                                Vector::default()
                            },
                            linearity: if parts.len() >= 8 {
                                parts[7][0]
                            } else {
                                LINEARITY_PERSPECTIVE
                            },
                            pupil_type: PupilType::General,
                        })))
                    }
                    _ => Ok(CameraModel::default()),
                }
            }
        }
    }
}

/////////////////////////////////////////////////
// Raw image metadata tuples
/////////////////////////////////////////////////
pub mod as_tuple {

    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::{str_to_vec, vec_to_str};

    pub fn serialize<S>(tuple_vec_opt: &Option<Vec<f64>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match tuple_vec_opt {
            None => serializer.serialize_unit(),
            Some(v) => {
                let s = vec_to_str(v);
                serializer.serialize_str(s.as_ref())
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<f64>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: Result<&str, D::Error> = Deserialize::deserialize(deserializer);
        match r {
            Err(_) => Ok(None),
            Ok(s) => match s {
                "UNK" => Ok(None),
                _ => {
                    let tuple_vec = str_to_vec(s).unwrap();
                    Ok(Some(tuple_vec))
                }
            },
        }
    }
}

////////////////////////////////////////////
// Defaults
////////////////////////////////////////////

pub fn default_vec_f64_none() -> Option<Vec<f64>> {
    None
}

pub fn default_false() -> bool {
    false
}

pub fn default_blank() -> String {
    "".to_string()
}

pub fn default_vec<T>() -> Vec<T> {
    vec![]
}
