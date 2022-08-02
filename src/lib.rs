
extern crate clap;

#[macro_use]
extern crate lazy_static;

pub mod print;
pub mod m20;
pub mod msl;
pub mod nsyt;
pub mod mer;
pub mod constants;
pub mod calibfile;
pub mod path;
pub mod decompanding;
pub mod inpaintmask;
pub mod enums;
pub mod flatfield;
pub mod util;
pub mod httpfetch;
pub mod jsonfetch;
pub mod time;
pub mod metadata;
pub mod image;
pub mod diffgif;
pub mod calprofile;
pub mod focusmerge;
pub mod calibrate;
pub mod drawable;
pub mod anaglyph;
pub mod composite;
pub mod vecmath;
pub mod prelude;


use sciimg::imagebuffer::ImageBuffer;

trait Isolate {
    fn isolate_window(&self, window_size:usize, x:usize, y:usize) -> Vec<f32>;
}

impl Isolate for ImageBuffer {
    fn isolate_window(&self, window_size:usize, x:usize, y:usize) -> Vec<f32> {
        let mut v:Vec<f32> = Vec::with_capacity(window_size * window_size);
        let start = window_size as i32 / 2 * -1;
        let end = window_size as i32 / 2 + 1;
        for _y in start..end as i32 {
            for _x in start..end as i32 {
                let get_x = x as i32 + _x;
                let get_y = y as i32 + _y;
                if get_x >= 0 
                    && get_x < self.width as i32 
                    && get_y >= 0 
                    && get_y < self.height as i32
                    && self.get_mask_at_point(get_x as usize, get_y as usize).unwrap()
                    {
                    v.push(self.get(get_x as usize, get_y as usize).unwrap());
                }
            }
        }
        v
    }
}