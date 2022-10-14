use crate::{constants, httpfetch, path, vprintln};

use sciimg::{error, ok};

use sciimg::util as sciutil;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

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

    pub fn find_remote_instrument_names(&self, instrument: &str) -> error::Result<Vec<String>> {
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
            Err(constants::status::UNSUPPORTED_INSTRUMENT)
        }
    }

    pub fn find_remote_instrument_names_fromlist(
        &self,
        instrument_inputs: &Vec<String>,
    ) -> error::Result<Vec<String>> {
        let mut inst_list: Vec<String> = Vec::new();

        for c in instrument_inputs.iter() {
            let found_list_res = self.find_remote_instrument_names(c);
            let res = match found_list_res {
                Err(_e) => return Err(constants::status::UNSUPPORTED_INSTRUMENT),
                Ok(v) => v,
            };
            inst_list.extend(res);
        }

        if !inst_list.is_empty() {
            Ok(inst_list)
        } else {
            Err(constants::status::UNSUPPORTED_INSTRUMENT)
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

pub fn fetch_image(
    image_url: &str,
    only_new: bool,
    output_path: Option<&str>,
) -> error::Result<&'static str> {
    let write_to = match output_path {
        Some(p) => {
            let bn = path::basename(image_url);
            format!("{}/{}", p, bn)
        }
        None => String::from(image_url),
    };

    if path::file_exists(&write_to) && only_new {
        vprintln!("Output file {} exists, skipping", write_to);
        return ok!();
    } else {
        let image_data = match httpfetch::simple_fetch_bin(image_url) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };

        let path = Path::new(write_to.as_str());
        vprintln!("Writing image data to {}", write_to);

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}", why),
            Ok(file) => file,
        };

        match file.write_all(&image_data[..]) {
            Ok(_) => ok!(),
            Err(_e) => Err("Error writing image to filesystem"),
        }
    }
}

pub fn save_image_json<T: Serialize>(
    image_url: &str,
    item: &T,
    only_new: bool,
    output_path: Option<&str>,
) -> error::Result<&'static str> {
    let item_str = serde_json::to_string_pretty(item).unwrap();

    let write_to = match output_path {
        Some(p) => {
            let bn = path::basename(image_url);
            format!("{}/{}", p, bn)
        }
        None => String::from(image_url),
    };

    save_image_json_from_string(&write_to, &item_str, only_new)
}

pub fn save_image_json_from_string(
    image_path: &str,
    item: &String,
    only_new: bool,
) -> error::Result<&'static str> {
    let out_file = image_path
        .replace(".jpg", "-metadata.json")
        .replace(".JPG", "-metadata.json")
        .replace(".png", "-metadata.json")
        .replace(".PNG", "-metadata.json");

    if path::file_exists(out_file.as_str()) && only_new {
        vprintln!("Output file {} exists, skipping", image_path);
        return ok!();
    }

    let path = Path::new(out_file.as_str());

    vprintln!("Writing metadata file to {}", path.to_str().unwrap());

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}", why),
        Ok(file) => file,
    };

    match file.write_all(item.as_bytes()) {
        Ok(_) => ok!(),
        Err(_e) => Err("Error writing metadata to filesystem"),
    }
}

pub fn append_file_name(input_file: &str, append: &str) -> String {
    let append_with_ext = format!("-{}.png", append);
    replace_image_extension(input_file, append_with_ext.as_str())
    // let append_with_ext = format!("-{}.png", append);
    // let out_file = input_file.replace(".png", append_with_ext.as_str())
    //                          .replace(".PNG", append_with_ext.as_str())
    //                          .replace(".jpg", append_with_ext.as_str())
    //                          .replace(".JPG", append_with_ext.as_str())
    //                          .replace(".tif", append_with_ext.as_str())
    //                          .replace(".TIF", append_with_ext.as_str());
    // String::from(out_file)
}

pub fn replace_image_extension(input_file: &str, append: &str) -> String {
    let out_file = input_file
        .replace(".png", append)
        .replace(".PNG", append)
        .replace(".jpg", append)
        .replace(".JPG", append)
        .replace(".tif", append)
        .replace(".TIF", append);
    String::from(out_file)
}
