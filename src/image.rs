

use crate::{
    enums,
    metadata::*,
    util,
    path,
    vprintln,
    flatfield, 
    decompanding,
    inpaintmask
};

use sciimg::{
    rgbimage::RgbImage,
    enums::ImageMode,
    imagebuffer::ImageBuffer,
    inpaint
};


#[derive(Debug, Clone)]
pub struct MarsImage {
    pub image: RgbImage,
    instrument: enums::Instrument,
    metadata: Option<Metadata>
}

impl MarsImage {

    pub fn new(width:usize, height:usize, instrument:enums::Instrument) -> Self {
        MarsImage {
            image:RgbImage::new_with_bands(width, height, 3, ImageMode::U8BIT).unwrap(),
            instrument:instrument,
            metadata:None
        }
    }

    pub fn open(file_path:String, instrument:enums::Instrument) -> Self {
        if !path::file_exists(file_path.as_str()) {
            panic!("File not found: {}",  file_path);
        }

        vprintln!("Loading image from {}", file_path);
        
        MarsImage {
            image:RgbImage::open(&file_path).unwrap(),
            instrument:instrument,
            metadata:MarsImage::load_image_metadata(&file_path)
        }

    }

    fn load_image_metadata(file_path:&str) -> Option<Metadata> {
        let metadata_file = util::replace_image_extension(&file_path, "-metadata.json");
        vprintln!("Checking for metadata file at {}", metadata_file);
        if path::file_exists(metadata_file.as_str()) {
            vprintln!("Metadata file exists for loaded image: {}", metadata_file);
            match load_image_metadata(&metadata_file) {
                Err(why) => panic!("couldn't open {}", why),
                Ok(md) => Some(md)
            }
        } else {
            None
            //panic!("Metadata file not found: {}", metadata_file);
        }
    }


    pub fn save(&self, to_file:&str) {
        self.image.save(to_file);
        
        vprintln!("Writing image buffer to file at {}", to_file);
        if path::parent_exists_and_writable(&to_file) {
            match &self.metadata {
                Some(md) => {
                    util::save_image_json(to_file, &md, false).unwrap();
                },
                None => {}
            };
            vprintln!("File saved.");
        } else {
            panic!("Parent does not exist or cannot be written: {}", path::get_parent(to_file));
        }
    }



    pub fn apply_weight(&mut self, r_scalar:f32, g_scalar:f32, b_scalar:f32) {

        self.image.apply_weight_on_band(r_scalar, 0);
        self.image.apply_weight_on_band(g_scalar, 1);
        self.image.apply_weight_on_band(b_scalar, 2);

        if let Some(ref mut md) = self.metadata {
            md.radiometric = true;
        }
    }


    pub fn debayer(&mut self) {
        self.image.debayer();

        if let Some(ref mut md) = self.metadata {
            md.debayer = true;
        }
    }

    pub fn decompand(&mut self, ilt:&[u32; 256]) {
        self.image.decompand(ilt);

        if let Some(ref mut md) = self.metadata {
            md.decompand = true;
        }
    }

    pub fn compand(&mut self, ilt:&[u32; 256]) {
        self.image.compand(ilt);

        if let Some(ref mut md) = self.metadata {
            md.decompand = false;
        }
    }


    fn apply_flat(&mut self, flat:&RgbImage)  {
        self.image.apply_flat(&flat);

        if let Some(ref mut md) = self.metadata {
            md.flatfield = true;
        }
    }


    pub fn flatfield_with_flat(&mut self, flat:&MarsImage) {

        self.apply_flat(&flat.image);

    }

    pub fn flatfield(&mut self) {

        let mut flat = flatfield::load_flat(self.instrument).unwrap();

        // These instrument-specific crops don't really belong in here.
        if self.instrument == enums::Instrument::MslMAHLI && flat.image.width == 1632 && flat.image.height == 1200 {
            flat.image.crop(32, 16, 1584, 1184);
        } 
        
        if self.instrument == enums::Instrument::MslMastcamRight {

            if self.image.width == 1328 && self.image.height == 1184 {
                //x160, y16
                flat.image.crop(160, 16, 1328, 1184);
            } else if self.image.width == 848 && self.image.height == 848 {
                //x400, y192
                flat.image.crop(400, 192, 848, 848);
            }

            if self.image.get_mode() == ImageMode::U8BIT {
                flat.image.normalize_to_12bit_with_max(decompanding::get_max_for_instrument(self.instrument) as f32, 255.0);
                flat.compand(&decompanding::get_ilt_for_instrument(self.instrument));
            }

        }

        if self.instrument == enums::Instrument::MslMastcamLeft {

            if self.image.width == 1328 && self.image.height == 1184 { //9
                flat.image.crop(160, 16, 1328, 1184);
            }  else if self.image.width == 1152 && self.image.height == 432 {
                flat.image.crop(305, 385, 1152, 432);
            }

            if self.image.get_mode() == ImageMode::U8BIT {
                flat.image.normalize_to_12bit_with_max(decompanding::get_max_for_instrument(self.instrument) as f32, 255.0);
                flat.compand(&decompanding::get_ilt_for_instrument(self.instrument));
            }
        }
        

        // Crop the flatfield image if it's larger than the input image. 
        // Sizes need to match
        if flat.image.width > self.image.width {
            let x = (flat.image.width - self.image.width) / 2;
            let y = (flat.image.height - self.image.height) / 2;
            vprintln!("Cropping flat with x/y/width/height: {},{} {}x{}", x, y, self.image.width, self.image.height);
            flat.image.crop(x, y, self.image.width, self.image.height);
        }

        // if inpaint::inpaint_supported_for_instrument(self.instrument) {
        //     flat.apply_inpaint_fix().unwrap();
        // } else {
        //     vprintln!("No inpaint available for flatfield image on {:?}", self.instrument);
        // }
        self.apply_flat(&flat.image);
    }

    pub fn apply_mask(&mut self, mask:&ImageBuffer) {
        self.image.apply_mask_to_band(&mask, 0);
        self.image.apply_mask_to_band(&mask, 1);
        self.image.apply_mask_to_band(&mask, 2);
    }

    pub fn clear_mask(&mut self) {
        self.image.clear_mask_on_band(0);
        self.image.clear_mask_on_band(1);
        self.image.clear_mask_on_band(2);
    }


    pub fn apply_inpaint_fix(&mut self) {

        let mask = inpaintmask::load_mask(self.instrument).unwrap();
        let mut fixed = inpaint::apply_inpaint_to_buffer(&self.image, &mask).unwrap();
        fixed.set_mode(self.image.get_mode());
        self.image = fixed;

        if let Some(ref mut md) = self.metadata {
            md.inpaint = true;
        }
    }

    pub fn hot_pixel_correction(&mut self, window_size:i32, threshold:f32) {
        self.image.hot_pixel_correction(window_size, threshold);
    }
}