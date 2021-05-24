
// https://www.researchgate.net/publication/238183352_An_Image_Inpainting_Technique_Based_on_the_Fast_Marching_Method


use crate::{
    constants, 
    path, 
    error, 
    enums, 
    imagebuffer::ImageBuffer, 
    vprintln,
    stats,
    rgbimage::RgbImage,
    ok,
    calibfile
};

#[derive(Debug, Clone)]
struct Point {
    x: usize,
    y: usize,
    score: u32
}

struct RgbVec {
    rgb: Vec<[f32; 3]>,
    width: usize,
    height: usize
}

const DEFAULT_WINDOW_SIZE : i32 = 3;

fn determine_mask_file(instrument:enums::Instrument) -> error::Result<String> {
    calibfile::get_calibration_file_for_instrument(instrument, enums::CalFileType::InpaintMask)
}

pub fn inpaint_supported_for_instrument(instrument:enums::Instrument) -> bool {
    let r = determine_mask_file(instrument);
    r.is_ok()
}

fn load_mask_file(filename:&str, instrument:enums::Instrument) -> error::Result<ImageBuffer> {
    vprintln!("Loading inpaint mask file {}", filename);

    if ! path::file_exists(filename) {
        return Err(constants::status::FILE_NOT_FOUND);
    }
    let mask = match ImageBuffer::from_file(filename) {
        Ok(m) => m,
        Err(e) => return Err(e)
    };
    
    match instrument {
        enums::Instrument::MslMAHLI => mask.get_subframe(32, 16, 1584, 1184),
        _ => Ok(mask)
    }
}

fn load_mask(instrument:enums::Instrument) -> error::Result<ImageBuffer> {
    let mask_file = match determine_mask_file(instrument) {
        Ok(m) => m,
        Err(e) => return Err(e)
    };

    load_mask_file(mask_file.as_str(), instrument)
}

fn get_num_good_neighbors(mask:&ImageBuffer, x:i32, y:i32) -> u32 {

    // Juggling the possibility of negitive numbers and whether or now we allow that.
    let t = if y > 0 { mask.get(x as usize, (y-1) as usize).unwrap() == 0.0 } else { false };
    let tl = if x > 0 && y > 0 { mask.get((x-1) as usize, (y-1) as usize).unwrap() == 0.0 } else { false };
    let l = if x > 0 { mask.get((x-1)  as usize, y as usize).unwrap() == 0.0 } else { false };
    let bl = if x > 0 && y < mask.height as i32 - 1 { mask.get((x-1) as usize, (y+1) as usize).unwrap() == 0.0 } else { false };
    let b = if y < mask.height as i32 - 1 { mask.get(x as usize, (y+1) as usize).unwrap() == 0.0 } else { false };
    let br = if x < mask.width as i32 - 1 && y < mask.height as i32 - 1 { mask.get((x+1) as usize, (y+1) as usize).unwrap() == 0.0 } else { false };
    let r = if x < mask.width as i32 - 1 { mask.get((x+1) as usize, y as usize).unwrap() == 0.0 } else { false };
    let tr = if x < mask.width as i32 - 1 && y > 0 { mask.get((x+1) as usize, (y-1) as usize).unwrap() == 0.0 } else { false };

    let mut s = 0;

    s += if t  { 1 } else { 0 };
    s += if tl { 1 } else { 0 };
    s += if l  { 1 } else { 0 };
    s += if bl { 1 } else { 0 };
    s += if b  { 1 } else { 0 };
    s += if br { 1 } else { 0 };
    s += if r  { 1 } else { 0 };
    s += if tr { 1 } else { 0 };

    s
}

// SOOOOOOooooooooo sloooooooooooooooow :-(
fn find_starting_point(mask:&ImageBuffer) -> Option<Point> {
    for y in 0..mask.height {
        for x in 0..mask.width {
            if let Ok(v) = mask.get(x, y) {
                if v > 0.0 {
                    return Some(Point{x, y, score:0});
                }
            }
        }
    }
    None
}

