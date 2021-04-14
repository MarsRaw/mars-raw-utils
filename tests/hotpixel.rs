
use mars_raw_utils::{
    hotpixel,
    imagebuffer
};

const MSL_ECAM_NRB_WITH_HOT_PIXELS : &str = "tests/testdata/NRB_670586006EDR_S0871444NCAM00545M_.jpg";

#[test]
fn test_hot_pixel_correction() {
    let img = imagebuffer::ImageBuffer::from_file(MSL_ECAM_NRB_WITH_HOT_PIXELS).unwrap();

    let hpc_results_2p0 = hotpixel::hot_pixel_detection(&img, 6, 2.0).unwrap();
    assert_eq!(hpc_results_2p0.replaced_pixels.len(), 21218);

    let hpc_results_2p5 = hotpixel::hot_pixel_detection(&img, 6, 2.5).unwrap();
    assert_eq!(hpc_results_2p5.replaced_pixels.len(), 8404);

    // Doesn't always get all of them and/or accidentally creates new ones via the replacement method...
    let hpc_results_2p5_2nd_pass = hotpixel::hot_pixel_detection(&hpc_results_2p5.buffer, 6, 2.5).unwrap();
    assert_eq!(hpc_results_2p5_2nd_pass.replaced_pixels.len(), 2002);
}