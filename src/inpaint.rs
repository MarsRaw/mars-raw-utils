
// https://www.researchgate.net/publication/238183352_An_Image_Inpainting_Technique_Based_on_the_Fast_Marching_Method


use crate::{
    constants, 
    path, 
    error, 
    enums, 
    imagebuffer::ImageBuffer, 
    vprintln,
    not_implemented,
    stats
};

#[derive(Debug, Clone)]
struct Point {
    x: usize,
    y: usize,
    score: u32
}

fn determine_mask_file(instrument:enums::Instrument) -> error::Result<&'static str> {
    match instrument {
        enums::Instrument::MslMAHLI => 
                    Ok(constants::cal::MSL_MAHLI_INPAINT_MASK_PATH),
        enums::Instrument::M20MastcamZLeft => 
                    Ok(constants::cal::M20_INPAINT_MASK_LEFT_PATH),
        enums::Instrument::M20MastcamZRight =>
                    Ok(constants::cal::M20_INPAINT_MASK_RIGHT_PATH),
        enums::Instrument::MslNavCamRight =>
                    Ok(constants::cal::MSL_NCAM_RIGHT_INPAINT_PATH),
        enums::Instrument::MslMastcamLeft =>
                    Ok(constants::cal::MSL_MCAM_LEFT_INPAINT_PATH),
        enums::Instrument::M20Watson =>
                    Ok(constants::cal::M20_WATSON_INPAINT_MASK_PATH),
        _ => Err(constants::status::UNSUPPORTED_INSTRUMENT)
    }
}

pub fn inpaint_supported_for_instrument(instrument:enums::Instrument) -> bool {
    let r = determine_mask_file(instrument);
    match r {
        Ok(_) => true,
        Err(_) => false
    }
}

fn load_mask_file(filename:&str, instrument:enums::Instrument) -> error::Result<ImageBuffer> {
    vprintln!("Loading inpaint mask file {}", filename);

    if ! path::file_exists(filename) {
        return Err(constants::status::FILE_NOT_FOUND);
    }
    let mask = ImageBuffer::from_file(filename).unwrap();
    
    match instrument {
        enums::Instrument::MslMAHLI => Ok(mask.get_subframe(32, 16, 1584, 1184).unwrap()),
        _ => Ok(mask)
    }
}

fn load_mask(instrument:enums::Instrument) -> error::Result<ImageBuffer> {
    let mask_file = determine_mask_file(instrument).unwrap();
    load_mask_file(mask_file, instrument)
}

fn get_num_good_neighbors(mask:&ImageBuffer, x:i32, y:i32) -> u32 {

    // Juggling the possibility of negitive numbers and whether or now we allow that.
    let t = if y > 0 { mask.get(x as usize, (y-1) as usize).unwrap() > 0.0 } else { false };
    let tl = if x > 0 && y > 0 { mask.get((x-1) as usize, (y-1) as usize).unwrap() > 0.0 } else { false };
    let l = if x > 0 { mask.get((x-1)  as usize, y as usize).unwrap() > 0.0 } else { false };
    let bl = if x > 0 && y < mask.height as i32 - 1 { mask.get((x-1) as usize, (y+1) as usize).unwrap() > 0.0 } else { false };
    let b = if y < mask.height as i32 - 1 { mask.get(x as usize, (y+1) as usize).unwrap() > 0.0 } else { false };
    let br = if x < mask.width as i32 - 1 && y < mask.height as i32 - 1 { mask.get((x+1) as usize, (y+1) as usize).unwrap() > 0.0 } else { false };
    let r = if x < mask.width as i32 - 1 { mask.get((x+1) as usize, y as usize).unwrap() > 0.0 } else { false };
    let tr = if x < mask.width as i32 - 1 && y > 0 { mask.get((x+1) as usize, (y-1) as usize).unwrap() > 0.0 } else { false };

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
            let v = mask.get(x, y).unwrap();
            if v > 0.0 {
                return Some(Point{x:x, y:y, score:0});
            }
        }
    }
    None
}

fn isolate_window(buffer:&ImageBuffer, mask:&ImageBuffer, window_size:i32, x:usize, y:usize) -> error::Result<Vec<f32>> {
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
                v.push(buffer.get(get_x as usize, get_y as usize).unwrap());
            }
        }
    }
    Ok(v)
}

fn predict_value(buffer:&ImageBuffer, mask:&ImageBuffer, x:usize, y:usize) -> f32 {
    let window = isolate_window(&buffer, &mask, 3, x, y).unwrap();
    let m = stats::mean(&window[0..]).unwrap();
    m
}


fn get_point_and_score_at_xy(mask:&ImageBuffer, x:i32, y:i32) -> Option<Point> {

    if x < 0 || x >= mask.width as i32 || y < 0 || y > mask.height as i32 {
        return None;
    }

    let v = mask.get(x as usize, y as usize).unwrap();
    if v == 0.0 {
        return None;
    }

    let score = get_num_good_neighbors(&mask, x, y);

    Some(Point{x:x as usize, y:y as usize, score:score})
}


fn find_larger(left:Option<Point>, right:&Point) -> Option<Point> {
    match left {
        Some(pt) => {
            let m = if pt.score > right.score { pt } else { right.clone() };
            Some(m)
        },
        None => return Some(right.to_owned())
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
                largest_score = find_larger(largest_score, pt);
            },
            None => ()
        }
    }

    largest_score
}


fn infill(buffer:&mut ImageBuffer, mask:&mut ImageBuffer, starting:&Point) {

    let mut current = starting.to_owned();
    loop {
        vprintln!("Filling in pixel at {}, {}", current.x, current.y);
        let pt_new_value = predict_value(&buffer, &mask, current.x, current.y);
        buffer.put(current.x, current.y, pt_new_value).unwrap();
        mask.put(current.x, current.y, 0.0).unwrap();

        match find_next_point(&mask, current.x as i32, current.y as i32) {
            Some(pt) => current = pt.to_owned(),
            None => break
        }
    }
}

// Embarrassingly slow and inefficient.
pub fn apply_inpaint_to_buffer(buffer:&ImageBuffer, instrument:enums::Instrument) -> error::Result<ImageBuffer> {

    let mut working_buffer = buffer.clone();
    let mut mask = load_mask(instrument).unwrap();

    // Crop the mask image if it's larger than the input image. 
    // Sizes need to match
    if mask.width > buffer.width {
        let x = (mask.width - buffer.width) / 2;
        let y = (mask.height - buffer.width) / 2;
        vprintln!("Cropping inpaint mask with params {}, {}, {}, {}", x, y, buffer.width, buffer.height);
        mask = mask.get_subframe(x, y, buffer.width, buffer.height).unwrap();
    }

    // For this to work, we need the mask to be mutable and we're
    // going to fill it in with 0x0 values as we go. If we don't, then
    // we'll keep finding starting points and this will be an infinite
    // loop. Which is bad. Perhaps consider an alternate method here.
    loop {
        match find_starting_point(&mask) {
            Some(pt) => {
                vprintln!("Starting point: {}, {}", pt.x, pt.y);
                infill(&mut working_buffer, &mut mask, &pt);
            },
            None => break
        };
    }

    Ok(working_buffer)
}


// pub fn apply_inpaint_to_rgb_array(rgb:Vec<[f32; 3]>, instrument:enums::Instrument)  {

    
// }