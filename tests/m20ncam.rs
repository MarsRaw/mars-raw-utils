use mars_raw_utils::enums::Instrument;
use mars_raw_utils::image::MarsImage;
use mars_raw_utils::m20::ncamtile::BufferGetBorderOverLap;
use mars_raw_utils::m20::ncamtile::NavcamTile;

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
    assert_eq!(top.height, 12);

    assert_eq!(bottom.width, raw.image.width);
    assert_eq!(bottom.height, 12);

    assert_eq!(left.width, 12);
    assert_eq!(left.height, raw.image.height);

    assert_eq!(right.width, 12);
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
    assert_eq!(top.height, 12);
    assert_eq!(top.num_bands(), 3);

    assert_eq!(bottom.width, raw.image.width);
    assert_eq!(bottom.height, 12);
    assert_eq!(bottom.num_bands(), 3);

    assert_eq!(left.width, 12);
    assert_eq!(left.height, raw.image.height);
    assert_eq!(left.num_bands(), 3);

    assert_eq!(right.width, 12);
    assert_eq!(right.height, raw.image.height);
    assert_eq!(right.num_bands(), 3);
}

#[test]
fn test_tile_id_determination() {
    let raw = load_test_image_navright_sf_2();
    assert_eq!(raw.get_subframe_region(), vec![2545.0, 1.0, 2576.0, 1936.0]);
    assert_eq!(raw.get_tile_id(), 4);
    assert_eq!(raw.get_tile_id_scale_factor_2(), 4);
    assert_eq!(raw.get_scale_factor(), 2);
}

#[test]
#[should_panic]
fn test_tile_id_determination_wrong_scale_factor() {
    let raw = load_test_image_navright_sf_2();
    let _ = raw.get_tile_id_scale_factor_1();
}
