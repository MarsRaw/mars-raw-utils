
use crate::{
    vector::Vector
};

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    Cahvor,
    Cahv
}
impl Default for Mode {
    fn default() -> Self {
        Mode::Cahv
    }
}


pub struct Point {
    pub i: f64,
    pub j: f64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cahvor {
    
    
    #[serde(skip_deserializing)]
    #[serde(default)]
    pub mode: Mode,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub c: Vector,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub a: Vector,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub h: Vector,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub v: Vector,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub o: Vector,

    #[serde(with = "crate::jsonfetch::vector_format")]
    pub r: Vector
}

impl Cahvor {

    pub fn hc(&self) -> f64 {
        self.a.dot_product(&self.h)
    }

    pub fn vc(&self) -> f64 {
        self.a.dot_product(&self.v)
    }

    pub fn hs(&self) -> f64 {
        let cp = self.a.cross_product(&self.h);
        cp.len()
    }

    pub fn vs(&self) -> f64 {
        let cp = self.a.cross_product(&self.v);
        cp.len()
    }

    pub fn project_object_to_image_point(&self, p:&Vector) -> Point {
        let i = p.subtract(&self.c).dot_product(&self.h) / p.subtract(&self.c).dot_product(&self.a);
        let j = p.subtract(&self.c).dot_product(&self.v) / p.subtract(&self.c).dot_product(&self.a);

        Point{
            i:i,
            j:j
        }
    } 
}