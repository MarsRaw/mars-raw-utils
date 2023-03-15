use crate::image::MarsImage;
use crate::vprintln;

use lazy_static;
use sciimg::{error::Result, prelude::ImageBuffer, rgbimage::RgbImage};

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

pub trait NavcamTile {
    fn get_tile_id_scale_factor_1(&self) -> usize;
    fn get_tile_id_scale_factor_2(&self) -> usize;
    fn get_tile_id(&self) -> usize;
    fn get_scale_factor(&self) -> u32;
    fn get_subframe_region(&self) -> Vec<f64>;
    fn match_levels(&self, adjust: &mut Self);
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

trait RgbImageAdjust {
    fn mean(&self) -> f32;
    fn determine_match_normalize_high(&self, target: &Self) -> f32;
}

pub fn get_subframes_for_tile_id_pair_scale_factor_2(
    target: &RgbImage,
    adjust: &RgbImage,
    target_tile_id: usize,
    adjust_tile_id: usize,
) -> (RgbImage, RgbImage) {
    let pair = (target_tile_id, adjust_tile_id);
    if pair == (1, 4) || pair == (7, 10) {
        (target.get_right().unwrap(), adjust.get_left().unwrap())
    } else if pair == (1, 7) || pair == (4, 10) {
        (target.get_bottom().unwrap(), adjust.get_top().unwrap())
    } else {
        panic!(
            "Unsupported scale factor 2 tile id combination {} / {}",
            target_tile_id, adjust_tile_id
        )
    }
}

pub fn get_subframes_for_tile_id_pair_scale_factor_1(
    target: &RgbImage,
    adjust: &RgbImage,
    target_tile_id: usize,
    adjust_tile_id: usize,
) -> (RgbImage, RgbImage) {
    let pair = (target_tile_id, adjust_tile_id);
    if pair == (1, 5) || pair == (5, 9) || pair == (9, 13) {
        (target.get_right().unwrap(), adjust.get_left().unwrap())
    } else if pair == (1, 2)
        || pair == (2, 3)
        || pair == (3, 4)
        || pair == (5, 6)
        || pair == (6, 7)
        || pair == (7, 8)
        || pair == (9, 10)
        || pair == (10, 11)
        || pair == (11, 12)
        || pair == (13, 14)
        || pair == (14, 15)
        || pair == (15, 16)
    {
        (target.get_bottom().unwrap(), adjust.get_top().unwrap())
    } else {
        panic!("Unsupported scale factor 1 tile id combination")
    }
}

pub fn get_subframes_for_tile_id_pair(
    target: &RgbImage,
    adjust: &RgbImage,
    target_tile_id: usize,
    adjust_tile_id: usize,
    scale_factor: u32,
) -> (RgbImage, RgbImage) {
    match scale_factor {
        1 => get_subframes_for_tile_id_pair_scale_factor_1(
            target,
            adjust,
            target_tile_id,
            adjust_tile_id,
        ),
        2 => get_subframes_for_tile_id_pair_scale_factor_2(
            target,
            adjust,
            target_tile_id,
            adjust_tile_id,
        ),
        _ => panic!("Unsupported scale factor"),
    }
}

impl RgbImageAdjust for RgbImage {
    fn mean(&self) -> f32 {
        (self.get_band(0).sum() + self.get_band(1).sum() + self.get_band(2).sum())
            / (self.get_band(0).width as f32 * self.get_band(0).height as f32 * 3.0)
    }
    fn determine_match_normalize_high(&self, target: &Self) -> f32 {
        let mut prev_diff = None;
        let mut prev_normed_2 = None;
        let mut normalize_to_high = 350.0;

        let (self_min, self_max) = self.get_min_max_all_channel();
        let target_mean = target.mean();

        for i in (self_min as i16 * 10)..3500 {
            let i = f32::from(i) * 0.1;
            //vprintln!("Checking normalization value {}", i);

            let mut normed_2 = self.clone();
            normed_2.normalize_band_to_with_min_max(0, self_min, i, self_min, self_max);
            normed_2.normalize_band_to_with_min_max(1, self_min, i, self_min, self_max);
            normed_2.normalize_band_to_with_min_max(2, self_min, i, self_min, self_max);

            if prev_normed_2.is_some() && prev_diff.is_some() {
                let curr_diff = (target_mean - normed_2.mean()).abs();
                println!("Curr Diff: {} ({})", curr_diff, i);
                if let Some(pd) = prev_diff {
                    if curr_diff > pd {
                        vprintln!("Correcting high to within a mean difference of {}", pd);
                        normalize_to_high = i - 0.1;
                        break;
                    }
                }
            }
            if prev_normed_2.is_some() {
                prev_diff = Some((target_mean - normed_2.mean()).abs())
            }
            prev_normed_2 = Some(normed_2);
        }

        normalize_to_high
    }
}

impl BufferGetBorderOverLap for ImageBuffer {
    fn get_left(&self) -> Result<ImageBuffer> {
        self.get_subframe(0, 0, 16, self.height)
    }
    fn get_right(&self) -> Result<ImageBuffer> {
        self.get_subframe(self.width - 16, 0, 16, self.height)
    }
    fn get_top(&self) -> Result<ImageBuffer> {
        self.get_subframe(0, 0, self.width, 16)
    }
    fn get_bottom(&self) -> Result<ImageBuffer> {
        self.get_subframe(0, self.height - 16, self.width, 16)
    }
}

impl BufferGetBorderOverLap for RgbImage {
    fn get_left(&self) -> Result<RgbImage> {
        RgbImage::new_from_buffers_rgb(
            &self
                .get_band(0)
                .get_left()
                .expect("Failed to retrieve subframe, channel 0"),
            &self
                .get_band(1)
                .get_left()
                .expect("Failed to retrieve subframe, channel 1"),
            &self
                .get_band(2)
                .get_left()
                .expect("Failed to retrieve subframe, channel 2"),
            self.get_mode(),
        )
    }
    fn get_right(&self) -> Result<RgbImage> {
        RgbImage::new_from_buffers_rgb(
            &self
                .get_band(0)
                .get_right()
                .expect("Failed to retrieve subframe, channel 0"),
            &self
                .get_band(1)
                .get_right()
                .expect("Failed to retrieve subframe, channel 1"),
            &self
                .get_band(2)
                .get_right()
                .expect("Failed to retrieve subframe, channel 2"),
            self.get_mode(),
        )
    }
    fn get_top(&self) -> Result<RgbImage> {
        RgbImage::new_from_buffers_rgb(
            &self
                .get_band(0)
                .get_top()
                .expect("Failed to retrieve subframe, channel 0"),
            &self
                .get_band(1)
                .get_top()
                .expect("Failed to retrieve subframe, channel 1"),
            &self
                .get_band(2)
                .get_top()
                .expect("Failed to retrieve subframe, channel 2"),
            self.get_mode(),
        )
    }
    fn get_bottom(&self) -> Result<RgbImage> {
        RgbImage::new_from_buffers_rgb(
            &self
                .get_band(0)
                .get_bottom()
                .expect("Failed to retrieve subframe, channel 0"),
            &self
                .get_band(1)
                .get_bottom()
                .expect("Failed to retrieve subframe, channel 1"),
            &self
                .get_band(2)
                .get_bottom()
                .expect("Failed to retrieve subframe, channel 2"),
            self.get_mode(),
        )
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
        if self.get_scale_factor() != 1 {
            panic!("Cannot determine scale factor 1 tile id on a scale factor != 1 image");
        }
        let sf = self.get_subframe_region();
        let x_frac = (sf[0] / 5121.0 * 4.0).round() as usize;
        let y_frac = (sf[1] / 3841.0 * 4.0).round() as usize;
        SUBFRAME_IDS_SCALE_FACTOR_1[y_frac][x_frac]
    }

