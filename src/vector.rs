
use crate::{
    stats::degrees,
    stats::radians,
    constants,
    error
};

#[derive(Debug, Clone)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Axis {
    XAxis,
    YAxis,
    ZAxis
}

impl Vector {

    pub fn default() -> Vector {
        Vector{x:0.0, y:0.0, z:0.0}
    }

    pub fn x_axis_vector() -> Vector {
        Vector{x:1.0, y:0.0, z:0.0}
    }

    pub fn y_axis_vector() -> Vector {
        Vector{x:0.0, y:1.0, z:0.0}
    }

    pub fn z_axis_vector() -> Vector {
        Vector{x:0.0, y:0.0, z:1.0}
    }

    pub fn new(x:f64, y:f64, z:f64) -> Vector {
        Vector{x, y, z}
    }

    pub fn from_vec(v:&[f64]) -> error::Result<Vector> {
        if v.len() != 3 {
            Err(constants::status::ARRAY_SIZE_MISMATCH)
        } else {
            Ok(Vector{
                x:v[0],
                y:v[1],
                z:v[2]
            })
        }
    }

    pub fn to_vec(&self) -> Vec<f64> {
        vec![self.x, self.y, self.z]
    }

    pub fn copy_to(&self, other:&mut Vector) {
        other.x = self.x;
        other.y = self.y;
        other.z = self.z;
    }

    pub fn copy_from(&mut self, other:&Vector) {
        self.x = other.x;
        self.y = other.y;
        self.z = other.z;
    }

    pub fn len(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn scale(&mut self, scalar:f64) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }

    pub fn distance_to(&self, other:&Vector) -> f64 {
        let v = self.subtract(other);
        v.len()
    }

    pub fn unit_vector(&self) -> Vector {
        let l = self.len();
        if l == 0.0 {
            Vector::default()
        } else {
            Vector{
                x:self.x / l,
                y:self.y / l,
                z:self.z / l
            }
        }
    }

    pub fn normalized(&self) -> Vector {
        let mut l = self.len();
        if l == 0.0 {
            l = 1.0;
        }
        Vector{
            x:self.x / l,
            y:self.y / l,
            z:self.z / l
        }
    }

    pub fn normalize(&mut self) {
        let n = self.normalized();
        self.x = n.x;
        self.y = n.y;
        self.z = n.z;
    }

    pub fn inversed(&self) -> Vector {
        Vector{
            x:self.x * -1.0,
            y:self.y * -1.0,
            z:self.z * -1.0
        }
    }

    pub fn inverse(&mut self) {
        let i = self.inversed();
        self.x = i.x;
        self.y = i.y;
        self.z = i.z;
    }

    pub fn dot_product(&self, other:&Vector) -> f64 {
        let v0 = self.normalized();
        let v1 = other.normalized();
        v0.x * v1.x + v0.y * v1.y + v0.z * v1.z
    }

    pub fn cross_product(&self, other:&Vector) -> Vector {
        Vector{
            x: self.y * other.z - other.y * self.z,
            y: self.z * other.x - other.z * self.x,
            z: self.x * other.y - other.x * self.y
        }
    }

    pub fn angle(&self, other:&Vector) -> f64 {
        let dot = self.dot_product(other);
        degrees(radians(dot).acos())
    }
    
    pub fn subtract(&self, other:&Vector) -> Vector {
        Vector{
            x:self.x - other.x,
            y:self.y - other.y,
            z:self.z - other.z
        }
    }

    pub fn add(&self, other:&Vector) -> Vector {
        Vector{
            x:self.x + other.x,
            y:self.y + other.y,
            z:self.z + other.z
        }
    }

    pub fn direction_to(&self, other:&Vector) -> Vector {
        other.subtract(self).normalized()
    }


    pub fn rotate(&self, angle:f64, axis:Axis) -> Vector {
        match axis {
            Axis::XAxis => self.rotate_x(angle),
            Axis::YAxis => self.rotate_y(angle),
            Axis::ZAxis => self.rotate_z(angle),
        }
    }

    pub fn rotate_x(&self, angle:f64) -> Vector {
        if angle > 0.0 {
            let x = radians(self.x);

            let cos_x = x.cos();
            let sin_x = x.sin();

            let ry = cos_x * self.y + -sin_x * self.z;
            let rz = sin_x * self.y + cos_x * self.z;

            Vector{
                x: self.x,
                y: ry,
                z: rz
            }

        } else {
            self.clone()
        }
    }

    pub fn rotate_y(&self, angle:f64) -> Vector {
        if angle > 0.0 {
            let y = radians(self.y);

            let cos_y = y.cos();
            let sin_y = y.sin();

            let rx = cos_y * self.x + sin_y * self.z;
            let rz = -sin_y * self.x + cos_y * self.z;

            Vector{
                x:rx,
                y:self.y,
                z:rz
            }
        }else {
            self.clone()
        }
    }

    pub fn rotate_z(&self, angle:f64) -> Vector {
        if angle > 0.0 {
            let z = radians(self.z);
            
            let cos_z = z.cos();
            let sin_z = z.sin();

            let rx = cos_z * self.x + -sin_z * self.y;
            let ry = sin_z * self.x + cos_z * self.y;

            Vector{
                x:rx,
                y:ry,
                z:self.z
            }
        }else {
            self.clone()
        }
    }
    
    pub fn translate(&self, x:f64, y:f64, z:f64) -> Vector {
        Vector{
            x:self.x + x,
            y:self.y + y,
            z:self.z + z
        }
    }

    pub fn normal_2pt(pt0:&Vector, pt1:&Vector) -> Vector {
        Vector{
            x: pt0.y * pt1.z - pt1.z * pt0.z, 
            y: pt0.x * pt1.z - pt1.x * pt0.z, // Ummm....
            z: pt0.x * pt1.y - pt1.x * pt0.y
        }
    }
    
    pub fn normal_3pt(pt0:&Vector, pt1:&Vector, pt2:&Vector) -> Vector {
        let b0 = pt0.subtract(pt1);
        let b1 = pt1.subtract(pt2);

        let cp = b0.cross_product(&b1);
        cp.normalized()
    }

}