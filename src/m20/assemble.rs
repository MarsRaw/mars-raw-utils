use crate::prelude::*;
use sciimg::{enums::ImageMode, rgbimage};

lazy_static! {
    /// Matrix of expected tile ids in a scale factor 1 navcam image. These do not
    /// appear to be consistent in the public raw images, so I will standardize them
    /// here and determine them by their subframe coordinates
    static ref SUBFRAME_IDS_SCALE_FACTOR_1: Vec<Vec<usize>> = vec![
        vec![1, 5, 9, 13],
        vec![2, 6, 10, 14],
        vec![3, 7, 11, 15],
        vec![4, 8, 12, 16],
    ];
}

lazy_static! {
    /// Matrix of expected tile ids in a scale factor 2 navcam image. These do not
    /// appear to be consistent in the public raw images, so I will standardize them
    /// here and determine them by their subframe coordinates
    static ref SUBFRAME_IDS_SCALE_FACTOR_2: Vec<Vec<usize>> = vec![vec![1, 4], vec![7, 10]];
}

lazy_static! {
    /// This sets a series and ordering of tile ids that need color matching, starting
    /// with tile id 1 (top left). This can be optimized.
    pub static ref FRAME_MATCH_PAIRS_SCALEFACTOR_1: Vec<Vec<usize>> = vec![
        vec![1, 5],
        vec![5, 9],
        vec![9, 13],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![5, 6],
        vec![6, 7],
        vec![7, 8],
        vec![9, 10],
        vec![10, 11],
        vec![11, 12],
        vec![13, 14],
        vec![14, 15],
        vec![15, 16],
    ];
}

lazy_static! {
    // This sets a series and ordering of tile ids that need color matching, starting
    // with tile id 1 (top left). This can be optimized.
    pub static ref FRAME_MATCH_PAIRS_SCALEFACTOR_2: Vec<Vec<usize>> =
        vec![vec![1, 4], vec![1, 7], vec![4, 10]];
}

/// Defines an extension of `MarsImage` to support NavCam reassembly
pub struct NavcamTile {
    pub image: MarsImage,
}

/// Container for tile coordinates and scale
pub struct TileCoordinates {
    pub top_left_x: usize,
    pub top_left_y: usize,
    pub bottom_right_x: usize,
    pub bottom_right_y: usize,
    pub scale: u32,
}

impl NavcamTile {
    /// Constructs a new `NavcamTile` by opening the image and assigning an instrument.
    ///
    /// # Examples
    ///
    /// ```
    /// use mars_raw_utils::enums::Instrument;
    /// use mars_raw_utils::m20::assemble::NavcamTile;
    ///
    /// NavcamTile::new_from_file(&String::from("tests/testdata/NLF_0670_0726421423_362ECM_N0320604NCAM08111_01_095J01.png"), Instrument::M20NavcamRight);
    /// ```
    pub fn new_from_file(file_path: &String, instrument: Instrument) -> Self {
        NavcamTile {
            image: MarsImage::open(String::from(file_path), instrument),
        }
    }

    /// Constructs a new `NavcamTile` with an existing instance of `MarsImage`.
    pub fn new_with_image(image: &MarsImage) -> Self {
        NavcamTile {
            image: image.clone(),
        }
    }

    /// Determines tile coordinates from metadata `scale` and `subframe_region` fields
    pub fn get_tile_coordinates(&self) -> Option<TileCoordinates> {
        let sf = self.get_subframe_region();
        let scale = self.get_scale_factor();

        if sf.len() != 4 {
            return None;
        }
        let tl_x = (sf[0] / scale as f64).floor() as usize + 2;
        let tl_y = (sf[1] / scale as f64).floor() as usize + 2;
        let right_x = tl_x + self.image.image.width;
        let bottom_y = tl_y + self.image.image.height;

        Some(TileCoordinates {
            top_left_x: tl_x,
            top_left_y: tl_y,
            bottom_right_x: right_x,
            bottom_right_y: bottom_y,
            scale,
        })
    }

    /// Proxies to scale factor specific `get_tile_id...` functions
    pub fn get_tile_id(&self) -> usize {
        match self.get_scale_factor() {
            1 => self.get_tile_id_scale_factor_1(),
            2 => self.get_tile_id_scale_factor_2(),
            _ => panic!("Error: Cannot determine tile id: Unsupported scale factor"),
        }
    }

