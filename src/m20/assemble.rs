use crate::prelude::*;
use sciimg::{enums::ImageMode, rgbimage};

lazy_static! {
    static ref SUBFRAME_IDS_SCALE_FACTOR_1: Vec<Vec<usize>> = vec![
        vec![1, 5, 9, 13],
        vec![2, 6, 10, 14],
        vec![3, 7, 11, 15],
        vec![4, 8, 12, 16],
    ];
}

lazy_static! {
    static ref SUBFRAME_IDS_SCALE_FACTOR_2: Vec<Vec<usize>> = vec![vec![1, 4], vec![7, 10]];
}

lazy_static! {
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
    pub static ref FRAME_MATCH_PAIRS_SCALEFACTOR_2: Vec<Vec<usize>> =
        vec![vec![1, 4], vec![1, 7], vec![4, 10]];
}

pub struct NavcamTile {
    pub image: MarsImage,
}

pub struct TileCoordinates {
    pub top_left_x: usize,
    pub top_left_y: usize,
    pub bottom_right_x: usize,
    pub bottom_right_y: usize,
    pub scale: u32,
}

impl NavcamTile {
    pub fn new_from_file(file_path: &String, instrument: Instrument) -> Self {
        NavcamTile {
            image: MarsImage::open(String::from(file_path), instrument),
        }
    }

    pub fn new_with_image(image: &MarsImage) -> Self {
        NavcamTile {
            image: image.clone(),
        }
    }

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

    pub fn get_tile_id(&self) -> usize {
        match self.get_scale_factor() {
            1 => self.get_tile_id_scale_factor_1(),
            2 => self.get_tile_id_scale_factor_2(),
            _ => panic!("Error: Cannot determine tile id: Unsupported scale factor"),
        }
    }
    pub fn get_tile_id_scale_factor_1(&self) -> usize {
        if self.get_scale_factor() != 1 {
            panic!("Cannot determine scale factor 1 tile id on a scale factor != 1 image");
        }
        let sf = self.get_subframe_region();
        let x_frac = (sf[0] / 5121.0 * 4.0).round() as usize;
        let y_frac = (sf[1] / 3841.0 * 4.0).round() as usize;
        SUBFRAME_IDS_SCALE_FACTOR_1[y_frac][x_frac]
    }

    pub fn get_tile_id_scale_factor_2(&self) -> usize {
        if self.get_scale_factor() != 2 {
            panic!("Cannot determine scale factor 2 tile id on a scale factor != 2 image");
        }
        let sf = self.get_subframe_region();
        let x_frac = (sf[0] / 2560.0).round() as usize;
        let y_frac = (sf[1] / 1920.0).round() as usize;
        SUBFRAME_IDS_SCALE_FACTOR_2[y_frac][x_frac]
    }

    pub fn get_scale_factor(&self) -> u32 {
        if let Some(md) = &self.image.metadata {
            md.scale_factor
        } else {
            1
        }
    }
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

pub struct Composite {
    pub scale: u32,
    pub width: usize,
    pub height: usize,
    composite_image: rgbimage::RgbImage,
}

impl Composite {
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

    fn determine_composite_size(tiles: &[NavcamTile]) -> (usize, usize) {
        let mut max_x: usize = 0;
        let mut max_y: usize = 0;

        tiles.iter().for_each(|t| {
            if let Some(tc) = t.get_tile_coordinates() {
                max_x = if tc.bottom_right_x > max_x {
                    tc.bottom_right_x
                } else {
                    max_x
                };
                max_y = if tc.bottom_right_y > max_y {
                    tc.bottom_right_y
                } else {
                    max_y
                };
            }
        });

        (max_x, max_y)
    }

    pub fn paste_tiles(&mut self, tiles: &[NavcamTile]) {
        tiles.iter().for_each(|t| {
            self.paste_tile(t);
        });
    }

    pub fn paste_tile(&mut self, tile: &NavcamTile) {
        if let Some(tilecoord) = tile.get_tile_coordinates() {
            self.composite_image.paste(
                &tile.image.image,
                tilecoord.top_left_x,
                tilecoord.top_left_y,
            );
        }
    }

    pub fn finalize_and_save(&mut self, output_path: &str) {
        self.composite_image.normalize_to_16bit_with_max(255.0);
        self.composite_image.save(output_path);
    }
}
