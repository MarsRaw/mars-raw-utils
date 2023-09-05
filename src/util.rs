use crate::constants;
use anyhow::{anyhow, Result};
use sciimg::path;
use sciimg::util as sciutil;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[macro_export]
macro_rules! f {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}

#[macro_export]
macro_rules! t {
    ($s:expr) => {
        format!("{:?}", $s)
    };
}

pub fn string_is_valid_f64(s: &str) -> bool {
    sciutil::string_is_valid_f64(s)
}

pub fn string_is_valid_f32(s: &str) -> bool {
    sciutil::string_is_valid_f32(s)
}

pub fn string_is_valid_i32(s: &str) -> bool {
    sciutil::string_is_valid_i32(s)
}

pub fn string_is_valid_u16(s: &str) -> bool {
    sciutil::string_is_valid_u16(s)
}

pub fn filename_char_at_pos(filename: &str, pos: usize) -> char {
    sciutil::filename_char_at_pos(filename, pos)
}

/// Returns the maximum value from a list of values
#[macro_export]
macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

/// Returns the minimum value from a list of values
#[macro_export]
macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}

////////////////////////////////////
/// Functions for supporting instrument lists
////////////////////////////////////

pub struct InstrumentMap {
    pub map: HashMap<&'static str, Vec<&'static str>>,
}

impl InstrumentMap {
    pub fn is_name_a_remote_instrument(&self, instrument: &str) -> bool {
        for rem_inst_list in self.map.values() {
            for s in rem_inst_list.iter() {
                if &instrument == s {
                    return true;
                }
            }
        }
        false
    }

    pub fn remote_instrument_names(&self) -> Vec<String> {
        self.map
            .values()
            .flatten()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
    }

    pub fn find_remote_instrument_names(&self, instrument: &str) -> Result<Vec<String>> {
        let mut inst_list: Vec<String> = Vec::new();
        if self.is_name_a_remote_instrument(instrument) {
            inst_list.push(String::from(instrument));
            return Ok(inst_list);
        }

        if self.map.contains_key(instrument) {
            let sublist = self.map.get(instrument).unwrap();
            inst_list.extend(sublist.iter().map(|&i| String::from(i)));
        }

        if !inst_list.is_empty() {
            Ok(inst_list)
        } else {
            Err(anyhow!(constants::status::UNSUPPORTED_INSTRUMENT))
        }
    }

    pub fn find_remote_instrument_names_fromlist(
        &self,
        instrument_inputs: &[String],
    ) -> Result<Vec<String>> {
        let mut inst_list: Vec<String> = Vec::new();

        for c in instrument_inputs.iter() {
            let found_list_res = self.find_remote_instrument_names(c);
            let res = match found_list_res {
                Err(_e) => return Err(anyhow!(constants::status::UNSUPPORTED_INSTRUMENT)),
                Ok(v) => v,
            };
            inst_list.extend(res);
        }

        if !inst_list.is_empty() {
            Ok(inst_list)
        } else {
            Err(anyhow!(constants::status::UNSUPPORTED_INSTRUMENT))
        }
    }

    pub fn print_instruments(&self) {
        for (key, rem_inst_list) in &self.map {
            println!("{}", key);
            for s in rem_inst_list.iter() {
                println!("  {}", s);
            }
        }
    }
}

pub fn stringvec(a: &str, b: &str) -> Vec<String> {
    vec![a.to_owned(), b.to_owned()]
}

pub fn stringvec_b(a: &str, b: String) -> Vec<String> {
    vec![a.to_owned(), b]
}

#[deprecated]
pub fn image_exists_on_filesystem(image_url: &str) -> bool {
    let bn = path::basename(image_url);
    path::file_exists(bn.as_str())
}

pub fn save_image_json<T: Serialize>(
    image_url: &str,
    item: &T,
    output_path: Option<&str>,
) -> Result<()> {
    let item_str = serde_json::to_string_pretty(item)?;

    let write_to = match output_path {
        Some(p) => {
            let bn = path::basename(image_url);
            format!("{}/{}", p, bn)
        }
        None => String::from(image_url),
    };

    save_image_json_from_string(&write_to, &item_str)
}

pub fn save_image_json_from_string(image_path: &str, item: &String) -> Result<()> {
    let out_file = image_path
        .replace(".jpg", "-metadata.json")
        .replace(".JPG", "-metadata.json")
        .replace(".png", "-metadata.json")
        .replace(".PNG", "-metadata.json");

    let path = Path::new(out_file.as_str());
    info!("Writing metadata file to {}", path.to_str().unwrap());

    let mut file = File::create(path)?;
    file.write_all(item.as_bytes())?;
    Ok(())
}

pub fn append_file_name(input_file: &str, append: &str) -> String {
    let append_with_ext = format!("-{}.png", append);
    replace_image_extension(input_file, append_with_ext.as_str())
}

pub fn replace_extension<S>(from_file: &S, new_extension: &str) -> Result<String>
where
    S: AsRef<Path> + ?Sized + AsRef<OsStr>,
{
    if let Some(new_filename) = Path::new(from_file).with_extension(new_extension).to_str() {
        Ok(new_filename.to_string())
    } else {
        Err(anyhow!("Unable to replace filename"))
    }
}

pub fn replace_image_extension(input_file: &str, append: &str) -> String {
    input_file
        .replace(".png", append)
        .replace(".PNG", append)
        .replace(".jpg", append)
        .replace(".JPG", append)
        .replace(".tif", append)
        .replace(".TIF", append)
        .replace(".dng", append)
        .replace(".DNG", append)
}
