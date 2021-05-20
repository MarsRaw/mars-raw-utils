
use crate::{
    vector::Vector
};

use serde::{
    Deserialize, 
    Serialize
};



#[derive(Serialize, Deserialize, Debug)]
pub struct Cahvor {
    
    #[serde(with = "crate::jsonfetch::vector_format")]
    pub c: Option<Vector>,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub a: Option<Vector>,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub h: Option<Vector>,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub v: Option<Vector>,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub o: Option<Vector>,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub r: Option<Vector>
}