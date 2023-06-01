use crate::veprintln;
use crate::vprintln;
use sciimg::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use anyhow::Result;

lazy_static! {
    static ref IMAGE_CACHE: Arc<Mutex<ImageCache<Image>>> =
        Arc::new(Mutex::new(ImageCache::default()));
    static ref IMAGEBUFFER_CACHE: Arc<Mutex<ImageCache<ImageBuffer>>> =
        Arc::new(Mutex::new(ImageCache::default()));
    static ref TEXT_CACHE: Arc<Mutex<ImageCache<String>>> =
        Arc::new(Mutex::new(ImageCache::default()));
}

/// A *very simple* in-memory cache. Wrapped in a mutex for some semblance of thread
/// safety. Returns copies which causes some memcopy overhead.
struct ImageCache<T: Clone> {
    cache: HashMap<String, T>,
}

impl<T: Clone> ImageCache<T> {
    pub fn default() -> Self {
        ImageCache {
            cache: HashMap::new(),
        }
    }

    pub fn cache_has_file(&self, file_path: &str) -> bool {
        self.cache.contains_key(file_path)
    }

    pub fn load_file<F: Fn(&str) -> Result<T>>(
        &mut self,
        file_path: &str,
        load_file: F,
    ) -> Result<T> {
        if self.cache_has_file(file_path) {
            match self.cache.get(file_path) {
                Some(img) => {
                    vprintln!("File found in cache: {}", file_path);
                    Ok(img.clone())
                }
                None => Err(anyhow!("file not in cache")),
            }
        } else {
            let img_res = load_file(file_path);
            match img_res {
                Ok(img) => {
                    vprintln!("Adding file to calibration cache: {}", file_path);
                    self.cache.insert(file_path.into(), img.clone());
                    vprintln!("File inserted into cache");
                    Ok(img)
                }
                Err(why) => panic!("Failed to load image from {}: {:?}", file_path, why),
            }
        }
    }
}

pub fn load_image(file_path: &str) -> Result<Image> {
    IMAGE_CACHE
        .lock()
        .unwrap()
        .load_file(file_path, Image::open_str)
}

pub fn load_imagebuffer(file_path: &str) -> Result<ImageBuffer> {
    IMAGEBUFFER_CACHE
        .lock()
        .unwrap()
        .load_file(file_path, ImageBuffer::from_file)
}

pub fn load_text_file(file_path: &str) -> Result<String> {
    TEXT_CACHE.lock().unwrap().load_file(file_path, |fp| {
        vprintln!("Loading text file from {}", fp);
        match fs::read_to_string(fp) {
            Ok(s) => {
                vprintln!("File of {} length read successfully", s.len());
                Ok(s)
            }
            Err(why) => {
                veprintln!("Error reading file: {:?}", why);
                Err(anyhow!(why))
            }
        }
    })
}
