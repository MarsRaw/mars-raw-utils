use crate::{
    prelude::*,
    projection::{
        cylindrical,
        polar::{self, PolarView},
    },
};
use anyhow::Result;
use sciimg::image::Image;

pub fn create_composite(
    in_files: &[String],
    anaglyph: bool,
    azimuth_rotation: f64,
    max_edge_length_polar: usize,
    polar_view: PolarView,
    projection: Projection,
    draw_grid: bool,
) -> Result<Image> {
    match projection {
        Projection::Equirectangular => cylindrical::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            cylindrical::CylMap::Equirectangular,
            draw_grid,
        ),
        Projection::EqualArea => cylindrical::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            cylindrical::CylMap::EqualArea,
            draw_grid,
        ),
        Projection::Perspective => cylindrical::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            cylindrical::CylMap::Perspective,
            draw_grid,
        ),
        Projection::Mercator => cylindrical::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            cylindrical::CylMap::Mercator,
            draw_grid,
        ),
        Projection::PolarEqualArea => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::EqualArea,
            max_edge_length_polar,
            polar_view,
            draw_grid,
        ),
        Projection::PolarEquidistant => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Equidistant,
            max_edge_length_polar,
            polar_view,
            draw_grid,
        ),
        Projection::PolarOrthographic => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Orthographic,
            max_edge_length_polar,
            polar_view,
            draw_grid,
        ),
        Projection::PolarStereographic => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Stereographic,
            max_edge_length_polar,
            polar_view,
            draw_grid,
        ),
        Projection::PolarRectilinear => polar::create_composite(
            in_files,
            anaglyph,
            azimuth_rotation,
            polar::RadialMap::Rectilinear,
            max_edge_length_polar,
            polar_view,
            draw_grid,
        ),
    }
}
