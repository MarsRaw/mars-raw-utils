use crate::prelude::*;
use sciimg::{
    prelude::*,
    vector::Vector,
    min,
    max,
    quaternion::Quaternion
};



pub fn get_cahvor(img:&MarsImage) -> Option<CameraModel> {
    match &img.metadata {
        Some(md) => {
            if md.camera_model_component_list.is_valid() {
                Some(md.camera_model_component_list.clone())
            } else {
                None
            }
        },
        None => {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MapContext {
    pub top_lat:f64,
    pub bottom_lat:f64,
    pub left_lon:f64,
    pub right_lon:f64,
    pub width:usize,
    pub height:usize,
    pub degrees_per_pixel:f64
}

pub struct LatLon{
    lat:f64,
    lon:f64
}



fn vector_to_cylindrical(v:&Vector) -> LatLon {
    LatLon{
        lat:v.z.atan2((v.x * v.x + v.y * v.y).sqrt()).to_degrees(),
        lon:v.y.atan2(v.x).to_degrees() + 180.0
    }
}

fn lookvector_to_cylindrical(lv:&LookVector, quat_o:Option<&Quaternion>, origin_diff:&Vector) -> LatLon {
    let ray = lv.intersect_to_sphere(SPHERE_RADIUS);
    let ray_moved = ray.subtract(&origin_diff);
    let rotated = if let Some(quat) = quat_o {
        quat.rotate_vector(&ray_moved)
    } else {
        ray
    };
    vector_to_cylindrical(&rotated)
}

static SPHERE_RADIUS:f64 = 100.0;

fn get_lat_lon(c:&CameraModel, x:usize, y:usize, quat:&Quaternion) -> error::Result<LatLon> {
    match c.ls_to_look_vector(&ImageCoordinate{ line:y as f64, sample:x as f64 }) {
        Ok(lv) => {
            Ok(lookvector_to_cylindrical(&lv, None, &Vector::default()))
        },
        Err(e) => {
            Err(e)
        }
    }
}


pub fn determine_map_context(input_files:&Vec<String>, quat:&Quaternion) -> MapContext {
    let mut context = MapContext{
        top_lat : -90.0,
        bottom_lat : 90.0,
        left_lon: 360.0,
        right_lon: -360.0,
        width: 0,
        height: 0,
        degrees_per_pixel: 0.0
    };

    input_files.iter().for_each(|input_file| {
        let img = MarsImage::open(input_file.to_owned(), Instrument::M20MastcamZLeft);
        match get_cahvor(&img) {
            Some(c) => {
                match get_lat_lon(&c, 0, 0, &quat) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, img.image.width, 0, &quat) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, 0, img.image.height, &quat) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, img.image.width, img.image.height, &quat) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, img.image.width / 2, 0, &quat) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, img.image.width / 2, img.image.height, &quat) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                let ang_horiz = c.pixel_angle_horiz().to_degrees();
                context.degrees_per_pixel = max!(context.degrees_per_pixel, ang_horiz);

            },
            None => {}
        };

    });

    context.top_lat = min!(context.top_lat, 90.0);
    context.bottom_lat = max!(context.bottom_lat, -90.0);
    context.left_lon = max!(context.left_lon, 0.0);
    context.right_lon = min!(context.right_lon, 360.0);
    context.height = ((context.top_lat - context.bottom_lat) / context.degrees_per_pixel).floor() as usize;
    context.width = ((context.right_lon - context.left_lon) / context.degrees_per_pixel).floor() as usize;

    context
}

fn get_ls_from_map_xy(model:&CameraModel, map_context:&MapContext, x:usize, y:usize, quat:&Quaternion, origin_diff:&Vector) -> (f64, f64) {
    let img_x = x as f64;
    let img_y = y as f64;


    let lv = match model.ls_to_look_vector(&ImageCoordinate{ line:img_y, sample: img_x }) {
        Ok(lv) => lv,
        Err(_) => panic!("Unable to convert ls to look vector")
    };

    

    let ll = lookvector_to_cylindrical(&lv, Some(&quat), origin_diff);
    let lat = ll.lat;
    let lon = ll.lon;
    
    let out_y_f = (lat - map_context.bottom_lat) / (map_context.top_lat - map_context.bottom_lat) * map_context.height as f64;
    let out_x_f = (lon - map_context.left_lon) / (map_context.right_lon - map_context.left_lon) * map_context.width as f64;

    (out_x_f, out_y_f)
}

