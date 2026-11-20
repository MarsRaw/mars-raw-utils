use std::str::FromStr as _;

use crate::{
    enums::{Eye, Instrument, Projection},
    marsimage::MarsImage,
    prelude::WrapCylindrical as _,
    util,
};
use anyhow::{Result, anyhow, bail};
use sciimg::{
    camera::{
        cahvore::Cahvore,
        model::{ImageCoordinate, LookVector},
    },
    drawable::{Drawable, Point, Triangle},
    image::Image,
    path,
    quaternion::Quaternion,
    vector::Vector,
};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

static SPHERE_RADIUS: f64 = 100.0;

#[derive(Clone, Copy)]
pub enum CylMap {
    Equirectangular,
    Perspective,
    EqualArea,
    Mercator,
}

impl CylMap {
    pub fn project(&self, az: f64, el: f64) -> (f64, f64) {
        let el_c = el.clamp(-FRAC_PI_2 + 1e-6, FRAC_PI_2 - 1e-6);
        match self {
            CylMap::Equirectangular => (az, el),
            CylMap::Perspective => (az, el_c.tan()),
            CylMap::EqualArea => (az, el.sin()),
            CylMap::Mercator => (az, (FRAC_PI_4 + el_c / 2.0).tan().ln()),
        }
    }

    pub fn from_projection(projection: &Projection) -> Result<CylMap> {
        match projection {
            Projection::Equirectangular => Ok(CylMap::Equirectangular),
            Projection::EqualArea => Ok(CylMap::EqualArea),
            Projection::Perspective => Ok(CylMap::Perspective),
            Projection::Mercator => Ok(CylMap::Mercator),
            _ => Err(anyhow!("Invalid cylindrical projection")),
        }
    }
}

fn vector_to_cylindrical(v: &Vector, cyl_map: CylMap) -> AzEl {
    let (az, el) = cyl_map.project(
        v.y.atan2(v.x).wrap_tau(),
        (v.z).atan2(v.x.hypot(v.y)).wrap_pi(),
    );

    AzEl {
        el: -el.to_degrees(),
        az: az.to_degrees(),
    }
}