    /// Determines standardized tile id for scale factor 1 via coordinates
    /// in metadata `subframe_region` field
    pub fn get_tile_id_scale_factor_1(&self) -> usize {
        if self.get_scale_factor() != 1 {
            panic!("Cannot determine scale factor 1 tile id on a scale factor != 1 image");
        }
        let sf = self.get_subframe_region();
        let x_frac = (sf[0] / 5121.0 * 4.0).round() as usize;
        let y_frac = (sf[1] / 3841.0 * 4.0).round() as usize;
        SUBFRAME_IDS_SCALE_FACTOR_1[y_frac][x_frac]
    }

    /// Determines standardized tile id for scale factor 2 via coordinates
    /// in metadata `subframe_region` field
    pub fn get_tile_id_scale_factor_2(&self) -> usize {
        if self.get_scale_factor() != 2 {
            panic!("Cannot determine scale factor 2 tile id on a scale factor != 2 image");
        }
        let sf = self.get_subframe_region();
        let x_frac = (sf[0] / 2560.0).round() as usize;
        let y_frac = (sf[1] / 1920.0).round() as usize;
        SUBFRAME_IDS_SCALE_FACTOR_2[y_frac][x_frac]
    }

    /// Indicates whether the image is of a scale factor supported by this
    /// module
    pub fn is_supported_scale_factor(&self) -> bool {
        self.get_scale_factor() <= 2
    }

    /// Returns the scale factor in the metadata `scale_factor` field
    pub fn get_scale_factor(&self) -> u32 {
        if let Some(md) = &self.image.metadata {
            md.scale_factor
        } else {
            1
        }
    }

    // Returns the subframe coordinates from the metadata `subframe_rect` field
    pub fn get_subframe_region(&self) -> Vec<f64> {
        if let Some(md) = &self.image.metadata {
            if let Some(sf) = &md.subframe_rect {
                sf.clone()
            } else {
                vec![0.0]
            }
        } else {
            vec![0.0]
        }
    }
}

/// Implements a composite canvas image that will recieve the navcam tiles and
/// eventually saved to disk
pub struct Composite {
    pub scale: u32,
    pub width: usize,
    pub height: usize,
    composite_image: rgbimage::RgbImage,
}

impl Composite {
    /// Constructs a new `Composite` instance, creating an image canvas of the size indicated by
    /// the tiles scale factor and coordinates
    pub fn new(tiles: &Vec<NavcamTile>) -> Self {
        if tiles.is_empty() {
            panic!("Cannot assemble composite with no tiles!");
        }

        let scale = tiles[0].get_scale_factor();

        let (max_x, max_y) = Composite::determine_composite_size(tiles);

        let composite_image =
            rgbimage::RgbImage::new_with_bands(max_x, max_y, 3, ImageMode::U16BIT).unwrap();

        vprintln!(
            "Composite has width {}px, height {}px, and scale factor of {}",
            max_x,
            max_y,
            scale
        );

        Composite {
            scale,
            width: max_x,
            height: max_y,
            composite_image,
        }
    }

    /// Crops the canvas using the specified dimensions.
    pub fn crop(&mut self, x: usize, y: usize, width: usize, height: usize) {
        self.composite_image.crop(x, y, width, height);
        self.width = width;
        self.height = height;
    }

    /// Returns standard full-frame composite sizes based on the tiles'
    /// scale factor
    fn determine_composite_size(tiles: &[NavcamTile]) -> (usize, usize) {
        match tiles[0].get_scale_factor() {
            1 => (5120, 3840),
            2 => (2560, 1920),
            4 => (1280, 960),
            _ => panic!("Unsupported scale factor"),
        }
    }

    /// Pastes a set of `NavcamTile`s onto the canvas.
    pub fn paste_tiles(&mut self, tiles: &[NavcamTile]) {
        tiles.iter().for_each(|t| {
            self.paste_tile(t);
        });
    }

    /// Pastes a `NavcamTile` onto the canvas. Will also attempt to crop tile edges
    /// to avoid compression artifacts and telemetry pixels
    pub fn paste_tile(&mut self, tile: &NavcamTile) {
        if let Some(tilecoord) = tile.get_tile_coordinates() {
            let mut paste_image = tile.image.image.clone();
            paste_image.crop(2, 2, paste_image.width - 2, paste_image.height - 2);

            self.composite_image.paste(
                &paste_image,
                tilecoord.top_left_x - 1,
                tilecoord.top_left_y - 1,
            );
        }
    }

    /// Normalize the canvas to 16 bit value range and save to disk.
    pub fn finalize_and_save(&mut self, output_path: &str) {
        self.composite_image.normalize_to_16bit_with_max(255.0);
        self.composite_image.save(output_path);
    }
}