pub fn process_file<D:Drawable>(input_file:&str, map_context:&MapContext, map:&mut D, anaglyph:bool, quat:&Quaternion, initial_origin:&Vector) {

    let mut img = MarsImage::open(String::from(input_file), Instrument::M20MastcamZLeft);
    img.instrument = match &img.metadata {
        Some(md) => Instrument::from_str(md.instrument.as_str()),
        None => Instrument::M20MastcamZLeft
    };

    let eye = if anaglyph {
        match util::filename_char_at_pos(&input_file, 1) {
            'R' => Eye::Right,
            'L' => Eye::Left,
            _ => Eye::DontCare
        }
    } else {
        Eye::DontCare
    };

    //let pitch = (map_context.top_lat + map_context.bottom_lat) / 2.0;
    //vprintln!("Rotating pitch to {}", pitch);
    //let quat = Quaternion::from_pitch_roll_yaw(0.0, -20.0f64.to_radians(), azimuth_rotation.to_radians());


    match get_cahvor(&img) {
        Some(input_model) => {

            vprintln!("");
            vprintln!("Input Model C: {:?}", input_model.c());
            vprintln!("Input Model A: {:?}", input_model.a());
            vprintln!("Input Model H: {:?}", input_model.h());
            vprintln!("Input Model V: {:?}", input_model.v());
            vprintln!("Input Model O: {:?}", input_model.o());
            vprintln!("Input Model R: {:?}", input_model.r());
            vprintln!("Input Model E: {:?}", input_model.e());
            vprintln!("");

            let band_0 = img.image.get_band(0);
            let band_1 =  img.image.get_band(1);
            let band_2 =  img.image.get_band(2);

            for x in 0..(img.image.width - 1) {
                for y in 0..(img.image.height - 1) {
                    
                    let origin_diff = input_model.c().subtract(&initial_origin);

                    let (tl_x, tl_y) = get_ls_from_map_xy(&input_model, &map_context, x, y, &quat, &origin_diff);
                    let (tr_x, tr_y) = get_ls_from_map_xy(&input_model, &map_context, x+1, y, &quat, &origin_diff);
                    let (bl_x, bl_y) = get_ls_from_map_xy(&input_model, &map_context, x, y+1, &quat, &origin_diff);
                    let (br_x, br_y) = get_ls_from_map_xy(&input_model, &map_context, x+1, y+1, &quat, &origin_diff);
                    
                    if ! band_0.get_mask_at_point(x, y).unwrap() {
                        continue;
                    }
                    

                    let tl = Point::create(
                        tl_x,
                        tl_y,
                        band_0.get(x, y).unwrap() as f64,
                        band_1.get(x, y).unwrap() as f64,
                        band_2.get(x, y).unwrap() as f64
                    );

                    let tr = Point::create(
                        tr_x,
                        tr_y,
                        band_0.get(x+1, y).unwrap() as f64,
                        band_1.get(x+1, y).unwrap() as f64,
                        band_2.get(x+1, y).unwrap() as f64
                    );

                    let bl = Point::create(
                        bl_x,
                        bl_y,
                        band_0.get(x, y+1).unwrap() as f64,
                        band_1.get(x, y+1).unwrap() as f64,
                        band_2.get(x, y+1).unwrap() as f64
                    );

                    let br = Point::create(
                        br_x,
                        br_y,
                        band_0.get(x+1, y+1).unwrap() as f64,
                        band_1.get(x+1, y+1).unwrap() as f64,
                        band_2.get(x+1, y+1).unwrap() as f64
                    );


                    map.paint_square(&tl, &bl, &br, &tr, false, eye);
                }

            }
                   
        },
        None => {
            eprintln!("CAHVOR not found for image, cannot continue");
            panic!("CAHVOR not found for image, cannot continue");
        }
    }


}