fn lookvector_to_cylindrical(
    lv: &LookVector,
    quat: &Option<Quaternion>,
    origin_diff: &Vector,
    cyl_map: CylMap,
) -> AzEl {
    let ray = lv.intersect_to_sphere(SPHERE_RADIUS);
    let ray_moved = ray.subtract(origin_diff);
    let rotated = if let Some(quat) = quat {
        quat.rotate_vector(&ray_moved)
    } else {
        ray
    };
    vector_to_cylindrical(&rotated, cyl_map)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CylindricalMapExtent {
    pub top_lat: f64,
    pub bottom_lat: f64,
    pub left_lon: f64,
    pub right_lon: f64,
    pub width: usize,
    pub height: usize,
    pub degrees_per_pixel: f64,
}

impl Default for CylindricalMapExtent {
    fn default() -> Self {
        CylindricalMapExtent {
            top_lat: f64::MIN,
            bottom_lat: f64::MAX,
            left_lon: f64::MAX,
            right_lon: f64::MIN,
            width: 0,
            height: 0,
            degrees_per_pixel: 0.0,
        }
    }
}

impl CylindricalMapExtent {
    pub fn union_azel(&mut self, other: &AzEl) {
        self.bottom_lat = self.bottom_lat.min(other.el);
        self.top_lat = self.top_lat.max(other.el);
        self.left_lon = self.left_lon.min(other.az);
        self.right_lon = self.right_lon.max(other.az);
    }

    pub fn union(&mut self, other: &CylindricalMapExtent) {
        self.bottom_lat = self.bottom_lat.min(other.bottom_lat);
        self.top_lat = self.top_lat.max(other.top_lat);
        self.left_lon = self.left_lon.min(other.left_lon);
        self.right_lon = self.right_lon.max(other.right_lon);
        self.degrees_per_pixel = self.degrees_per_pixel.max(other.degrees_per_pixel);
    }

    pub fn wrap_coord_extents(&mut self) {
        self.left_lon = self.left_lon.wrap_tau_degrees();
        self.right_lon = self.right_lon.wrap_tau_degrees();
    }

    pub fn dimensions_from_extents(&mut self) {
        self.height = ((self.top_lat - self.bottom_lat) / self.degrees_per_pixel).floor() as usize;
        self.width = ((self.right_lon - self.left_lon) / self.degrees_per_pixel).floor() as usize;
    }

    fn image_ls_to_az_el(
        model: &Cahvore,
        quat: &Option<Quaternion>,
        img_ls: &ImageCoordinate,
        cyl_map: CylMap,
    ) -> Option<AzEl> {
        match model.ls_to_look_vector(img_ls) {
            Ok(lv) => Some(lookvector_to_cylindrical(
                &lv,
                quat,
                &Vector::default(),
                cyl_map,
            )),
            Err(_) => None,
        }
    }

    fn determine_map_extent_for_image(
        img: &MarsImage,
        quat: &Option<Quaternion>,
        cyl_map: CylMap,
    ) -> Result<CylindricalMapExtent> {
        let mut context = CylindricalMapExtent::default();

        let scale_factor = img.get_scale_factor();
        let model = match img.get_cahvor() {
            Some(model) => model.scale(scale_factor as f64, scale_factor as f64),
            None => bail!("Error: Invalid or missing camera model"),
        };

        let (x0, xh, x1) = (
            -0.5,
            (img.image.width as f64 / 2.0 - 0.5) * scale_factor as f64,
            (img.image.width as f64 - 0.5) * scale_factor as f64,
        );
        let (y0, yh, y1) = (
            -0.5,
            (img.image.height as f64 / 2.0 - 0.5) * scale_factor as f64,
            (img.image.height as f64 - 0.5) * scale_factor as f64,
        );

        // Here we check the four corners and the middle point of each four sides
        [
            (x0, y0),
            (x1, y0),
            (x0, y1),
            (x1, y1),
            (xh, y0),
            (xh, y1),
            (x0, yh),
            (x1, yh),
        ]
        .iter()
        .for_each(|(x, y)| {
            if let Some(azel) = CylindricalMapExtent::image_ls_to_az_el(
                &model,
                quat,
                &ImageCoordinate {
                    line: *y,
                    sample: *x,
                },
                cyl_map,
            ) {
                context.union_azel(&azel);
            }
        });

        let ang_horiz = model.pixel_angle_horiz().to_degrees();
        context.degrees_per_pixel = context.degrees_per_pixel.max(ang_horiz);

        Ok(context)
    }

    pub fn new_from_files(
        input_files: &[String],
        quat: Option<Quaternion>,
        cyl_map: CylMap,
    ) -> Result<CylindricalMapExtent> {
        let mut context = CylindricalMapExtent::default();

        input_files.iter().try_for_each(|input_file| {
            let img = match MarsImage::open(input_file) {
                Ok(img) => img,
                Err(why) => bail!("Failed to open file {}: {:?}", input_file, why),
            };

            let map_extent =
                CylindricalMapExtent::determine_map_extent_for_image(&img, &quat, cyl_map)?;
            context.union(&map_extent);
            Ok(())
        })?;

        context.wrap_coord_extents();
        context.dimensions_from_extents();

        Ok(context)
    }
}

#[derive(Debug)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

/////////////////////////////////
/////////////////////////////////

pub struct AzEl {
    pub az: f64,
    pub el: f64,
}

pub struct MapCanvas {
    pub ctx: CylindricalMapExtent,
    pub quat: Option<Quaternion>,
    pub cyl_map: CylMap,
    pub canvas: Image,
    pub anaglyph: bool,
}

impl MapCanvas {
    pub fn new(
        ctx: CylindricalMapExtent,
        quat: Option<Quaternion>,
        cyl_map: CylMap,
        anaglyph: bool,
        transparent: bool,
    ) -> MapCanvas {
        MapCanvas {
            ctx,
            quat,
            cyl_map,
            canvas: Image::create_masked(ctx.width, ctx.height, !transparent),
            anaglyph,
        }
    }

    fn vector_at_az_el(az: f64, el: f64) -> Vector {
        Vector {
            x: el.cos() * az.cos(),
            y: el.cos() * az.sin(),
            z: -el.sin(),
        }
    }

    fn az_el_to_map_xy(&self, az_el: &AzEl) -> ImageCoordinate {
        let y = self.ctx.height as f64
            - ((az_el.el - self.ctx.bottom_lat) / (self.ctx.top_lat - self.ctx.bottom_lat)
                * self.ctx.height as f64);
        let x = (az_el.az - self.ctx.left_lon) / (self.ctx.right_lon - self.ctx.left_lon)
            * self.ctx.width as f64;
        ImageCoordinate { line: y, sample: x }
    }

    fn vector_to_cylindrical(&self, v: &Vector) -> AzEl {
        vector_to_cylindrical(v, self.cyl_map)
    }

    fn vector_to_map_xy(&self, v: &Vector) -> Option<ImageCoordinate> {
        // Images are rotated by `quat` before projection, so the vector must be too
        let rotated = match &self.quat {
            Some(quat) => quat.rotate_vector(v),
            None => v.to_owned(),
        };
        let mut ll = self.vector_to_cylindrical(&rotated);

        // lon is az in [0, 360); shift by a turn if the extent sits across the seam
        if ll.az < self.ctx.left_lon {
            ll.az += 360.0;
        } else if ll.az > self.ctx.right_lon {
            ll.az -= 360.0;
        }

        let map_xy = self.az_el_to_map_xy(&ll);
        let inside = map_xy.sample >= 0.0
            && map_xy.sample < self.ctx.width as f64
            && map_xy.line >= 0.0
            && map_xy.line < self.ctx.height as f64;
        inside.then_some(map_xy)
    }

    fn image_ls_to_map_xy(
        &self,
        model: &Cahvore,
        img_ls: &ImageCoordinate,
        origin_diff: &Vector,
    ) -> Option<ImageCoordinate> {
        let lv = match model.ls_to_look_vector(img_ls) {
            Ok(lv) => lv,
            Err(_) => return None,
        };
        let azel = lookvector_to_cylindrical(&lv, &self.quat, origin_diff, self.cyl_map);
        Some(self.az_el_to_map_xy(&azel))
    }

    //////////////////////////////////
    //// Grids
    //////////////////////////////////

    fn draw_horizontal_gridline_at_elevation(&mut self, elevation: f64, data_value: f32) {
        let steps = self.ctx.width.max(1) * 2;

        for i in 0..steps {
            let az = i as f64 / steps as f64 * std::f64::consts::TAU;
            let v = MapCanvas::vector_at_az_el(az, elevation.to_radians());
            if let Some(ic) = self.vector_to_map_xy(&v) {
                for b in 0..self.canvas.bands.len() {
                    self.canvas.put(
                        ic.sample.floor() as usize,
                        ic.line.floor() as usize,
                        data_value,
                        b,
                    );
                }
            }
        }
    }

    fn draw_vertical_gridline_at_azimuth(&mut self, azimuth: f64, data_value: f32) {
        let steps = self.ctx.height.max(1) * 2;

        for i in 0..steps {
            let el = i as f64 / steps as f64 * std::f64::consts::PI - std::f64::consts::PI / 2.0;
            let v = MapCanvas::vector_at_az_el(azimuth.to_radians(), el);
            if let Some(ic) = self.vector_to_map_xy(&v) {
                for b in 0..self.canvas.bands.len() {
                    self.canvas.put(
                        ic.sample.floor() as usize,
                        ic.line.floor() as usize,
                        data_value,
                        b,
                    );
                }
            }
        }
    }

    fn draw_grid(&mut self, data_value: f32, azimuth_step: usize, elevation_step: usize) {
        let grid_bottom = ((self.ctx.bottom_lat / 10.0).floor() * 10.0) as i32;
        let grid_top = ((self.ctx.top_lat / 10.0).ceil() * 10.0) as i32;

        let grid_left = ((self.ctx.left_lon / 10.0).floor() * 10.0) as i32;
        let grid_right = ((self.ctx.right_lon / 10.0).ceil() * 10.0) as i32;

        for el in (grid_bottom..=grid_top).step_by(elevation_step) {
            self.draw_horizontal_gridline_at_elevation(el as f64, data_value);
        }

        for az in (grid_left..=grid_right).step_by(azimuth_step) {
            self.draw_vertical_gridline_at_azimuth(az as f64, data_value);
        }
    }

    //////////////////////////////////
    //// Images
    //////////////////////////////////

    pub fn project_file_to_canvas(
        &mut self,
        input_file: &str,
        initial_origin: &Vector,
    ) -> Result<()> {
        let mut img = MarsImage::open(input_file)?;

        img.instrument = if let Some(rover_metadata) = img.metadata.as_surface() {
            Instrument::from_str(rover_metadata.instrument.as_str()).unwrap()
        } else {
            return Err(anyhow!(
                "Composite via CAHVORE camera model not supported for non-surface missions"
            ));
        };

        let eye = if self.anaglyph {
            match util::filename_char_at_pos(input_file, 1) {
                'R' => Eye::Right,
                'L' => Eye::Left,
                _ => Eye::DontCare,
            }
        } else {
            Eye::DontCare
        };

        let subframe = img.get_subframe_region();
        let scale_factor = img.get_scale_factor();

        let mut input_model = match img.get_cahvor() {
            Some(input_model) => input_model,
            None => return Err(anyhow!("CAHVOR not found for image, cannot continue")),
        };

        input_model = input_model.scale(scale_factor as f64, scale_factor as f64);

        let band_0 = img.image.get_band(0);
        let band_1 = img.image.get_band(1);
        let band_2 = img.image.get_band(2);

        let origin_diff = input_model.c().subtract(initial_origin);

        for x in 0..(img.image.width - 1) {
            for y in 0..(img.image.height - 1) {
                if !band_0.get_mask_at_point(x, y) || !img.image.get_alpha_at(x, y) {
                    continue;
                }

                let full_frame_x = (x + subframe.x) * scale_factor;
                let full_frame_y = (y + subframe.y) * scale_factor;

                let tl_ls = self
                    .image_ls_to_map_xy(
                        &input_model,
                        &ImageCoordinate {
                            line: full_frame_y as f64,
                            sample: full_frame_x as f64,
                        },
                        &origin_diff,
                    )
                    .map(|ls| {
                        Point::create_rgb(
                            ls.sample,
                            ls.line,
                            band_0.get(x, y) as f64,
                            band_1.get(x, y) as f64,
                            band_2.get(x, y) as f64,
                        )
                    });

                let tr_ls = self
                    .image_ls_to_map_xy(
                        &input_model,
                        &ImageCoordinate {
                            line: full_frame_y as f64,
                            sample: (full_frame_x + scale_factor) as f64,
                        },
                        &origin_diff,
                    )
                    .map(|ls| {
                        Point::create_rgb(
                            ls.sample,
                            ls.line,
                            band_0.get(x + 1, y) as f64,
                            band_1.get(x + 1, y) as f64,
                            band_2.get(x + 1, y) as f64,
                        )
                    });

                let bl_ls = self
                    .image_ls_to_map_xy(
                        &input_model,
                        &ImageCoordinate {
                            line: (full_frame_y + scale_factor) as f64,
                            sample: full_frame_x as f64,
                        },
                        &origin_diff,
                    )
                    .map(|ls| {
                        Point::create_rgb(
                            ls.sample,
                            ls.line,
                            band_0.get(x, y + 1) as f64,
                            band_1.get(x, y + 1) as f64,
                            band_2.get(x, y + 1) as f64,
                        )
                    });

                let br_ls = self
                    .image_ls_to_map_xy(
                        &input_model,
                        &ImageCoordinate {
                            line: (full_frame_y + scale_factor) as f64,
                            sample: (full_frame_x + scale_factor) as f64,
                        },
                        &origin_diff,
                    )
                    .map(|ls| {
                        Point::create_rgb(
                            ls.sample,
                            ls.line,
                            band_0.get(x + 1, y + 1) as f64,
                            band_1.get(x + 1, y + 1) as f64,
                            band_2.get(x + 1, y + 1) as f64,
                        )
                    });

                if let Some(tl) = &tl_ls
                    && let Some(bl) = &bl_ls
                    && let Some(tr) = &tr_ls
                {
                    self.canvas.paint_tri_with_channel_rule(
                        &Triangle {
                            p0: tl.to_owned(),
                            p1: bl.to_owned(),
                            p2: tr.to_owned(),
                        },
                        false,
                        |c| {
                            (c == 0 && matches!(eye, Eye::Left | Eye::DontCare))
                                || ((c == 1 || c == 2) && matches!(eye, Eye::Right | Eye::DontCare))
                        },
                    );
                }

                if let Some(tr) = &tr_ls
                    && let Some(bl) = &bl_ls
                    && let Some(br) = &br_ls
                {
                    self.canvas.paint_tri_with_channel_rule(
                        &Triangle {
                            p0: tr.to_owned(),
                            p1: bl.to_owned(),
                            p2: br.to_owned(),
                        },
                        false,
                        |c| {
                            (c == 0 && matches!(eye, Eye::Left | Eye::DontCare))
                                || ((c == 1 || c == 2) && matches!(eye, Eye::Right | Eye::DontCare))
                        },
                    );
                }
            }
        }

        Ok(())
    }
}

pub fn create_composite(
    in_files: &[String],
    anaglyph: bool,
    azimuth_rotation: f64,
    cyl_map: CylMap,
    draw_grid: bool,
    transparent: bool,
) -> Result<Image> {
    let quat = Quaternion::from_pitch_roll_yaw(0.0, 0.0, azimuth_rotation.to_radians());

    let map_context = CylindricalMapExtent::new_from_files(in_files, Some(quat), cyl_map)?;
    debug!("Map Context: {:?}", map_context);
    debug!(
        "FOV Vertical: {}",
        map_context.top_lat - map_context.bottom_lat
    );
    debug!(
        "FOV Horizontal: {}",
        map_context.right_lon - map_context.left_lon
    );

    if map_context.width == 0 {
        return Err(anyhow!("Error: Output expected to have zero width"));
    } else if map_context.height == 0 {
        return Err(anyhow!("Error: output expected to have zero height"));
    }

    let first_image = MarsImage::open(&in_files[0])?;
    let initial_origin = if let Some(model) = first_image.get_cahvor() {
        model.c()
    } else {
        return Err(anyhow!("Error: Cannot determine initial camera origin"));
    };

    let mut canvas = MapCanvas::new(map_context, Some(quat), cyl_map, anaglyph, transparent);

    if draw_grid {
        canvas.draw_grid(65535.0, 10, 10);
    }

    in_files.iter().try_for_each(|in_file| {
        if path::file_exists(in_file) {
            info!("Processing File: {}", in_file);
            canvas.project_file_to_canvas(in_file, &initial_origin)
        } else {
            bail!("Error: input file not found: {}", in_file)
        }
    })?;

    Ok(canvas.canvas)
}
