
use crate::enums::*;
use sciimg::{
    prelude::*,
    min,
    max
};

/// A single two-dimensional point on a raster. Contains the x/y coordinate and the RGB values to be placed
/// into the buffer.
#[derive(Debug, Clone)]
pub struct Point {
    pub x:f64,
    pub y:f64,
    pub r:f64,
    pub g:f64,
    pub b:f64
}

impl Point{

    /// Simple creation for a Point.
    pub fn create(x:f64, y:f64, r:f64, g:f64, b:f64) -> Self {
        Point{
            x:x,
            y:y,
            r:r,
            g:g,
            b:b
        }
    }
}

/// A triangle (polygon) of three two-dimensional points
pub struct Triangle {
    pub p0:Point,
    pub p1:Point,
    pub p2:Point
}

impl Triangle {

    /// Determine if a two dimensional point is contained within the  area bounded by the triangle
    pub fn contains(&self, x:f64, y:f64) -> bool {
        let p = Point{x:x, y:y, r:0.0, g:0.0, b:0.0};
        let b0 = Triangle::sign(&p, &self.p0, &self.p1) <= 0.0;
        let b1 = Triangle::sign(&p, &self.p1, &self.p2) <= 0.0;
        let b2 = Triangle::sign(&p, &self.p2, &self.p0) <= 0.0;

        (b0 == b1) && (b1 == b2)
    }

    
    pub fn sign(p0:&Point, p1:&Point, p2:&Point) -> f64 {
        (p0.x - p2.x) * (p1.y - p2.y) - (p1.x - p2.x) * (p0.y - p2.y)
    }

    pub fn x_min(&self) -> f64 {
        min!(self.p0.x, self.p1.x, self.p2.x)
    }

    pub fn x_max(&self) -> f64 {
        max!(self.p0.x, self.p1.x, self.p2.x)
    }

    pub fn y_min(&self) -> f64 {
        min!(self.p0.y, self.p1.y, self.p2.y)
    }

    pub fn y_max(&self) -> f64 {
        max!(self.p0.y, self.p1.y, self.p2.y)
    }

    /// Determines an interpolated single-channel color value for a point in the triangle
    pub fn interpolate_color_channel(&self, x:f64, y:f64, c0:f64, c1:f64, c2:f64) -> f64 {
        let det = self.p0.x * self.p1.y - self.p1.x * self.p0.y + self.p1.x * self.p2.y - self.p2.x * self.p1.y + self.p2.x * self.p0.y - self.p0.x * self.p2.y;
        let a = ((self.p1.y-self.p2.y)*c0+(self.p2.y-self.p0.y)*c1+(self.p0.y-self.p1.y)*c2) / det;
        let b = ((self.p2.x-self.p1.x)*c0+(self.p0.x-self.p2.x)*c1+(self.p1.x-self.p0.x)*c2) / det;
        let c = ((self.p1.x*self.p2.y-self.p2.x*self.p1.y)*c0+(self.p2.x*self.p0.y-self.p0.x*self.p2.y)*c1+(self.p0.x*self.p1.y-self.p1.x*self.p0.y)*c2) / det;

        let v = a*x+b*y+c;
        v
    }

    /// Determines an interpolated three-channel (RGB) color value for a point in the triangle
    pub fn interpolate_color(&self, x:f64, y:f64) -> (f64, f64, f64) {
        let r = self.interpolate_color_channel(x, y, self.p0.r, self.p1.r, self.p2.r);
        let g = self.interpolate_color_channel(x, y, self.p0.g, self.p1.g, self.p2.g);
        let b = self.interpolate_color_channel(x, y, self.p0.b, self.p1.b, self.p2.b);
        (r, g, b)
    }

}

/// Defines a buffer that can be drawn on using triangle or square polygons.
pub trait Drawable {

    /// Create a simple three-channel image buffer
    fn create(width:usize, height:usize) -> Self;

    /// Paint a triangle on the buffer.
    fn paint_tri(&mut self, tri:&Triangle, avg_pixels:bool, eye:Eye);

    /// Paint a square on the buffer using four points
    fn paint_square(&mut self, tl:&Point, bl:&Point, br:&Point, tr:&Point, avg_pixels:bool, eye:Eye);

    /// Width of the buffer
    fn get_width(&self) -> usize;

    /// Height of the buffer
    fn get_height(&self) -> usize;

    /// Converts color to mono
    fn to_mono(&mut self);
}

/// Implements the Drawable trait for the RgbImage class. This is probably later be merged fully into RgbImage
/// in the sciimg crate.
impl Drawable for RgbImage {
    fn create(width:usize, height:usize) -> Self {
        RgbImage::new_with_bands(width, height, 3, ImageMode::U16BIT).unwrap()
    }

    fn get_width(&self) -> usize {
        self.width
    }

    fn get_height(&self) -> usize {
        self.height
    }

    fn paint_tri(&mut self, tri:&Triangle, avg_pixels:bool, eye:Eye) {

        let min_x = tri.x_min().floor() as usize;
        let max_x = tri.x_max().ceil() as usize;
        let min_y = tri.y_min().floor() as usize;
        let max_y = tri.y_max().ceil() as usize;

        // Gonna limit the max dimension of a poly to just 100x100 
        // to prevent those that wrap the entire image. 
        // Until I plan out a better control to handle polygons that
        // wrap the cut-off azimuth
        if max_x - min_x < 100 && max_y - min_y <  100 {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if x < self.width && y < self.height && tri.contains(x as f64, y as f64) {
                        let (mut r, mut g, mut b) = tri.interpolate_color(x as f64,y as f64);
                        

                        let r0 = self.get_band(0).get(x, y).unwrap() as f64;
                        let g0 = self.get_band(1).get(x, y).unwrap() as f64;
                        let b0 = self.get_band(2).get(x, y).unwrap() as f64;

                        if avg_pixels && (r0 > 0.0 || g0 > 0.0 || b0 > 0.0) {
                            r = (r + r0) / 2.0;
                            g = (g + g0) / 2.0;
                            b = (b + b0) / 2.0;
                        }

                        match eye {
                            Eye::Left => {
                                self.put(x, y, r as f32, 0);
                            },
                            Eye::Right => {
                                self.put(x, y, g as f32, 1);
                                self.put(x, y, b as f32, 2);
                            },
                            Eye::DontCare => {
                                self.put(x, y, r as f32, 0);
                                self.put(x, y, g as f32, 1);
                                self.put(x, y, b as f32, 2);
                            }
                        };
                    }
                }
            }
        }
    }

    /// Paints a square on the image by breaking it into two triangles.
    fn paint_square(&mut self, tl:&Point, bl:&Point, br:&Point, tr:&Point, avg_pixels:bool, eye:Eye) {
        self.paint_tri(&Triangle {
            p0: tl.clone(),
            p1: bl.clone(),
            p2: tr.clone()
        }, avg_pixels, eye);
        self.paint_tri(&Triangle {
            p0: tr.clone(),
            p1: bl.clone(),
            p2: br.clone()
        }, avg_pixels, eye);
    }

    fn to_mono(&mut self) {
        if self.num_bands() != 3 {
            panic!("Cannot convert to mono: Already mono or unsupported number of bands");
        }

        let r = self.get_band(0).scale(0.2125).unwrap();
        let g = self.get_band(1).scale(0.7154).unwrap();
        let b = self.get_band(2).scale(0.0721).unwrap();

        let m = r.add(&g).unwrap().add(&b).unwrap();

        self.set_band(&m, 0);
        self.set_band(&m, 1);
        self.set_band(&m, 2);

    }
}
