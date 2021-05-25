
use crate::{
    path,
    constants,
    error,
    ok,
    vprintln,
    httpfetch
};

use std::str::FromStr;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::Write;

use serde::{
    Serialize
};

pub fn string_is_valid_num<T:FromStr>(s:&str) -> bool {
    let num = s.parse::<T>();
    num.is_ok()
}

pub fn string_is_valid_f64(s:&str) -> bool {
    string_is_valid_num::<f64>(s)
}


pub fn string_is_valid_f32(s:&str) -> bool {
    string_is_valid_num::<f32>(s)
}

pub fn string_is_valid_i32(s:&str) -> bool {
    string_is_valid_num::<i32>(s)
}

pub fn filename_char_at_pos(filename:&str, pos:usize) -> char {
    let bn = path::basename(&filename);
    bn.chars().nth(pos).unwrap()
}



////////////////////////////////////
/// Functions for supporting instrument lists
////////////////////////////////////


pub struct InstrumentMap {
    pub map: HashMap<&'static str, Vec<&'static str>>
}

impl InstrumentMap {
    pub fn is_name_a_remote_instrument(&self, instrument:&str) -> bool {
        for rem_inst_list in self.map.values() {
            for s in rem_inst_list.iter() {
                if &instrument == s {
                    return true;
                }
            }
        }
        false
    }

    pub fn find_remote_instrument_names(&self, instrument:&str) -> error::Result<Vec<String>> {
        let mut inst_list : Vec<String> = Vec::new();
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

    pub fn find_remote_instrument_names_fromlist(&self, instrument_inputs:&[&str]) -> error::Result<Vec<String>> {
        let mut inst_list : Vec<String> = Vec::new();
    
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











pub fn stringvec(a:&str, b:&str) -> Vec<String> {
    vec![a.to_owned(), b.to_owned()]
}

pub fn stringvec_b(a:&str, b:String) -> Vec<String> {
    vec![a.to_owned(), b]
}


pub fn image_exists_on_filesystem(image_url:&str) -> bool {
    //let image_url = &image["image_files"]["full_res"].as_str().unwrap();
    let bn = path::basename(image_url);
    path::file_exists(bn.as_str())
}

pub fn fetch_image(image_url:&str, only_new:bool) -> error::Result<&'static str> {
    //let image_url = &image["url"].as_str().unwrap();
    let bn = path::basename(image_url);

    if image_exists_on_filesystem(&image_url) && only_new {
        vprintln!("Output file {} exists, skipping", bn);
        return ok!();
    }

    let image_data = match httpfetch::simple_fetch_bin(image_url) {
        Ok(i) => i,
        Err(e) => return Err(e)
    };
    
    let path = Path::new(bn.as_str());

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}", why),
        Ok(file) => file,
    };

    match file.write_all(&image_data[..]) {
        Ok(_) => ok!(),
        Err(_e) => Err("Error writing image to filesystem")
    }
}

pub fn save_image_json<T:Serialize>(image_url:&str, item:&T, only_new:bool) -> error::Result<&'static str> {
    let item_str = serde_json::to_string_pretty(item).unwrap();
    save_image_json_from_string(&image_url, &item_str, only_new)
}

pub fn save_image_json_from_string(image_url:&str, item:&String, only_new:bool) -> error::Result<&'static str> {
    let bn = path::basename(image_url);

    let out_file = bn.replace(".jpg", "-metadata.json").replace(".JPG", "-metadata.json")
                             .replace(".png", "-metadata.json").replace(".PNG", "-metadata.json");

    if path::file_exists(out_file.as_str()) && only_new {
        vprintln!("Output file {} exists, skipping", bn);
        return ok!();
    }

    let path = Path::new(out_file.as_str());

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}", why),
        Ok(file) => file,
    };

    match file.write_all(item.as_bytes()) {
        Ok(_) => ok!(),
        Err(_e) => Err("Error writing metadata to filesystem")
    }
}


pub fn append_file_name(input_file:&str, append:&str) -> String {

    let append_with_ext = format!("-{}.png", append);
    let out_file = input_file.replace(".png", append_with_ext.as_str())
                             .replace(".PNG", append_with_ext.as_str())
                             .replace(".jpg", append_with_ext.as_str())
                             .replace(".JPG", append_with_ext.as_str())
                             .replace(".tif", append_with_ext.as_str())
                             .replace(".TIF", append_with_ext.as_str());
    String::from(out_file)
}