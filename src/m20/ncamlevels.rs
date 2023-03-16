use crate::m20::assemble::NavcamTile;
use crate::vprintln;

use sciimg::{blur, error::Result, prelude::ImageBuffer, rgbimage::RgbImage};

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

pub trait RgbImageAdjust {
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

            let mut normed_2 = self.clone();
            normed_2.normalize_band_to_with_min_max(0, self_min, i, self_min, self_max);
            normed_2.normalize_band_to_with_min_max(1, self_min, i, self_min, self_max);
            normed_2.normalize_band_to_with_min_max(2, self_min, i, self_min, self_max);

            if prev_normed_2.is_some() && prev_diff.is_some() {
                let curr_diff = (target_mean - normed_2.mean()).abs();
                vprintln!("Curr Diff: {} ({})", curr_diff, i);
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
        self.get_subframe(2, 2, 12, self.height - 4)
    }
    fn get_right(&self) -> Result<ImageBuffer> {
        self.get_subframe(self.width - 14, 2, 12, self.height - 4)
    }
    fn get_top(&self) -> Result<ImageBuffer> {
        self.get_subframe(2, 2, self.width - 4, 12)
    }
    fn get_bottom(&self) -> Result<ImageBuffer> {
        self.get_subframe(2, self.height - 14, self.width - 4, 12)
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

pub fn determine_match_normalize_high(
    target: &NavcamTile,
    adjust: &NavcamTile,
) -> (f32, f32, f32, f32) {
    let target_tile_id = target.get_tile_id();
    let adjust_tile_id = adjust.get_tile_id();

    let (mut target_subframe, mut adjust_subframe) = get_subframes_for_tile_id_pair(
        &target.image.image,
        &adjust.image.image,
        target_tile_id,
        adjust_tile_id,
        target.get_scale_factor(),
    );

    //target_subframe.save("/home/kgill/data/M20/0629/NCAM/foo-sf-0.png");
    //adjust_subframe.save("/home/kgill/data/M20/0629/NCAM/foo-sf-1.png");

    //let normalization_factor_high =
    //    adjust_subframe.determine_match_normalize_high(&target_subframe);
    target_subframe.set_band(
        &blur::blur_imagebuffer(target_subframe.get_band(0), 10.0),
        0,
    );
    target_subframe.set_band(
        &blur::blur_imagebuffer(target_subframe.get_band(1), 10.0),
        1,
    );
    target_subframe.set_band(
        &blur::blur_imagebuffer(target_subframe.get_band(2), 10.0),
        2,
    );

    adjust_subframe.set_band(
        &blur::blur_imagebuffer(adjust_subframe.get_band(0), 10.0),
        0,
    );
    adjust_subframe.set_band(
        &blur::blur_imagebuffer(adjust_subframe.get_band(1), 10.0),
        1,
    );
    adjust_subframe.set_band(
        &blur::blur_imagebuffer(adjust_subframe.get_band(2), 10.0),
        2,
    );

    let (adjust_min, adjust_max) = adjust_subframe.get_min_max_all_channel();
    let (target_min, target_max) = target_subframe.get_min_max_all_channel();
    (adjust_min, adjust_max, target_min, target_max)
}

fn get_image_index_by_id(images: &[NavcamTile], tile_id: usize) -> Option<usize> {
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

pub fn match_levels_with_pairs(images: &mut [NavcamTile], pair_list: &Vec<Vec<usize>>) {
    for pair in pair_list {
        let target_index_opt = get_image_index_by_id(images, pair[0]);
        let adjust_index_opt = get_image_index_by_id(images, pair[1]);

        if target_index_opt.is_none() || adjust_index_opt.is_none() {
            continue;
        }

        let target_index = target_index_opt.unwrap();
        let adjust_index = adjust_index_opt.unwrap();

        vprintln!("Checking pair ({}, {})", pair[0], pair[1]);

        let (adjust_min, adjust_max, normalization_factor_low, normalization_factor_high) =
            determine_match_normalize_high(&images[target_index], &images[adjust_index]);

        vprintln!(
            "Adjusting pair ({}, {}) with high value of {}",
            pair[0],
            pair[1],
            normalization_factor_high
        );
        // corrected_image_2 = normalize(adjust_image[:,:], sub_frame_2_0.min(), sub_frame_2_0.max(), sub_frame_2_0.min(), normalize_to_high)
        images[adjust_index]
            .image
            .image
            .normalize_band_to_with_min_max(
                0,
                normalization_factor_low,
                normalization_factor_high,
                adjust_min,
                adjust_max,
            );
        images[adjust_index]
            .image
            .image
            .normalize_band_to_with_min_max(
                1,
                normalization_factor_low,
                normalization_factor_high,
                adjust_min,
                adjust_max,
            );
        images[adjust_index]
            .image
            .image
            .normalize_band_to_with_min_max(
                2,
                normalization_factor_low,
                normalization_factor_high,
                adjust_min,
                adjust_max,
            );
    }
}

pub fn match_levels(images: &mut [NavcamTile]) {
    if images.is_empty() {
        return;
    }

    if images[0].get_scale_factor() == 2 {
        match_levels_with_pairs(images, &vec![vec![1, 4], vec![1, 7], vec![7, 10]]);
    } else if images[0].get_scale_factor() == 1 {
        match_levels_with_pairs(
            images,
            &vec![
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
            ],
        );
    }

    /*
    for pair in FRAME_MATCH_PAIRS_SCALEFACTOR_2.iter() {
        let target_index_opt = get_image_index_by_id(images, pair[0]);
        let adjust_index_opt = get_image_index_by_id(images, pair[1]);

        if target_index_opt.is_none() || adjust_index_opt.is_none() {
            continue;
        }

        let target_index = target_index_opt.unwrap();
        let adjust_index = adjust_index_opt.unwrap();

        vprintln!("Checking pair ({}, {})", pair[0], pair[1]);

        let (adjust_min, adjust_max, normalization_factor_low, normalization_factor_high) =
            determine_match_normalize_high(&images[target_index], &images[adjust_index]);

        vprintln!(
            "Adjusting pair ({}, {}) with high value of {}",
            pair[0],
            pair[1],
            normalization_factor_high
        );
        // corrected_image_2 = normalize(adjust_image[:,:], sub_frame_2_0.min(), sub_frame_2_0.max(), sub_frame_2_0.min(), normalize_to_high)
        images[adjust_index]
            .image
            .image
            .normalize_band_to_with_min_max(
                0,
                normalization_factor_low,
                normalization_factor_high,
                adjust_min,
                adjust_max,
            );
        images[adjust_index]
            .image
            .image
            .normalize_band_to_with_min_max(
                1,
                normalization_factor_low,
                normalization_factor_high,
                adjust_min,
                adjust_max,
            );
        images[adjust_index]
            .image
            .image
            .normalize_band_to_with_min_max(
                2,
                normalization_factor_low,
                normalization_factor_high,
                adjust_min,
                adjust_max,
            );
    }
    */
}
