
use serde::{
    Deserialize, 
    Serialize
};



#[derive(Serialize, Deserialize, Debug)]
pub struct Cahvor {
    
    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub c: Option<Vec<f64>>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub a: Option<Vec<f64>>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub h: Option<Vec<f64>>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub v: Option<Vec<f64>>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub o: Option<Vec<f64>>,

    #[serde(with = "crate::jsonfetch::tuple_format")]
    pub r: Option<Vec<f64>>,
}