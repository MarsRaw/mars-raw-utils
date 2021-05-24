
use crate::{
    error, 
    imagebuffer::ImageBuffer, 
    rgbimage::RgbImage,
    enums,
    vprintln
};

// G at R locations
// G at B locations
#[allow(non_upper_case_globals)]
pub const GR_GB : [f32; 25] = [
    0.0,  0.0, -1.0,  0.0,  0.0,
    0.0,  0.0,  2.0,  0.0,  0.0,
   -1.0,  2.0,  4.0,  2.0, -1.0,
    0.0,  0.0,  2.0,  0.0,  0.0, 
    0.0,  0.0, -1.0,  0.0,  0.0
];

// R at G in R row, B column
// B at G in B row, R column
#[allow(non_upper_case_globals)]
pub const Rg_RB_Bg_BR : [f32; 25] = [
    0.0,  0.0,  0.5,  0.0,  0.0,
    0.0, -1.0,  0.0, -1.0,  0.0,
   -1.0,  4.0,  5.0,  4.0, -1.0,
    0.0, -1.0,  0.0, -1.0,  0.0,
    0.0,  0.0,  0.5,  0.0,  0.0
];

// R at G in B row, R column
// B at G in R row, B column
#[allow(non_upper_case_globals)]
pub const Rg_BR_Bg_RB : [f32; 25] = [
    0.0,  0.0, -1.0,  0.0,  0.0,
    0.0, -1.0,  4.0, -1.0,  0.0,
    0.5,  0.0,  5.0,  0.0,  0.5,
    0.0, -1.0,  4.0, -1.0,  0.0,
    0.0,  0.0, -1.0,  0.0,  0.0
];

//R at B in B row, B column
//B at R in R row, R column
#[allow(non_upper_case_globals)]
pub const Rb_BB_Br_RR : [f32; 25] = [
    0.0,  0.0, -1.5,  0.0,  0.0,
    0.0,  2.0,  0.0,  2.0,  0.0,
   -1.5,  0.0,  6.0,  0.0, -1.5,
    0.0,  2.0,  0.0,  2.0,  0.0,
    0.0,  0.0, -1.5,  0.0,  0.0
];

fn extract_window(buffer:&ImageBuffer, x:usize, y:usize, data_5x5_window:&mut [f32; 25], mask_5x5_window : &mut [bool; 25]) {

    for ny in -2..3_i32 {
        for nx in -2..3_i32 {
            let bx = x as i32 + nx;
            let by = y as i32 + ny;
            
            if bx < 0 || by < 0 || bx >= buffer.width as i32 || by >= buffer.height as i32 {
                mask_5x5_window[((ny + 2) * 5 + (nx + 2)) as usize] = false;
            } else {
                mask_5x5_window[((ny + 2) * 5 + (nx + 2)) as usize] = true;
                data_5x5_window[((ny + 2) * 5 + (nx + 2)) as usize] = buffer.get(bx as usize, by as usize).unwrap();
            }
        }
    }
}

fn solve(data_5x5_window:&[f32; 25], mask_5x5_window:&[bool; 25], coefficients:&[f32; 25]) -> f32 {
    let mut v = 0.0;
    let mut s = 0.0;
    for i in 0..25_usize {
        if mask_5x5_window[i] {
            v += data_5x5_window[i] * coefficients[i];
            s += coefficients[i];
        }
    }
    v * (1.0/s)
}

pub fn debayer(buffer:&ImageBuffer) -> error::Result<RgbImage> {
    vprintln!("Applying Malvar Demosaicking...");

    let mut red = ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.mask).unwrap();
    let mut green = ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.mask).unwrap();
    let mut blue = ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.mask).unwrap();

    let mut data_5x5_window : [f32; 25] =  [0.0; 25];
    let mut mask_5x5_window : [bool; 25] = [false; 25];

    for y in 0..buffer.height {
        for x in 0..buffer.width {

            extract_window(&buffer, x, y, &mut data_5x5_window, &mut mask_5x5_window);

            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;

            if x % 2 == 0 && y % 2 == 0 { // Then we're at a red pixel
                r = data_5x5_window[12];
                g = solve(&data_5x5_window, &mask_5x5_window, &GR_GB);
                b = solve(&data_5x5_window, &mask_5x5_window, &Rb_BB_Br_RR);
            } else if x % 2 != 0 && y % 2 != 0 { // Then we're a blue pixel
                r = solve(&data_5x5_window, &mask_5x5_window, &Rb_BB_Br_RR);
                g = solve(&data_5x5_window, &mask_5x5_window, &GR_GB);
                b = data_5x5_window[12];
            } else if x % 2 != 0 && y % 2 == 0 { // Then we're at Green, R row, B column
                r = solve(&data_5x5_window, &mask_5x5_window, &Rg_RB_Bg_BR);
                g = data_5x5_window[12];
                b = solve(&data_5x5_window, &mask_5x5_window, &Rg_BR_Bg_RB);
            } else if x % 2 == 0 && y % 2 != 0 { // Then we're at Green, B row, R column
                r = solve(&data_5x5_window, &mask_5x5_window, &Rg_BR_Bg_RB);
                g = data_5x5_window[12];
                b = solve(&data_5x5_window, &mask_5x5_window, &Rg_RB_Bg_BR);
            }

            red.put(x, y, r).unwrap();
            green.put(x, y, g).unwrap();
            blue.put(x, y, b).unwrap();
        }
    }

    let newimage = RgbImage::new_from_buffers_rgb(&red, &green, &blue, enums::Instrument::None, enums::ImageMode::U8BIT).unwrap();
    Ok(newimage)
}

