
use crate::{
    path,
    constants,
    error
};

use std::str::FromStr;
use std::collections::HashMap;

pub fn string_is_valid_num<T:FromStr>(s:&str) -> bool {
    let num = s.parse::<T>();
    match num {
        Ok(_) => true,
        Err(_) => false
    }
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
/// 
pub fn is_name_a_remote_instrument(instrument:&str, instruments:&HashMap<&str, Vec<&str>>) -> bool {
    for (key, rem_inst_list) in instruments {
        for s in rem_inst_list.iter() {
            if &instrument == s {
                return true;
            }
        }
    }
    false
}


pub fn find_remote_instrument_names(instrument:&str, instruments:&HashMap<&str, Vec<&str>>) -> error::Result<Vec<String>> {
    let mut inst_list : Vec<String> = Vec::new();
    if is_name_a_remote_instrument(instrument, instruments) {
        inst_list.push(String::from(instrument));
        return Ok(inst_list);
    }

    if instruments.contains_key(instrument) {
        let sublist = instruments.get(instrument).unwrap();
        inst_list.extend(sublist.iter().map(|&i| String::from(i)));
        
    }

    if inst_list.len() > 0 {
        return Ok(inst_list);
    } else {
        return Err(constants::status::UNSUPPORTED_INSTRUMENT);
    }
}


pub fn find_remote_instrument_names_fromlist(instrument_inputs:&Vec<&str>, instruments:&HashMap<&str, Vec<&str>>) -> error::Result<Vec<String>> {
    let mut inst_list : Vec<String> = Vec::new();

    for c in instrument_inputs.iter() {
        let found_list_res = find_remote_instrument_names(c, instruments);
        let res = match found_list_res {
            Err(e) => return Err(constants::status::UNSUPPORTED_INSTRUMENT),
            Ok(v) => v,
        };
        inst_list.extend(res);
    }

    if inst_list.len() > 0 {
        return Ok(inst_list);
    } else {
        return Err(constants::status::UNSUPPORTED_INSTRUMENT);
    }
}
