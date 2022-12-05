use sciimg::prelude::*;

pub fn decorrelation_stretch(image: &mut RgbImage) {
    let mut r_sum = 0.0;
    let mut g_sum = 0.0;
    let mut b_sum = 0.0;

    let mut r_squared_sum = 0.0;
    let mut g_squared_sum = 0.0;
    let mut b_squared_sum = 0.0;

    let num_pixels = (image.width * image.height) as f32;

    let mut r_buffer = image.get_band(0).clone();
    let mut g_buffer = image.get_band(1).clone();
    let mut b_buffer = image.get_band(2).clone();

    for y in 0..image.height {
        for x in 0..image.width {
            r_sum += image.get_band(0).get(x, y).unwrap();
            g_sum += image.get_band(1).get(x, y).unwrap();
            b_sum += image.get_band(2).get(x, y).unwrap();

            r_squared_sum += image.get_band(0).get(x, y).unwrap().powf(2.0);
            g_squared_sum += image.get_band(1).get(x, y).unwrap().powf(2.0);
            b_squared_sum += image.get_band(2).get(x, y).unwrap().powf(2.0);
        }
    }

    let r_mean = r_sum / num_pixels;
    let g_mean = g_sum / num_pixels;
    let b_mean = b_sum / num_pixels;

    let r_stddev = (r_squared_sum / num_pixels - r_mean * r_mean).sqrt();
    let g_stddev = (g_squared_sum / num_pixels - g_mean * g_mean).sqrt();
    let b_stddev = (b_squared_sum / num_pixels - b_mean * b_mean).sqrt();

    for y in 0..image.height {
        for x in 0..image.width {
            let mut r = image.get_band(0).get(x, y).unwrap();
            let mut g = image.get_band(1).get(x, y).unwrap();
            let mut b = image.get_band(2).get(x, y).unwrap();

            r = (r - r_mean) / r_stddev;
            g = (g - g_mean) / g_stddev;
            b = (b - b_mean) / b_stddev;

            r_buffer.put(x, y, r);
            g_buffer.put(x, y, g);
            b_buffer.put(x, y, b);
        }
    }

    image.set_band(&r_buffer, 0);
    image.set_band(&g_buffer, 1);
    image.set_band(&b_buffer, 2);
    image.set_mode(ImageMode::U16BIT);
}