fn isolate_window(buffer:&RgbVec, mask:&ImageBuffer, channel:usize, window_size:i32, x:usize, y:usize) -> Vec<f32> {
    let mut v:Vec<f32> = Vec::with_capacity(36);
    let start = window_size / 2 * -1;
    let end = window_size / 2 + 1;
    for _y in start..end as i32 {
        for _x in start..end as i32 {
            let get_x = x as i32 + _x;
            let get_y = y as i32 + _y;
            if get_x >= 0 
                && get_x < buffer.width as i32 
                && get_y >= 0 
                && get_y < buffer.height as i32
                && mask.get(get_x as usize, get_y as usize).unwrap() == 0.0
                {
                v.push(buffer.rgb[(get_y * buffer.width as i32 + get_x) as usize][channel]);
            }
        }
    }
    v
}

fn predict_value(buffer:&RgbVec, mask:&ImageBuffer, channel:usize, x:usize, y:usize) -> f32 {
    let window = isolate_window(&buffer, &mask, channel, DEFAULT_WINDOW_SIZE, x, y);
    stats::mean(&window[0..]).unwrap()
}


fn get_point_and_score_at_xy(mask:&ImageBuffer, x:i32, y:i32) -> Option<Point> {

    if x < 0 || x >= mask.width as i32 || y < 0 || y >= mask.height as i32 {
        return None;
    }

    let v = mask.get(x as usize, y as usize).unwrap();
    if v == 0.0 {
        return None;
    }

    let score = get_num_good_neighbors(&mask, x, y);

    Some(Point{x:x as usize, y:y as usize, score})
}


fn find_larger(left:Option<Point>, right:&Point) -> Point {
    match left {
        Some(pt) => {
            if pt.score > right.score { pt } else { right.clone() }
        },
        None => right.to_owned()
    }
}

fn find_next_point(mask:&ImageBuffer, x:i32, y:i32) -> Option<Point> {
    let mut pts : Vec<Option<Point>> = Vec::with_capacity(8);

    pts.push(get_point_and_score_at_xy(&mask, x, y - 1));
    pts.push(get_point_and_score_at_xy(&mask, x - 1, y - 1));
    pts.push(get_point_and_score_at_xy(&mask, x - 1, y));
    pts.push(get_point_and_score_at_xy(&mask, x - 1, y + 1));
    pts.push(get_point_and_score_at_xy(&mask, x, y + 1));
    pts.push(get_point_and_score_at_xy(&mask, x + 1, y + 1));
    pts.push(get_point_and_score_at_xy(&mask, x + 1, y));
    pts.push(get_point_and_score_at_xy(&mask, x + 1, y - 1));

    let mut largest_score : Option<Point> = None;

    for opt_pt in pts.iter() {
        match opt_pt {
            Some(pt) => {
                largest_score = Some(find_larger(largest_score, pt));
            },
            None => ()
        }
    }

    largest_score
}


fn infill(buffer:&mut RgbVec, mask:&mut ImageBuffer, starting:&Point) -> error::Result<&'static str> {

    let mut current = starting.to_owned();
    loop {
        let pt_new_value_0 = predict_value(&buffer, &mask, 0, current.x, current.y);
        let pt_new_value_1 = predict_value(&buffer, &mask, 1, current.x, current.y);
        let pt_new_value_2 = predict_value(&buffer, &mask, 2, current.x, current.y);
        
        buffer.rgb[current.y * buffer.width + current.x][0] = pt_new_value_0;
        buffer.rgb[current.y * buffer.width + current.x][1] = pt_new_value_1;
        buffer.rgb[current.y * buffer.width + current.x][2] = pt_new_value_2;

        match mask.put(current.x, current.y, 0.0) {
            Ok(_) => (),
            Err(e) => return Err(e)
        }

        match find_next_point(&mask, current.x as i32, current.y as i32) {
            Some(pt) => current = pt.to_owned(),
            None => break
        }
    }
    ok!()
}

fn rgb_image_to_vec(rgb:&RgbImage) -> error::Result<RgbVec> {

    let mut v: Vec<[f32; 3]> = Vec::with_capacity(rgb.width * rgb.height);
    v.resize(rgb.width * rgb.height, [0.0, 0.0, 0.0]);

    for y in 0..rgb.height {
        for x in 0..rgb.width {
            let idx = y * rgb.width + x;
            let r = match rgb.red().get(x, y) {
                Ok(v) => v,
                Err(e) => return Err(e)
            };
            let g = match rgb.green().get(x, y) {
                Ok(v) => v,
                Err(e) => return Err(e)
            };
            let b = match rgb.blue().get(x, y) {
                Ok(v) => v,
                Err(e) => return Err(e)
            };
            v[idx][0] = r;
            v[idx][1] = g;
            v[idx][2] = b;
        }
    }

    Ok(RgbVec{rgb:v, width:rgb.width, height:rgb.height})
}

