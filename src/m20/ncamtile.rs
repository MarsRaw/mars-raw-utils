use crate::image::MarsImage;

use lazy_static;
use sciimg::{error::Result, prelude::ImageBuffer};

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
    static ref DOWNSAMPLE_PAIRS_SCALEFACTOR_1: Vec<Vec<usize>> = vec![
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
    static ref DOWNSAMPLE_PAIRS_SCALEFACTOR_2: Vec<Vec<usize>> =
        vec![vec![1, 4], vec![1, 7], vec![4, 10]];
}

pub trait NavcamTile {
    fn get_tile_id_scale_factor_1(&self) -> usize;
    fn get_tile_id_scale_factor_2(&self) -> usize;
    fn get_tile_id(&self) -> usize;
    fn get_scale_factor(&self) -> u32;
    fn get_subframe_region(&self) -> Vec<f64>;
}

pub trait BufferGetBorderOverLap {
    fn get_left(&self) -> Result<Self>
    where
        Self: Sized;
    fn get_right(&self) -> Result<Self>
    where
        Self: Sized;
    fn get_top(&self) -> Result<Self>
    where
        Self: Sized;
    fn get_bottom(&self) -> Result<Self>
    where
        Self: Sized;
}

impl BufferGetBorderOverLap for ImageBuffer {
    fn get_left(&self) -> Result<ImageBuffer> {
        self.get_subframe(self.width - 12, 0, 12, self.height)
    }
    fn get_right(&self) -> Result<ImageBuffer> {
        self.get_subframe(0, 0, 12, self.height)
    }
    fn get_top(&self) -> Result<ImageBuffer> {
        self.get_subframe(0, 0, self.width, 12)
    }
    fn get_bottom(&self) -> Result<ImageBuffer> {
        self.get_subframe(0, self.height - 12, self.width, 12)
    }
}

impl NavcamTile for MarsImage {
    fn get_tile_id(&self) -> usize {
        match self.get_scale_factor() {
            1 => self.get_tile_id_scale_factor_1(),
            2 => self.get_tile_id_scale_factor_2(),
            _ => panic!("Error: Cannot determine tile id: Unsupported scale factor"),
        }
    }
    fn get_tile_id_scale_factor_1(&self) -> usize {
        let sf = self.get_subframe_region();
        let x_frac = (sf[0] / 5121.0 * 4.0).round() as usize;
        let y_frac = (sf[1] / 3841.0 * 4.0).round() as usize;
        SUBFRAME_IDS_SCALE_FACTOR_1[y_frac][x_frac]
    }

    fn get_tile_id_scale_factor_2(&self) -> usize {
        let sf = self.get_subframe_region();
        let x_frac = (sf[0] / 2560.0).round() as usize;
        let y_frac = (sf[1] / 1920.0).round() as usize;
        SUBFRAME_IDS_SCALE_FACTOR_2[y_frac][x_frac]
    }

    fn get_scale_factor(&self) -> u32 {
        if let Some(md) = &self.metadata {
            md.scale_factor
        } else {
            1
        }
    }
    fn get_subframe_region(&self) -> Vec<f64> {
        if let Some(md) = &self.metadata {
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
