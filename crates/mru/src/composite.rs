use crate::{
    prelude::*,
    projection::{cylindrical, polar},
};
use anyhow::Result;
use sciimg::image::Image;

pub fn create_composite(
    in_files: &[String],
    anaglyph: bool,
    azimuth_rotation: f64,
    max_edge_length_polar: usize,
    max_radius_polar: f64,
    eye_height_polar: Option<f64>,
    projection: Projection,
) -> Result<Image> {
    match projection {
        Projection::Equirectangular => cylindrical::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            cylindrical::CylMap::Equirectangular,
        ),
        Projection::EqualArea => cylindrical::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            cylindrical::CylMap::EqualArea,
        ),
        Projection::Perspective => cylindrical::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            cylindrical::CylMap::Perspective,
        ),
        Projection::Mercator => cylindrical::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            cylindrical::CylMap::Mercator,
        ),
        Projection::PolarEqualArea => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::EqualArea,
            max_edge_length_polar,
            max_radius_polar,
            eye_height_polar,
        ),
        Projection::PolarEquidistant => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Equidistant,
            max_edge_length_polar,
            max_radius_polar,
            eye_height_polar,
        ),
        Projection::PolarOrthographic => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Orthographic,
            max_edge_length_polar,
            max_radius_polar,
            eye_height_polar,
        ),
        Projection::PolarStereographic => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Stereographic,
            max_edge_length_polar,
            max_radius_polar,
            eye_height_polar,
        ),
        Projection::PolarRectilinear => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Rectilinear,
            max_edge_length_polar,
            max_radius_polar,
            eye_height_polar,
        ),
    }
}
