use crate::{decompanding::LookUpTable, enums, flatfield, inpaintmask, metadata::*, util};

use anyhow::Result;
use sciimg::{
    debayer::DebayerMethod, drawable::Drawable, enums::ImageMode, image::Image,
    imagebuffer::ImageBuffer, inpaint, path, DnVec, VecMath,
};

#[derive(Clone)]
pub struct MarsImage {
    pub image: Image,
    pub instrument: enums::Instrument,
    pub metadata: Option<Metadata>,
    empty: bool,
    pub file_path: Option<String>,
}

impl MarsImage {
    pub fn new(width: usize, height: usize, instrument: enums::Instrument) -> Self {
        MarsImage {
            image: Image::new_with_bands(width, height, 3, ImageMode::U8BIT).unwrap(),
            instrument,
            metadata: None,
            empty: false,
            file_path: None,
        }
    }

    pub fn from_image(img: &Image, instrument: enums::Instrument) -> Self {
        MarsImage {
            image: img.clone(),
            instrument,
            metadata: None,
            empty: false,
            file_path: None,
        }
    }

    pub fn new_emtpy() -> Self {
        MarsImage {
            image: Image::new_empty().unwrap(),
            instrument: enums::Instrument::None,
            metadata: None,
            empty: true,
            file_path: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn open(file_path: String, instrument: enums::Instrument) -> Self {
        if !path::file_exists(file_path.as_str()) {
            panic!("File not found: {}", file_path);
        }

        vprintln!("Loading image from {}", file_path);

        MarsImage {
            image: Image::open(&file_path).unwrap(),
            instrument,
            metadata: MarsImage::load_image_metadata(&file_path),
            empty: false,
            file_path: Some(file_path),
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

    pub fn save(&self, to_file: &str) -> Result<()> {
        self.image.save(to_file)
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

    pub fn debayer_with_method(&mut self, method: DebayerMethod) {
        vprintln!("Debayering with method: {:?}", method);
        self.image.debayer_with_method(method);

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

    pub fn apply_flat(&mut self, flat: &Image) {
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

    pub fn resize_to(&mut self, to_width: usize, to_height: usize) {
        self.image.resize_to(to_width, to_height);
    }

    pub fn calc_histogram(&self, band: usize) -> DnVec {
        let buffer = self.image.get_band(band);
        let mut hist = DnVec::fill(255, 0.0);
        (0..buffer.buffer.len()).for_each(|i| {
            let r = self.image.get_band(0).buffer[i];
            let g = self.image.get_band(1).buffer[i];
            let b = self.image.get_band(2).buffer[i];
            hist[r as usize] += 1.0;
            hist[g as usize] += 1.0;
            hist[b as usize] += 1.0;
        });
        hist
    }

    fn is_index_a_histogram_gap(hist: &DnVec, index: usize) -> bool {
        // We will define a histogram gap as an index with a zero value that is bounded by
        // non-zero values.
        if index == 0 || index == 254 {
            // So by definition, the zeroth and last index cannot be a gap
            false
        } else {
            hist[index] == 0.0 && hist[index - 1] > 0.0 && hist[index + 1] > 0.0
        }
    }

    fn compute_destretch_lut(&self) -> DnVec {
        let hist = self.calc_histogram(0);
        let mut lut = DnVec::zeros(255);

        let mut value_minus = 0.0;
        (0..255).for_each(|i| {
            if MarsImage::is_index_a_histogram_gap(&hist, i) {
                value_minus += 1.0;
            }
            lut[i] = i as f32 - value_minus;
        });
        lut
    }

    fn destretch_buffer_with_lut(buffer: &ImageBuffer, lut: &DnVec) -> ImageBuffer {
        let mut corrected = buffer.clone();
        (0..corrected.buffer.len()).for_each(|i| {
            corrected.buffer[i] = lut[corrected.buffer[i].round() as usize];
        });
        corrected
    }

    pub fn destretch_image(&mut self) {
        let lut = self.compute_destretch_lut();
        (0..self.image.num_bands()).for_each(|i| {
            self.image.set_band(
                &MarsImage::destretch_buffer_with_lut(self.image.get_band(i), &lut),
                i,
            );
        });
    }
}
