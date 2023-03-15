use mars_raw_utils::enums::Instrument;
use mars_raw_utils::image::MarsImage;
use mars_raw_utils::m20::assemble::{
    NavcamTile, FRAME_MATCH_PAIRS_SCALEFACTOR_1, FRAME_MATCH_PAIRS_SCALEFACTOR_2,
};
use mars_raw_utils::m20::ncamlevels;
use mars_raw_utils::m20::ncamlevels::BufferGetBorderOverLap;

fn load_test_image_navright_sf_2() -> MarsImage {
    let raw = MarsImage::open(
        String::from("tests/testdata/NRF_0731_0731848568_991ECM_N0361610NCAM12731_04_195J01.png"),
        Instrument::M20NavcamRight,
    );
    let expected_width = 1288;
    let expected_height = 968;
    assert_eq!(raw.image.height, expected_height);
    assert_eq!(raw.image.width, expected_width);
    raw
}

fn load_test_image_navright_sf_1() -> MarsImage {
    let raw = MarsImage::open(
        String::from("tests/testdata/NLF_0670_0726421423_362ECM_N0320604NCAM08111_01_095J01.png"),
        Instrument::M20NavcamLeft,
    );
    let expected_width = 1288;
    let expected_height = 968;
    assert_eq!(raw.image.height, expected_height);
    assert_eq!(raw.image.width, expected_width);
    raw
}

#[test]
fn test_load_image_sf_2() {
    let _ = load_test_image_navright_sf_2();
}

#[test]
fn test_load_image_sf_1() {
    let _ = load_test_image_navright_sf_1();
}

#[test]
fn test_get_buffer_border_overlap() {
    let raw = load_test_image_navright_sf_2();

    let top = raw
        .image
        .get_band(0)
        .get_top()
        .expect("Error extracting top subframe");

    let bottom = raw
        .image
        .get_band(0)
        .get_bottom()
        .expect("Error extracting bottom subframe");
    let left = raw
        .image
        .get_band(0)
        .get_left()
        .expect("Error extracting left subframe");
    let right = raw
        .image
        .get_band(0)
        .get_right()
        .expect("Error extracting right subframe");

    assert_eq!(top.width, raw.image.width);
    assert_eq!(top.height, 16);

    assert_eq!(bottom.width, raw.image.width);
    assert_eq!(bottom.height, 16);

    assert_eq!(left.width, 16);
    assert_eq!(left.height, raw.image.height);

    assert_eq!(right.width, 16);
    assert_eq!(right.height, raw.image.height);
}

#[test]
fn test_get_rgbimage_border_overlap() {
    let raw = load_test_image_navright_sf_2();

    let top = raw.image.get_top().expect("Error extracting top subframe");

    let bottom = raw
        .image
        .get_bottom()
        .expect("Error extracting bottom subframe");
    let left = raw
        .image
        .get_left()
        .expect("Error extracting left subframe");
    let right = raw
        .image
        .get_right()
        .expect("Error extracting right subframe");

    assert_eq!(top.width, raw.image.width);
    assert_eq!(top.height, 16);
    assert_eq!(top.num_bands(), 3);

    assert_eq!(bottom.width, raw.image.width);
    assert_eq!(bottom.height, 16);
    assert_eq!(bottom.num_bands(), 3);

    assert_eq!(left.width, 16);
    assert_eq!(left.height, raw.image.height);
    assert_eq!(left.num_bands(), 3);

    assert_eq!(right.width, 16);
    assert_eq!(right.height, raw.image.height);
    assert_eq!(right.num_bands(), 3);
}

#[test]
fn test_tile_id_determination_scale_factor_2() {
    let raw = load_test_image_navright_sf_2();
    let tile = NavcamTile::new_with_image(&raw);
    assert_eq!(
        tile.get_subframe_region(),
        vec![2545.0, 1.0, 2576.0, 1936.0]
    );
    assert_eq!(tile.get_tile_id(), 4);
    assert_eq!(tile.get_tile_id_scale_factor_2(), 4);
    assert_eq!(tile.get_scale_factor(), 2);
}

#[test]
fn test_tile_id_determination_scale_factor_1() {
    let raw = load_test_image_navright_sf_1();
    let tile = NavcamTile::new_with_image(&raw);
    assert_eq!(tile.get_subframe_region(), vec![1.0, 1.0, 1288.0, 968.0]);
    assert_eq!(tile.get_tile_id(), 1);
    assert_eq!(tile.get_tile_id_scale_factor_1(), 1);
    assert_eq!(tile.get_scale_factor(), 1);
}

#[test]
#[should_panic]
fn test_tile_id_determination_wrong_scale_factor_1() {
    let raw = load_test_image_navright_sf_2();
    let tile = NavcamTile::new_with_image(&raw);
    let _ = tile.get_tile_id_scale_factor_1();
}

#[test]
#[should_panic]
fn test_tile_id_determination_wrong_scale_factor_2() {
    let raw = load_test_image_navright_sf_1();
    let tile = NavcamTile::new_with_image(&raw);
    let _ = tile.get_tile_id_scale_factor_2();
}

#[test]
fn test_get_subframes_for_tile_id_pairs_scale_factor_2() {
    let image1 = load_test_image_navright_sf_2();
    let image2 = load_test_image_navright_sf_2();

    FRAME_MATCH_PAIRS_SCALEFACTOR_2.iter().for_each(|pair| {
        let _ = ncamlevels::get_subframes_for_tile_id_pair(
            &image1.image,
            &image2.image,
            pair[0],
            pair[1],
            2,
        );
    });
}

#[test]
fn test_get_subframes_for_tile_id_pairs_scale_factor_1() {
    let image1 = load_test_image_navright_sf_1();
    let image2 = load_test_image_navright_sf_1();

    FRAME_MATCH_PAIRS_SCALEFACTOR_1.iter().for_each(|pair| {
        let _ = ncamlevels::get_subframes_for_tile_id_pair(
            &image1.image,
            &image2.image,
            pair[0],
            pair[1],
            1,
        );
    });
}
