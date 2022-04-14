use mars_raw_utils::prelude::*;
use sciimg::{
    prelude::*,
    cahvor::*,
    vector::Vector//,
    // matrix::Matrix,
    // enums::Axis,
    // quaternion::Quaternion
};
#[macro_use]
extern crate clap;
use clap::{Arg, App};
use std::process;


fn get_cahvor(img:&MarsImage) -> Option<Cahvor> {
    match &img.metadata {
        Some(md) => {
            match &md.camera_model_component_list {
                Some(c) => Some(c.clone()),
                None => None
            }
        },
        None => {
            None
        }
    }
}

fn get_az(img:&MarsImage) -> f64 {
    match &img.metadata {
        Some(md) => {
            match &md.mast_az {
                Some(az) => az.clone(),
                None => 0.0
            }
        },
        None => {
            0.0
        }
    }
}

fn get_el(img:&MarsImage) -> f64 {
    match &img.metadata {
        Some(md) => {
            match &md.mast_el {
                Some(el) => el.clone(),
                None => 0.0
            }
        },
        None => {
            0.0
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct MapContext {
    pub top_lat:f64,
    pub bottom_lat:f64,
    pub left_lon:f64,
    pub right_lon:f64,
    pub width:usize,
    pub height:usize,
    pub degrees_per_pixel:f64
}

struct LatLon{
    lat:f64,
    lon:f64
}

fn intersect_to_sphere(lv:&LookVector, radius:f64) -> Vector {
    lv.look_direction.normalized().scale(radius).add(&lv.origin)
}

static SPHERE_RADIUS:f64 = 1000.0;

fn get_lat_lon(c:&Cahvor, x:usize, y:usize) -> error::Result<LatLon> {
    match c.ls_to_look_vector(&ImageCoordinate{ line:y as f64, sample:x as f64 }) {
        Ok(lv) => {
            let ray = intersect_to_sphere(&lv, SPHERE_RADIUS);

            let lat = ray.z.atan2((ray.x * ray.x + ray.y * ray.y).sqrt()).to_degrees();
            let lon =  ray.y.atan2(ray.x).to_degrees() + 180.0;

            Ok(LatLon{
                lat:lat,
                lon:lon
            })
        },
        Err(e) => {
            Err(e)
        }
    }
}



fn determine_map_context(input_files:&Vec<&str>) -> MapContext {
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
        let img = MarsImage::open(String::from(*input_file), Instrument::M20MastcamZLeft);
        match get_cahvor(&img) {
            Some(c) => {
                // let mast_az = get_az(&img);
                // let mast_el = get_el(&img);

                // let line = c.a.dot_product(&c.v);
                // let sample = c.a.dot_product(&c.h);

                // let center_x = img.image.width as f64 / 2.0 + sample;
                // let center_y = img.image.height as f64 / 2.0 + line;

                match get_lat_lon(&c, 0, 0) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, img.image.width, 0) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, 0, img.image.height) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, img.image.width, img.image.height) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, img.image.width / 2, 0) {
                    Ok(ll) => {
                        context.bottom_lat = min!(context.bottom_lat, ll.lat );
                        context.top_lat = max!(context.top_lat, ll.lat);
                        context.left_lon = min!(context.left_lon, ll.lon);
                        context.right_lon = max!(context.right_lon, ll.lon);
                    },
                    Err(_) => {}
                };

                match get_lat_lon(&c, img.image.width / 2, img.image.height) {
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



fn process_file(input_file:&str, map_context:&MapContext, map_r:&mut ImageBuffer, map_g:&mut ImageBuffer, map_b:&mut ImageBuffer) {

    let img = MarsImage::open(String::from(input_file), Instrument::M20MastcamZLeft);

    match get_cahvor(&img) {
        Some(c) => {
            vprintln!("CAHVOR: {:?}", c);
            let center_az = get_az(&img);
            let center_el = get_el(&img);
            vprintln!("Mast Az/El: {}/{}", center_az, center_el);
            
        

            for x in 0..img.image.width {
                for y in 0..img.image.height {

                    match c.ls_to_look_vector(&ImageCoordinate{ line:y as f64, sample: x as f64 }) {
                        Ok(lv) => {
                            let ray = intersect_to_sphere(&lv, SPHERE_RADIUS);
                            
                            let lat = ray.z.atan2((ray.x * ray.x + ray.y * ray.y).sqrt()).to_degrees();
                            let lon = ray.y.atan2(ray.x).to_degrees() + 180.0;

                            let out_y_f = (lat - map_context.bottom_lat) / (map_context.top_lat - map_context.bottom_lat) * map_context.height as f64;
                            let out_x_f = (lon - map_context.left_lon) / (map_context.right_lon - map_context.left_lon) * map_context.width as f64;

                            let out_x = out_x_f.round() as usize;
                            let out_y = out_y_f.round() as usize;

                            if out_x < map_r.width && out_y < map_r.height {
                                map_r.put(out_x, out_y, img.image.get_band(0).get(x, y).unwrap());
                                map_g.put(out_x, out_y, img.image.get_band(1).get(x, y).unwrap());
                                map_b.put(out_x, out_y, img.image.get_band(2).get(x, y).unwrap());
                            }
                            
                        },
                        Err(_) => {}
                    };

                }

            }

            // let min_x = ((360.0 * size_mult as f64) - ((360.0 - (center_az - fov_horiz)) * size_mult as f64)) as usize;
            // let max_x = ((360.0 * size_mult as f64) - ((360.0 - (center_az + fov_horiz)) * size_mult as f64)) as usize;

            // let min_y = ((90.0 * size_mult as f64) - (center_el + fov_vert) * (size_mult as f64)) as usize;
            // let max_y = ((90.0 * size_mult as f64) - (center_el - fov_vert) * (size_mult as f64)) as usize;

            // vprintln!("X Range: {} - {}", min_x, max_x);
            // vprintln!("Y Range: {} - {}  -- {} - {}", min_y, max_y, (90.0 - min_y as f64 * size_mult_div), (90.0 - max_y as f64 * size_mult_div));
            // for x in min_x..max_x {
            //     let az = x as f64 * size_mult_div;

            //     for y in min_y..max_y {
            //     //for y in 0..720 {
            // //         //let y = 360;

            //         let el = 90.0 - y as f64 * size_mult_div;
            //         //println!("{}, {}   --  {}, {}", center_az, az, center_el, el);

            //         if (az - center_az).abs() < fov_horiz  && (el - center_el).abs() < fov_vert {
            //             let src_x_f = ((img.image.width as f64 / 2.0) + (az - center_az) / ang_horiz).floor();
            //             let src_y_f = ((img.image.height as f64 / 2.0) - (el - center_el) / ang_vert).floor();

            //             let src_x = src_x_f as usize;
            //             let src_y = src_y_f as usize;
            //             //println!("{}, {} -- {}, {} -- {}, {} -- {}, {}", center_az, center_el, az, el, src_x, src_y, ang_horiz, ang_vert);
            //             //vprintln!("X/Y: {}, {} -- {}, {}", x, y, src_x, src_y);
            //             if src_x_f >= 0.0 && src_x < img.image.width &&  src_y_f >= 0.0 && src_y < img.image.height {
            //                 map_r.put(x, y, img.image.get_band(0).get(src_x, src_y).unwrap());
            //                 map_g.put(x, y, img.image.get_band(1).get(src_x, src_y).unwrap());
            //                 map_b.put(x, y, img.image.get_band(2).get(src_x, src_y).unwrap());
            //             }
            //         }


            //     }
            // }


            // for j in 0..img.image.height { 
            //     for i in 0..img.image.width {

            //         match c.ls_to_look_vector(&ImageCoordinate{ sample:i as f64, line:j as f64 }) {
            //             Ok(lv) => {
            //                 let v = intersect_ray(&lv.origin, &lv.look_direction, 10000.0);
            //                 let ls = c.xyz_to_ls(&v, false);
            //                 vprintln!("Out: j, i: {}, {}  Src: j, i: {}, {}", j, i, ls.line.to_degrees(), ls.sample.to_degrees());
            //             },
            //             Err(_) => {}
            //         };

            //     }
            // }



    //         for x in 0..1440 {
    //             let az = x as f64 * 0.25;

    //             //for y in 380..400 {
    //             for y in 0..720 {
    //         //         //let y = 360;

    //                 let el = y as f64 * 0.25;

    //                 let mut mast_vec = Vector::z_axis_vector();
    //                 mast_vec = mast_vec.scale(1000000.0);

    //                 let q = Quaternion::from_pitch_roll_yaw(el.to_radians(), 0.0, az.to_radians());
    //                 mast_vec = q.rotate_vector(&mast_vec);

    //                 let pt = c.project_object_to_image_point(&mast_vec);
    //                 //let line = ;
    //                 //let samp = pt.i.to_degrees();

    //                 let ls = c.xyz_to_ls(&mast_vec, true);
    //                 match c.ls_to_look_vector(&ls) {
    //                     Ok(lv) => {
    //                         let v = intersect_ray(&lv.origin, &lv.look_direction, 10000.0);
    //                         vprintln!("Ray: {:?}", v);
    //                     },
    //                     Err(_) => {}
    //                 };

                    
    //                 // let line = ls.line.to_degrees();
    //                 // let samp = ls.sample.to_degrees();

    //                 // let hc = img.image.width / 2;
    //                 // let vc = img.image.height / 2;
    //                 // let x0 = hc - img.image.width / 2;
    //                 // let y0 = img.image.height / 2 - vc;

    //                 // let img_x = ((pt.i.to_degrees() - (img.image.width / 2) as f64) - x0 as f64).round() as usize;
    //                 // let img_y = (((img.image.height / 2) as f64 - pt.j.to_degrees()) - y0 as f64).round() as usize;

    //                 let img_x = (img.image.width as f64 / 2.0 + pt.i) as usize;
    //                 let img_y = (img.image.height as f64 / 2.0 + pt.j) as usize;

    //                 //vprintln!("L/S: {}, {}, {}", line, samp, img_x);
    //                 if  img_y < img.image.height && img_x < img.image.width  {

    //                     map_r.put(x, y, img.image.get_band(0).get(img_x, img_y).unwrap());
    //                     map_g.put(x, y, img.image.get_band(1).get(img_x, img_y).unwrap());
    //                     map_b.put(x, y, img.image.get_band(2).get(img_x, img_y).unwrap());
    //                 }

    //             }

    //         }
        },
        None => {
            eprintln!("CAHVOR not found for image, cannot continue");
            process::exit(2);
        }
    }

    

}

fn main() {
    let matches = App::new(crate_name!())
                    .version(crate_version!())
                    .author(crate_authors!())
                    .arg(Arg::with_name(constants::param::PARAM_INPUTS)
                        .short(constants::param::PARAM_INPUTS_SHORT)
                        .long(constants::param::PARAM_INPUTS)
                        .value_name("INPUT")
                        .help("Input")
                        .required(true)
                        .multiple(true)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_OUTPUT)
                        .short(constants::param::PARAM_OUTPUT_SHORT)
                        .long(constants::param::PARAM_OUTPUT)
                        .value_name("OUTPUT")
                        .help("Output")
                        .required(true)
                        .takes_value(true)) 
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let input_files: Vec<&str> = matches.values_of(constants::param::PARAM_INPUTS).unwrap().collect();
    let output = matches.value_of("output").unwrap();


    let map_context = determine_map_context(&input_files);
    vprintln!("Map Context: {:?}", map_context);
    vprintln!("FOV Vertical: {}", map_context.top_lat - map_context.bottom_lat);
    vprintln!("FOV Horizontal: {}", map_context.right_lon - map_context.left_lon);

    // let map_context = MapContext{
    //     top_lat : 90.0,
    //     bottom_lat : -90.0,
    //     left_lon : 0.0,
    //     right_lon : 360.0,
    //     degrees_per_pixel : 0.03125,
    //     height: 5760,
    //     width: 11520
    // };
    let mut map_r = ImageBuffer::new_with_fill_as_mode(map_context.width, map_context.height, 100.0, ImageMode::U16BIT).unwrap();
    let mut map_g = ImageBuffer::new_with_fill_as_mode(map_context.width, map_context.height, 0.0, ImageMode::U16BIT).unwrap();
    let mut map_b = ImageBuffer::new_with_fill_as_mode(map_context.width, map_context.height, 0.0, ImageMode::U16BIT).unwrap();

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file, &map_context, &mut map_r, &mut map_g, &mut map_b);
        } else {
            eprintln!("File not found: {}", in_file);
            process::exit(1);
        }
    }


    let mut out_img = RgbImage::new_from_buffers_rgb(&map_r, &map_g, &map_b, ImageMode::U16BIT).unwrap();
    out_img.normalize_to_16bit_with_max(255.0);
    out_img.save(output);
    // map_r.normalize_mut(0.0, 65535.0);
    // map_r.save("test.png", ImageMode::U16BIT);
}