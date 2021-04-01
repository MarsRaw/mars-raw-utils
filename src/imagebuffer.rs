
use crate::{path, constants, vprintln, error, ok};

extern crate image;
use image::{open, DynamicImage, Rgb};

// A simple image raster buffer.
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    pub buffer: Vec<f32>,
    pub width: usize,
    pub height: usize,
    empty: bool,
}

pub struct Offset {
    pub h: i32,
    pub v: i32,
}

pub struct MinMax {
    pub min: f32,
    pub max: f32,
}

#[allow(dead_code)]
impl ImageBuffer {

    // Creates a new image buffer of the requested width and height
    pub fn new(width:usize, height:usize) -> error::Result<ImageBuffer> {

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        Ok(ImageBuffer{buffer:v,
            width:width,
            height:height,
            empty:false
        })
    }

    pub fn new_empty() -> error::Result<ImageBuffer> {
        Ok(ImageBuffer{buffer:Vec::new(),
            width:0,
            height:0,
            empty:true
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec(v:Vec<f32>, width:usize, height:usize) -> error::Result<ImageBuffer> {

        if v.len() != (width * height) {
            return Err(constants::status::DIMENSIONS_DO_NOT_MATCH_VECTOR_LENGTH);
        }

        Ok(ImageBuffer{buffer:v,
                    width:width,
                    height:height,
                    empty:false
        })
    }

    pub fn from_file(file_path:&str) -> error::Result<ImageBuffer> {

        if !path::file_exists(file_path) {
            return Err(constants::status::FILE_NOT_FOUND);
        }

        let image_data = open(file_path).unwrap().into_luma16();
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;
        vprintln!("Input image dimensions: {:?}", image_data.dimensions());

        
        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let value = pixel[0] as f32;
                let idx = y * width + x;
                v[idx] = value;
                //v.push(value);
            }
        }

        ImageBuffer::from_vec(v, width, height)
    }

    pub fn get_slice(&self, top_y:usize, len:usize) -> error::Result<ImageBuffer> {
        let start_index = top_y * self.width;
        let stop_index = (top_y + len) * self.width;

        let slice = self.buffer[start_index..stop_index].to_vec();

        ImageBuffer::from_vec(slice, self.width, len)
    }

    pub fn get_subframe(&self, left_x:usize, top_y:usize, width:usize, height:usize) -> error::Result<ImageBuffer> {

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        for y in 0..height {
            for x in 0..width {
                let from_idx = (top_y + y) * self.width + (left_x + x);
                let put_idx = y * width + x;
                
                v[put_idx] = self.buffer[from_idx];
            }
        }
        Ok(ImageBuffer::from_vec(v, width, height).unwrap())
    }

    pub fn get(&self, x:usize, y:usize) -> error::Result<f32> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            return Ok(self.buffer[index]);
        } else {
            return Err(constants::status::INVALID_PIXEL_COORDINATES); // TODO: learn to throw exceptions
        }
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn put_u16(&mut self, x:usize, y:usize, val:u16) -> error::Result<&str> {
        self.put(x, y, val as f32)
    }

