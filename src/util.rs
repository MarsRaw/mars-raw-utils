

use std::str::FromStr;

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

