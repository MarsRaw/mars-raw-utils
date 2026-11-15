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
    projection: Projection,
) -> Result<Image> {
    match projection {
        Projection::Cylindrical => {
            cylindrical::create_composite(in_files, anaglyph, azimuth_rotation)
        }
        Projection::PolarEqualArea => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::EqualArea,
        ),
        Projection::PolarEquidistant => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Equidistant,
        ),
        Projection::PolarOrthographic => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Orthographic,
        ),
        Projection::PolarStereographic => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Stereographic,
        ),
    }
}