fn vec_to_rgb_image(buffer:&RgbVec) -> error::Result<RgbImage> {
    let mut red = ImageBuffer::new(buffer.width, buffer.height).unwrap();
    let mut green = ImageBuffer::new(buffer.width, buffer.height).unwrap();
    let mut blue = ImageBuffer::new(buffer.width, buffer.height).unwrap();

    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let r = buffer.rgb[y * (buffer.width) + x][0];
            let g = buffer.rgb[y * (buffer.width) + x][1];
            let b = buffer.rgb[y * (buffer.width) + x][2];
            match red.put(x, y, r) {
                Ok(_) => (),
                Err(e) => return Err(e)
            };
            match green.put(x, y, g) {
                Ok(_) => (),
                Err(e) => return Err(e)
            };
            match blue.put(x, y, b) {
                Ok(_) => (),
                Err(e) => return Err(e)
            };
        }
    }

    RgbImage::new_from_buffers_rgb(&red, &green, &blue, enums::Instrument::None, enums::ImageMode::U8BIT)
}




// Embarrassingly slow and inefficient. Runs slow in debug. A lot faster with a release build.
pub fn apply_inpaint_to_buffer_with_mask(rgb:&RgbImage, mask_src:&ImageBuffer) -> error::Result<RgbImage> {

    let mut working_buffer = match rgb_image_to_vec(&rgb) {
        Ok(b) => b,
        Err(e) => return Err(e)
    };

    let mut mask = mask_src.clone();

    // Crop the mask image if it's larger than the input image. 
    // Sizes need to match
    if mask.width > working_buffer.width {
        let x = (mask.width - working_buffer.width) / 2;
        let y = (mask.height - working_buffer.height) / 2;
        vprintln!("Cropping inpaint mask with params {}, {}, {}, {}", x, y, working_buffer.width, working_buffer.height);
        mask = match mask.get_subframe(x, y, working_buffer.width, working_buffer.height) {
            Ok(m) => m,
            Err(_) => return Err("Error subframing mask")
        }
    }

    // For this to work, we need the mask to be mutable and we're
    // going to fill it in with 0x0 values as we go. If we don't, then
    // we'll keep finding starting points and this will be an infinite
    // loop. Which is bad. Perhaps consider an alternate method here.
    loop {

        // TODO: Don't leave embedded match statements. I hate that as much as embedded case statements...
        match find_starting_point(&mask) {
            Some(pt) => {
                //vprintln!("Starting point: {}, {}", pt.x, pt.y);
                match infill(&mut working_buffer, &mut mask, &pt) {
                    Ok(_) => (),
                    Err(e) => return Err(e)
                };
            },
            None => break
        };
    }

    let mut newimage = match vec_to_rgb_image(&working_buffer) {
        Ok(i) => i,
        Err(e) => return Err(e)
    };
    newimage.set_instrument(rgb.get_instrument());
    
    Ok(newimage)
}


pub fn apply_inpaint_to_buffer(rgb:&RgbImage) -> error::Result<RgbImage> {

    let mask = match load_mask(rgb.get_instrument()) {
        Ok(m) => m,
        Err(_) => return Err("Error loading mask")
    };

    apply_inpaint_to_buffer_with_mask(&rgb, &mask)
}

pub fn make_mask_from_red(rgbimage:&RgbImage) -> error::Result<ImageBuffer> {

    let mut new_mask = match ImageBuffer::new(rgbimage.width, rgbimage.height) {
        Ok(b) => b,
        Err(e) => return Err(e)
    };
    for y in 0..rgbimage.height {
        for x in 0..rgbimage.width {
            let r = rgbimage.red().get(x, y).unwrap();
            let g = rgbimage.green().get(x, y).unwrap();
            let b = rgbimage.blue().get(x, y).unwrap();
            
            // if r != g || r != b || g != b {
            //     new_mask.put(x, y, 255.0).unwrap();
            // }
            if r == 255.0 && g == 0.0 && b == 0.0 {
                new_mask.put(x, y, 255.0).unwrap();
            }

        }
    }


    Ok(new_mask)
}