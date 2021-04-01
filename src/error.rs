
use std::result;

use crate::constants;

pub type Result<T> = result::Result<T, &'static str>;


#[macro_export]
macro_rules! ok {
    () => (Ok(constants::status::OK));
}

#[macro_export]
macro_rules! not_implemented {
    () => (Err(constants::status::NOT_IMPLEMENTED));
}