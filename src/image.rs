use crate::{
    decompanding::LookUpTable, drawable::Drawable, enums, flatfield, inpaintmask, metadata::*,
    path, util, vprintln,
};

use sciimg::{enums::ImageMode, imagebuffer::ImageBuffer, inpaint, rgbimage::RgbImage};

#[derive(Clone)]
pub struct MarsImage {
    pub image: RgbImage,
    pub instrument: enums::Instrument,
    pub metadata: Option<Metadata>,
}

impl MarsImage {
    pub fn new(width: usize, height: usize, instrument: enums::Instrument) -> Self {
        MarsImage {
            image: RgbImage::new_with_bands(width, height, 3, ImageMode::U8BIT).unwrap(),
            instrument,
            metadata: None,
        }
    }

    pub fn open(file_path: String, instrument: enums::Instrument) -> Self {
        if !path::file_exists(file_path.as_str()) {
            panic!("File not found: {}", file_path);
        }

        vprintln!("Loading image from {}", file_path);

        MarsImage {
            image: RgbImage::open(&file_path).unwrap(),
            instrument,
            metadata: MarsImage::load_image_metadata(&file_path),
        }
    }

    fn load_image_metadata(file_path: &str) -> Option<Metadata> {
        let metadata_file = util::replace_image_extension(file_path, "-metadata.json");
        vprintln!("Checking for metadata file at {}", metadata_file);
        if path::file_exists(metadata_file.as_str()) {
            vprintln!("Metadata file exists for loaded image: {}", metadata_file);
            match load_image_metadata(&metadata_file) {
                Err(why) => panic!("couldn't open {}", why),
                Ok(md) => Some(md),
            }
        } else {
            None
            //panic!("Metadata file not found: {}", metadata_file);
        }
    }

    pub fn save(&self, to_file: &str) {
        self.image.save(to_file);

        vprintln!("Writing image buffer to file at {}", to_file);
        if path::parent_exists_and_writable(to_file) {
            match &self.metadata {
                Some(md) => {
                    util::save_image_json(to_file, &md, false, None).unwrap();
                }
                None => {}
            };
            vprintln!("File saved.");
        } else {
            panic!(
                "Parent does not exist or cannot be written: {}",
                path::get_parent(to_file)
            );
        }
    }

    pub fn apply_weight(&mut self, r_scalar: f32, g_scalar: f32, b_scalar: f32) {
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

    pub fn decompand(&mut self, ilt: &LookUpTable) {
        self.image.decompand(&ilt.to_array());

        if let Some(ref mut md) = self.metadata {
            md.decompand = true;
        }
    }

    pub fn compand(&mut self, ilt: &LookUpTable) {
        self.image.compand(&ilt.to_array());

        if let Some(ref mut md) = self.metadata {
            md.decompand = false;
        }
    }

    fn apply_flat(&mut self, flat: &RgbImage) {
        self.image.apply_flat(flat);

        if let Some(ref mut md) = self.metadata {
            md.flatfield = true;
        }
    }

    pub fn flatfield_with_flat(&mut self, flat: &MarsImage) {
        self.apply_flat(&flat.image);
    }

    pub fn crop(&mut self, x: usize, y: usize, width: usize, height: usize) {
        self.image.crop(x, y, width, height);
    }

    pub fn flatfield(&mut self) {
        let mut flat = if let Ok(flat) = flatfield::load_flat(self.instrument) {
            flat
        } else {
            vprintln!("No flat field found for instrument {:?}", self.instrument);
            return;
        };

        let subframe_opt = if let Some(md) = &self.metadata {
            md.subframe_rect.clone()
        } else {
            None
        };

        if let Some(sf) = subframe_opt {
            vprintln!(
                "Cropping flat with x/y/width/height: {},{} {}x{}",
                sf[0],
                sf[1],
                sf[2],
                sf[3]
            );

            flat.image.crop(
                sf[0] as usize - 1,
                sf[1] as usize - 1,
                sf[2] as usize,
                sf[3] as usize,
            );
        }

        // If the flat is still too big we'll
        // crop the flatfield image if it's larger than the input image.
        // Sizes need to match
        if flat.image.width > self.image.width {
            let x = (flat.image.width - self.image.width) / 2;
            let y = (flat.image.height - self.image.height) / 2;
            vprintln!(
                "Cropping flat with x/y/width/height: {},{} {}x{}",
                x,
                y,
                self.image.width,
                self.image.height
            );
            flat.image.crop(x, y, self.image.width, self.image.height);
        }

        // if inpaint::inpaint_supported_for_instrument(self.instrument) {
        //     flat.apply_inpaint_fix().unwrap();
        // } else {
        //     vprintln!("No inpaint available for flatfield image on {:?}", self.instrument);
        // }
        self.apply_flat(&flat.image);
    }

    pub fn apply_alpha(&mut self, mask: &ImageBuffer) {
        self.image.copy_alpha_from(mask);
    }

    pub fn clear_alpha(&mut self) {
        self.image.clear_alpha();
    }

    pub fn get_alpha_at(&self, x: usize, y: usize) -> bool {
        self.image.get_alpha_at(x, y)
    }

    pub fn apply_inpaint_fix(&mut self) {
        let mask = inpaintmask::load_mask(self.instrument).unwrap();
        self.apply_inpaint_fix_with_mask(&mask);
    }

    pub fn apply_inpaint_fix_with_mask(&mut self, mask: &ImageBuffer) {
        let mut fixed = inpaint::apply_inpaint_to_buffer(&self.image, mask).unwrap();
        fixed.set_mode(self.image.get_mode());
        self.image = fixed;

        if let Some(ref mut md) = self.metadata {
            md.inpaint = true;
        }
    }

    pub fn hot_pixel_correction(&mut self, window_size: i32, threshold: f32) {
        self.image.hot_pixel_correction(window_size, threshold);
    }

    pub fn to_mono(&mut self) {
        self.image.to_mono();
    }

    pub fn resize_to(&mut self, to_width:usize, to_height:usize) {
        self.image.resize_to(to_width, to_height);
    }
}
