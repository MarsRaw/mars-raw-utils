use anyhow::Result;
use sciimg::{
    camera::model::LookVector, image::Image, not_implemented, quaternion::Quaternion,
    vector::Vector,
};

static SPHERE_RADIUS: f64 = 100.0;

#[derive(Debug, Clone, Copy)]
pub struct PolarXY {
    pub x: f64,
    pub y: f64,
}

/// Which pole sits at the center of the polar plot.
pub enum Pole {
    Zenith, // up
    Nadir,  // down
}

// impl Pole {
//     pub fn cos_theta(self:Self, )
// }

/// Radial mapping from the angle off the center pole.
pub enum RadialMap {
    /// r = angle from pole (equidistant / azimuthal equidistant)
    Equidistant,
    /// r = 2 * tan(angle / 2) (stereographic, conformal)
    Stereographic,
    /// r = 2 * sin(angle / 2) (equal-area, Lambert azimuthal)
    EqualArea,
    /// r = sin(angle) (orthographic, true "from above" look)
    Orthographic,
}

impl RadialMap {
    pub fn radial(self, theta: f64) -> f64 {
        match self {
            RadialMap::Equidistant => theta,
            RadialMap::Stereographic => 2.0 * (theta / 2.0).tan(),
            RadialMap::EqualArea => 2.0 * (theta / 2.0).sin(),
            RadialMap::Orthographic => theta.sin(),
        }
    }
}

pub fn vector_to_polar(v: &Vector, pole: Pole, map: RadialMap) -> PolarXY {
    // Angle between the vector and the chosen pole, in [0, pi]
    let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    let cos_theta = match pole {
        Pole::Zenith => v.z / len,
        Pole::Nadir => -v.z / len,
    };
    let theta = cos_theta.clamp(-1.0, 1.0).acos();

    let r = map.radial(theta);

    // Azimuth, same definition as the cylindrical version
    let az = v.y.atan2(v.x);

    PolarXY {
        x: r * az.cos(),
        y: r * az.sin(),
    }
}

pub fn lookvector_to_polar(
    lv: &LookVector,
    quat: Option<&Quaternion>,
    origin_diff: &Vector,
    pole: Pole,
    map: RadialMap,
) -> PolarXY {
    let ray = lv.intersect_to_sphere(SPHERE_RADIUS);
    let ray_moved = ray.subtract(origin_diff);
    let rotated = if let Some(quat) = quat {
        quat.rotate_vector(&ray_moved)
    } else {
        ray
    };
    vector_to_polar(&rotated, pole, map)
}

pub fn create_composite(
    _in_files: &[String],
    _anaglyph: bool,
    azimuth_rotation: f64,
    _radial_map: RadialMap,
) -> Result<Image> {
    let _quat = Quaternion::from_pitch_roll_yaw(0.0, 0.0, azimuth_rotation.to_radians());

    not_implemented!()
}