    fn get_tile_id_scale_factor_2(&self) -> usize {
        if self.get_scale_factor() != 2 {
            panic!("Cannot determine scale factor 2 tile id on a scale factor != 2 image");
        }
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

    fn match_levels(&self, adjust: &mut Self) {
        let target_tile_id = self.get_tile_id();
        let adjust_tile_id = adjust.get_tile_id();

        let (target_subframe, adjust_subframe) = get_subframes_for_tile_id_pair(
            &self.image,
            &adjust.image,
            target_tile_id,
            adjust_tile_id,
            self.get_scale_factor(),
        );

        let normalization_factor_high =
            adjust_subframe.determine_match_normalize_high(&target_subframe);

        // corrected_image_2 = normalize(adjust_image[:,:], sub_frame_2_0.min(), sub_frame_2_0.max(), sub_frame_2_0.min(), normalize_to_high)
        let (adjust_min, adjust_max) = adjust_subframe.get_min_max_all_channel();
        adjust.image.normalize_band_to_with_min_max(
            0,
            adjust_min,
            normalization_factor_high,
            adjust_min,
            adjust_max,
        );
    }
}

fn determine_match_normalize_high(target: &MarsImage, adjust: &MarsImage) -> (f32, f32, f32) {
    let target_tile_id = target.get_tile_id();
    let adjust_tile_id = adjust.get_tile_id();

    let (target_subframe, adjust_subframe) = get_subframes_for_tile_id_pair(
        &target.image,
        &adjust.image,
        target_tile_id,
        adjust_tile_id,
        target.get_scale_factor(),
    );

    target_subframe.save("/data/M20/0629/NCAM/scale2-vert/NLF_0629_0722785336_039ECM_N0301524NCAM00428_01_195J01-subframe.png");
    adjust_subframe.save("/data/M20/0629/NCAM/scale2-vert/NLF_0629_0722785336_039ECM_N0301524NCAM00428_07_195J01-subframe.png");

    let normalization_factor_high =
        adjust_subframe.determine_match_normalize_high(&target_subframe);

    let (adjust_min, adjust_max) = adjust_subframe.get_min_max_all_channel();

    (adjust_min, adjust_max, normalization_factor_high)
}

fn get_image_index_by_id(images: &[MarsImage], tile_id: usize) -> Option<usize> {
    let mut found_image_index = None;
    for (i, _) in images.iter().enumerate() {
        if images[i].get_tile_id() == tile_id {
            vprintln!("Image found for tile id {}", tile_id);
            found_image_index = Some(i);
            break;
        }
    }
    found_image_index
}

pub fn match_levels(images: &mut [MarsImage]) {
    for pair in FRAME_MATCH_PAIRS_SCALEFACTOR_2.iter() {
        let target_index_opt = get_image_index_by_id(images, pair[0]);
        let adjust_index_opt = get_image_index_by_id(images, pair[1]);

        if target_index_opt.is_none() || adjust_index_opt.is_none() {
            continue;
        }

        let target_index = target_index_opt.unwrap();
        let adjust_index = adjust_index_opt.unwrap();

        vprintln!("Checking pair ({}, {})", pair[0], pair[1]);

        let (adjust_min, adjust_max, normalization_factor_high) =
            determine_match_normalize_high(&images[target_index], &images[adjust_index]);

        vprintln!(
            "Adjusting pair ({}, {}) with high value of {}",
            pair[0],
            pair[1],
            normalization_factor_high
        );
        // corrected_image_2 = normalize(adjust_image[:,:], sub_frame_2_0.min(), sub_frame_2_0.max(), sub_frame_2_0.min(), normalize_to_high)
        images[adjust_index].image.normalize_band_to_with_min_max(
            0,
            adjust_min,
            normalization_factor_high,
            adjust_min,
            adjust_max,
        );
        images[adjust_index].image.normalize_band_to_with_min_max(
            1,
            adjust_min,
            normalization_factor_high,
            adjust_min,
            adjust_max,
        );
        images[adjust_index].image.normalize_band_to_with_min_max(
            2,
            adjust_min,
            normalization_factor_high,
            adjust_min,
            adjust_max,
        );
    }
}
