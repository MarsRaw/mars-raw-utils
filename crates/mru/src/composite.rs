use crate::{
    prelude::*,
    projection::{
        cylindrical::{self, CylMap, CylindricalMapExtent},
        polar::{self, PolarMapExtent, PolarView, RadialMap},
    },
};
use anyhow::Result;
use sciimg::{image::Image, quaternion::Quaternion, vector::Vector};

#[derive(Debug, Clone)]
pub enum ProjectionCont {
    Polar(RadialMap),
    Cylindrical(CylMap),
}

impl ProjectionCont {
    pub fn from_projection(projection: Projection) -> ProjectionCont {
        match projection {
            Projection::Equirectangular => Self::Cylindrical(CylMap::Equirectangular),
            Projection::Perspective => Self::Cylindrical(CylMap::Perspective),
            Projection::EqualArea => Self::Cylindrical(CylMap::EqualArea),
            Projection::Mercator => Self::Cylindrical(CylMap::Mercator),
            Projection::PolarEquidistant => Self::Polar(RadialMap::Equidistant),
            Projection::PolarStereographic => Self::Polar(RadialMap::Stereographic),
            Projection::PolarEqualArea => Self::Polar(RadialMap::EqualArea),
            Projection::PolarOrthographic => Self::Polar(RadialMap::Orthographic),
            Projection::PolarRectilinear => Self::Polar(RadialMap::Rectilinear),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExtentCont {
    Polar(PolarMapExtent),
    Cylindrical(CylindricalMapExtent),
}

impl ExtentCont {
    pub fn determine_from_files(
        in_files: &[String],
        azimuth_rotation: f64,
        projection: Projection,
        max_edge_length: usize,
        view: PolarView,
    ) -> Result<ExtentCont> {
        let quat = Quaternion::from_pitch_roll_yaw(0.0, 0.0, azimuth_rotation.to_radians());

        match projection {
            Projection::EqualArea => Ok(ExtentCont::Cylindrical(
                CylindricalMapExtent::new_from_files(in_files, Some(quat), CylMap::EqualArea)?,
            )),
            Projection::Equirectangular => Ok(ExtentCont::Cylindrical(
                CylindricalMapExtent::new_from_files(
                    in_files,
                    Some(quat),
                    CylMap::Equirectangular,
                )?,
            )),
            Projection::Perspective => Ok(ExtentCont::Cylindrical(
                CylindricalMapExtent::new_from_files(in_files, Some(quat), CylMap::Perspective)?,
            )),
            Projection::Mercator => Ok(ExtentCont::Cylindrical(
                CylindricalMapExtent::new_from_files(in_files, Some(quat), CylMap::Mercator)?,
            )),
            Projection::PolarEquidistant => Ok(ExtentCont::Polar(PolarMapExtent::new_from_files(
                in_files,
                Some(quat),
                max_edge_length,
                view,
                RadialMap::Equidistant,
            )?)),
            Projection::PolarStereographic => {
                Ok(ExtentCont::Polar(PolarMapExtent::new_from_files(
                    in_files,
                    Some(quat),
                    max_edge_length,
                    view,
                    RadialMap::Stereographic,
                )?))
            }
            Projection::PolarEqualArea => Ok(ExtentCont::Polar(PolarMapExtent::new_from_files(
                in_files,
                Some(quat),
                max_edge_length,
                view,
                RadialMap::EqualArea,
            )?)),
            Projection::PolarOrthographic => Ok(ExtentCont::Polar(PolarMapExtent::new_from_files(
                in_files,
                Some(quat),
                max_edge_length,
                view,
                RadialMap::Orthographic,
            )?)),
            Projection::PolarRectilinear => Ok(ExtentCont::Polar(PolarMapExtent::new_from_files(
                in_files,
                Some(quat),
                max_edge_length,
                view,
                RadialMap::Rectilinear,
            )?)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompositeOptions {
    pub in_files: Vec<String>,
    pub anaglyph: bool,
    pub azimuth_rotation: f64,
    pub max_edge_length_polar: usize,
    pub polar_view: PolarView,
    pub projection: ProjectionCont,
    pub draw_grid: bool,
    pub transparent: bool,
    pub initial_origin: Option<Vector>,
    pub map_extent: Option<ExtentCont>,
}

impl Default for CompositeOptions {
    fn default() -> Self {
        Self {
            in_files: Default::default(),
            anaglyph: Default::default(),
            azimuth_rotation: Default::default(),
            max_edge_length_polar: Default::default(),
            polar_view: Default::default(),
            projection: ProjectionCont::Cylindrical(CylMap::Equirectangular),
            draw_grid: Default::default(),
            transparent: Default::default(),
            initial_origin: Default::default(),
            map_extent: None,
        }
    }
}

pub fn create_composite(options: CompositeOptions) -> Result<Image> {
    match options.projection {
        ProjectionCont::Cylindrical(_) => cylindrical::create_composite(options),
        ProjectionCont::Polar(_) => polar::create_composite(options),
    }
}
