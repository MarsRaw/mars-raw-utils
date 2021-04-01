
use crate::{
    imagebuffer::ImageBuffer, 
    constants, 
    vprintln, 
    path, 
    error, 
    decompanding, 
    enums, 
    flatfield, 
    inpaint,
    ok,
};

use image::{
    open, 
    DynamicImage, 
    Rgb
};

// A simple image raster buffer.
#[derive(Debug, Clone)]
pub struct RgbImage {
    _red: ImageBuffer,
    _green: ImageBuffer,
    _blue: ImageBuffer,
    pub width: usize,
    pub height: usize,
    empty: bool,
}

#[allow(dead_code)]
impl RgbImage {


    pub fn new(width:usize, height:usize) -> error::Result<RgbImage> {
        let red = ImageBuffer::new(width, height).unwrap();
        let green = ImageBuffer::new(width, height).unwrap();
        let blue = ImageBuffer::new(width, height).unwrap();

        Ok(RgbImage{
            _red:red,
            _green:green,
            _blue:blue,
            width:width,
            height:height,
            empty:false
        })
    }

    pub fn open(file_path:&str) -> error::Result<RgbImage> {
        if !path::file_exists(file_path) {
            return Err(constants::status::FILE_NOT_FOUND);
        }

        vprintln!("Loading image from {}", file_path);
        let image_data = open(file_path).unwrap().into_rgb8();
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;
        vprintln!("Input image dimensions: {:?}", image_data.dimensions());

        let mut rgbimage = RgbImage::new(width, height).unwrap();

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let red = pixel[0] as f32;
                let green = pixel[1] as f32;
                let blue = pixel[2] as f32;
                rgbimage.put(x, y, red, green, blue).unwrap();
            }
        }

        Ok(rgbimage)
    }

    pub fn new_empty() -> error::Result<RgbImage> {
        Ok(RgbImage{
            _red:ImageBuffer::new_empty().unwrap(),
            _green:ImageBuffer::new_empty().unwrap(),
            _blue:ImageBuffer::new_empty().unwrap(),
            width:0,
            height:0,
            empty:true
        })
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn put(&mut self, x:usize, y:usize, r:f32, g:f32, b:f32) -> error::Result<&str>{
        if x < self.width && y < self.height {
            self._red.put(x, y, r)?;
            self._green.put(x, y, g)?;
            self._blue.put(x, y, b)?;
            return ok!();
        } else {
            return Err(constants::status::INVALID_PIXEL_COORDINATES);
        }
    }

    pub fn red(&self) -> &ImageBuffer {
        &self._red
    }

    pub fn green(&self) -> &ImageBuffer {
        &self._green
    }

    pub fn blue(&self) -> &ImageBuffer {
        &self._blue
    }

    fn apply_flat_on_channel(buffer:&ImageBuffer, flat_buffer:&ImageBuffer) -> error::Result<ImageBuffer> {
        let mean_flat = flat_buffer.mean();
        let corrected = buffer.scale(mean_flat).unwrap().divide(&flat_buffer).unwrap();
        Ok(corrected)
    }

    fn apply_flat(&mut self, flat:RgbImage) -> error::Result<&str> {

        self._red = RgbImage::apply_flat_on_channel(&self._red, &flat.red()).unwrap();
        self._green = RgbImage::apply_flat_on_channel(&self._green, &flat.green()).unwrap();
        self._blue = RgbImage::apply_flat_on_channel(&self._blue, &flat.blue()).unwrap();

        ok!()
    }

    pub fn flatfield(&mut self, instrument:enums::Instrument) -> error::Result<&str> {

        let mut flat = flatfield::load_flat(instrument).unwrap();
        if flat.width == 1632 && flat.height == 1200 {
            flat.crop(32, 16, 1584, 1184).unwrap();
        }
        flat.apply_inpaint_fix(instrument).unwrap();
        self.apply_flat(flat).unwrap();
        ok!()
    }

    pub fn decompand(&mut self, instrument:enums::Instrument) -> error::Result<&str> {
        decompanding::decompand_buffer(&mut self._red, instrument).unwrap();
        decompanding::decompand_buffer(&mut self._green, instrument).unwrap();
        decompanding::decompand_buffer(&mut self._blue, instrument).unwrap();

        ok!()
    }

    pub fn apply_weight(&mut self, r_scalar:f32, g_scalar:f32, b_scalar:f32) -> error::Result<&str> {

        self._red = self._red.scale(r_scalar).unwrap();
        self._green = self._green.scale(g_scalar).unwrap();
        self._blue = self._blue.scale(b_scalar).unwrap();

        ok!()
    }


    pub fn crop(&mut self, x:usize, y:usize, width:usize, height:usize) -> error::Result<&str> {
        self._red = self._red.get_subframe(x, y, width, height).unwrap();
        self._green = self._green.get_subframe(x, y, width, height).unwrap();
        self._blue = self._blue.get_subframe(x, y, width, height).unwrap();
        ok!()
    }

    pub fn apply_inpaint_fix(&mut self, instrument:enums::Instrument) -> error::Result<&str> {
        self._red = inpaint::apply_inpaint_to_buffer(&self._red, instrument).unwrap();
        self._green = inpaint::apply_inpaint_to_buffer(&self._green, instrument).unwrap();
        self._blue = inpaint::apply_inpaint_to_buffer(&self._blue, instrument).unwrap();
        ok!()
    }

    pub fn normalize_to_16bit_with_max(&mut self, max:f32) -> error::Result<&str> {
        self._red = self._red.normalize_force_minmax(0.0, 65535.0, 0.0, max).unwrap();
        self._green = self._green.normalize_force_minmax(0.0, 65535.0, 0.0, max).unwrap();
        self._blue = self._blue.normalize_force_minmax(0.0, 65535.0, 0.0, max).unwrap();
        ok!()
    }

    pub fn normalize_to_16bit(&mut self) -> error::Result<&str> {

        let r_mnmx = self._red.get_min_max().unwrap();
        let g_mnmx = self._green.get_min_max().unwrap();
        let b_mnmx = self._blue.get_min_max().unwrap();

        let mut mx = if r_mnmx.max > g_mnmx.max { r_mnmx.max} else { g_mnmx.max };
        mx = if mx > b_mnmx.max { mx } else { b_mnmx.max };

        self.normalize_to_16bit_with_max(mx).unwrap();

        ok!()
    }

    pub fn normalize_8bit_to_16bit(&mut self) -> error::Result<&str> {
        self.normalize_to_16bit_with_max(255.0).unwrap();
        ok!()
    }

    pub fn save(&self, to_file:&str) -> error::Result<&str> {
        let mut out_img = DynamicImage::new_rgb16(self.width as u32, self.height as u32).into_rgb16();

        for y in 0..self.height {
            for x in 0..self.width {
                let r = self._red.get(x, y).unwrap().round() as u16;
                let g = self._green.get(x, y).unwrap().round() as u16;
                let b = self._blue.get(x, y).unwrap().round() as u16;
                out_img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
            }
        }

        vprintln!("Writing image buffer to file at {}", to_file);
        if path::parent_exists_and_writable(&to_file) {
            out_img.save(to_file).unwrap();
            vprintln!("    File saved.");
            return ok!();
        } else {
            eprintln!("Parent does not exist or cannot be written: {}", path::get_parent(to_file));
            return Err(constants::status::PARENT_NOT_EXISTS_OR_UNWRITABLE);
        }
    }
}