    pub fn put(&mut self, x:usize, y:usize, val:f32) -> error::Result<&str>{
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.buffer[index] = val;
            return ok!();
        } else {
            return Err(constants::status::INVALID_PIXEL_COORDINATES);
        }
    }

    // Computes the mean of all pixel values
    pub fn mean(&self) -> f32 {

        let mut total:f32 = 0.0;
        let mut count:f32 = 0.0;

        // It is *soooo* inefficient to keep doing this...
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel_value = self.get(x, y).unwrap();
                if pixel_value > 0.0 {
                    total = total + pixel_value;
                    count = count + 1.0;
                }
            }
        }

        return total / count;
    }

    pub fn divide(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            let quotient = if other.buffer[i] != 0.0 { self.buffer[i] / other.buffer[i] } else { 0.0 };
            v[i] = quotient;
        }

        ImageBuffer::from_vec(v, self.width, self.height)
    }

    pub fn divide_into(&self, divisor:f32) -> error::Result<ImageBuffer> {
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            let quotient = if self.buffer[i] != 0.0 { divisor / self.buffer[i] } else { 0.0 };
            v[i] = quotient;
        }

        ImageBuffer::from_vec(v, self.width, self.height)
    }

    pub fn scale(&self, scalar:f32) -> error::Result<ImageBuffer> {
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            let product = self.buffer[i] * scalar;
            v[i] = product;
        }

        ImageBuffer::from_vec(v, self.width, self.height)
    }

    pub fn multiply(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            let product = self.buffer[i] * other.buffer[i];
            v[i] = product;
        }

        ImageBuffer::from_vec(v, self.width, self.height)
    }

    pub fn add(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            let result = self.buffer[i] + other.buffer[i];
            v[i] = result;
        }

        ImageBuffer::from_vec(v, self.width, self.height)
    }

    pub fn subtract(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            return Err(constants::status::ARRAY_SIZE_MISMATCH);
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            let difference = self.buffer[i] - other.buffer[i];
            v[i] = difference;
        }

        ImageBuffer::from_vec(v, self.width, self.height)
    }


    pub fn shift_to_min_zero(&self) -> error::Result<ImageBuffer> {

        let minmax = self.get_min_max().unwrap();

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            let value = self.buffer[i];
            if minmax.min < 0.0 {
                v[i] = value + minmax.min;
            } else {
                v[i] = value - minmax.min;
            }
        }

        Ok(ImageBuffer::from_vec(v, self.width, self.height).unwrap())
    }

    // I suspect there's an error here...
    pub fn normalize_force_minmax(&self, min:f32, max:f32, forced_min:f32, forced_max:f32) -> error::Result<ImageBuffer> {
        let shifted = self.shift_to_min_zero().unwrap();

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            let value = ((shifted.buffer[i] - forced_min) / (forced_max- forced_min)) * (max - min) + min;
            v[i] = value;
        }

        Ok(ImageBuffer::from_vec(v, self.width, self.height).unwrap())
    }

    pub fn normalize(&self, min:f32, max:f32) -> error::Result<ImageBuffer> {
        let minmax = self.get_min_max().unwrap();
        self.normalize_force_minmax(min, max, minmax.min, minmax.max)
    }


    pub fn crop(&self, height:usize, width:usize) -> error::Result<ImageBuffer> {

        let mut cropped_buffer = ImageBuffer::new(width, height).unwrap();

        for y in 0..height {
            for x in 0..width {

                let src_x = ((self.width - width) / 2) + x;
                let src_y = ((self.height - height) / 2) + y;

                cropped_buffer.put(x, y, self.get(src_x, src_y).unwrap()).unwrap();
            }
        }

        return Ok(cropped_buffer)
    }

    pub fn shift(&self, horiz:i32, vert:i32) -> error::Result<ImageBuffer> {

        let mut shifted_buffer = ImageBuffer::new(self.width, self.height).unwrap();

        let h = self.height as i32;
        let w = self.width as i32;

        for y in 0..h {
            for x in 0..w {
                let shift_x = x as i32 + horiz;
                let shift_y = y as i32 + vert;
            
                if shift_x >= 0 && shift_y >= 0 && shift_x < w  && shift_y < h {
                    shifted_buffer.put(shift_x as usize, shift_y as usize, self.get(x as usize, y as usize).unwrap()).unwrap();
                }
            }
        }
        return Ok(shifted_buffer)
    }
    // Determined the minimum and maximum values within the 
    // red pixel channel.
    pub fn get_min_max(&self) -> error::Result<MinMax> {
        
        let mut mx:f32 = std::f32::MIN;
        let mut mn:f32 = std::f32::MAX;

        for y in 0..self.height {
            for x in 0..self.width {
                let val = self.get(x, y).unwrap() as f32;
                mx = if val > mx { val } else { mx };
                mn = if val < mn { val } else { mn };
            }
        }
        
        Ok(MinMax{min:mn, max:mx})
    }

    pub fn save(&self, to_file:&str) -> error::Result<&str> {
        let mut out_img = DynamicImage::new_rgb16(self.width as u32, self.height as u32).into_rgb16();
        
        for y in 0..self.height {
            for x in 0..self.width {
                let val = self.get(x, y).unwrap().round() as u16;
                out_img.put_pixel(x as u32, y as u32, Rgb([val, val, val]));
            }
        }

        vprintln!("Writing image buffer to file at {}", to_file);
        if path::parent_exists_and_writable(&to_file) {
            out_img.save(to_file).unwrap();
            vprintln!("File saved.");
            return ok!();
        } else {
            eprintln!("Parent does not exist or cannot be written: {}", path::get_parent(to_file));
            return Err(constants::status::PARENT_NOT_EXISTS_OR_UNWRITABLE);
        }
    
    }

}

