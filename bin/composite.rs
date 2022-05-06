use mars_raw_utils::prelude::*;
use sciimg::{
    prelude::*,
    vector::Vector//,
    // matrix::Matrix,
    // enums::Axis,
    // quaternion::Quaternion
};
#[macro_use]
extern crate clap;
use clap::{Arg, App};
use std::process;


fn get_cahvor(img:&MarsImage) -> Option<CameraModel> {
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

fn intersect_to_plane(lv:&LookVector, ground:&Vector) -> Vector {
    let normal = Vector::new(0.0, 0.0, -1.0);
    

    let dot = lv.look_direction.dot_product(&normal);
    if dot == 0.0 {
        return lv.look_direction.clone();
    }

    let ratio = ground.subtract(&lv.origin).dot_product(&normal) / dot;

    let intersect_point = lv.origin.add(&lv.look_direction.scale(ratio));
    
    if ratio < 0.0 {
        lv.look_direction.clone()
    } else {
        intersect_point
    }

}

fn intersect_to_sphere(lv:&LookVector, radius:f64) -> Vector {
    lv.look_direction.normalized().scale(radius).add(&lv.origin)
}

fn vector_to_cylindrical(v:&Vector) -> LatLon {
    // let rho = (v.x * v.x + v.y * v.y).sqrt();
    // let theta = (v.y / v.x).atan().to_degrees();
    LatLon{
        lat:v.z.atan2((v.x * v.x + v.y * v.y).sqrt()).to_degrees(),
        lon:v.y.atan2(v.x).to_degrees() + 180.0
    }
}

fn lookvector_to_cylindrical(lv:&LookVector) -> LatLon {
    let ray = intersect_to_sphere(&lv, SPHERE_RADIUS);
    vector_to_cylindrical(&ray)
}

static SPHERE_RADIUS:f64 = 100.0;

fn get_lat_lon(c:&CameraModel, x:usize, y:usize) -> error::Result<LatLon> {
    match c.ls_to_look_vector(&ImageCoordinate{ line:y as f64, sample:x as f64 }) {
        Ok(lv) => {
            Ok(lookvector_to_cylindrical(&lv))
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
        Some(input_model) => {
            //vprintln!("CAHVOR: {:?}", c);
            let center_az = get_az(&img);
            let center_el = get_el(&img);
            vprintln!("Mast Az/El: {}/{}", center_az, center_el);
            
            println!("");
            vprintln!("Input Model C: {:?}", input_model.c());
            vprintln!("Input Model A: {:?}", input_model.a());
            vprintln!("Input Model H: {:?}", input_model.h());
            vprintln!("Input Model V: {:?}", input_model.v());
            vprintln!("Input Model O: {:?}", input_model.o());
            vprintln!("Input Model R: {:?}", input_model.r());
            println!("");
            //let output_model = input_model.linearize(img.image.width, img.image.height, map_r.width, map_r.height).unwrap();
            let output_model = Cahv{
                c: Vector::new(0.564241, 0.554952, -1.9218),
                a: Vector::new(-0.502824, 0.854567, 0.12987),
                h: Vector::new(-1314.41, -190.012, 64.2655),
                v: Vector::new(-177.008, 297.727, 1282.35)
            };
            
            vprintln!("output model: {:?}", output_model);
            println!("");

            let ground = Vector::new(0.0,0.0,1.84566);
            let z = Vector::new(0.0, 0.0, -1.0);
            let mut min_angle = 1000000.0;
            let mut max_angle = -1000000.0;

            for y in 0..map_context.height {
                for x in 0..map_context.width {

                    if let Ok(lv) = output_model.ls_to_look_vector(&ImageCoordinate{line: y as f64, sample: x as f64}) {
                        
                        //vprintln!("lv -> {:?}", lv.look_direction);
                        let ray = intersect_to_plane(&lv, &ground);
                        //vprintln!("ray -> {:?} -- {}", ray, ray.len());
                        min_angle = min!(z.angle(&ray).to_degrees(), min_angle);
                        max_angle = max!(z.angle(&ray).to_degrees(), max_angle);
                        let ls_in = input_model.xyz_to_ls(&ray, false);
                        

                        let in_x = ls_in.sample.round() as usize;
                        let in_y = ls_in.line.round() as usize;
                        //vprintln!("{}, {} -> Line: {}, Sample: {}", y, x, ls_in.line, ls_in.sample);


                        if in_x < img.image.width && in_y < img.image.height {
                            map_r.put(x, y, img.image.get_band(0).get(in_x, in_y).unwrap());
                            map_g.put(x, y, img.image.get_band(1).get(in_x, in_y).unwrap());
                            map_b.put(x, y, img.image.get_band(2).get(in_x, in_y).unwrap());
                        }
                    }

                }
            }
            
            vprintln!("Min/Max angles: {}, {}", min_angle, max_angle);

            /*
            for x in 0..img.image.width {
                for y in 0..img.image.height {
                    let lv = match output_model.ls_to_look_vector(&ImageCoordinate{ line:y as f64, sample: x as f64 }) {
                        Ok(lv) => lv,
                        Err(_) => continue
                    };

                    let ll = lookvector_to_cylindrical(&lv);
                    let lat = ll.lat;
                    let lon = ll.lon;
                    let out_y_f = (lat - map_context.bottom_lat) / (map_context.top_lat - map_context.bottom_lat) * map_context.height as f64;
                    let out_x_f = (lon - map_context.left_lon) / (map_context.right_lon - map_context.left_lon) * map_context.width as f64;

                    let out_x = out_x_f.round() as usize;
                    let out_y = out_y_f.round() as usize;



                    if out_x < map_r.width && out_y < map_r.height {
                        map_r.put(out_x, out_y, img.image.get_band(0).get(x, y).unwrap());
                        map_g.put(out_x, out_y, img.image.get_band(1).get(x, y).unwrap());
                        map_b.put(out_x, out_y, img.image.get_band(2).get(x, y).unwrap());
                    }
                }

            }
            */
                   
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


    // let map_context = determine_map_context(&input_files);
    // vprintln!("Map Context: {:?}", map_context);
    // vprintln!("FOV Vertical: {}", map_context.top_lat - map_context.bottom_lat);
    // vprintln!("FOV Horizontal: {}", map_context.right_lon - map_context.left_lon);

    // if map_context.width == 0 {
    //     eprintln!("Output expected to have zero width. Cannot continue with that. Exiting...");
    //     process::exit(1);
    // } else if map_context.height == 0 {
    //     eprintln!("Output expected to have zero height. Cannot continue with that. Exiting...");
    //     process::exit(1);
    // }
    let map_context = MapContext{
        top_lat : -90.0,
        bottom_lat : 90.0,
        left_lon: 360.0,
        right_lon: -360.0,
        width: 1024,
        height: 1024,
        degrees_per_pixel: 0.0
    };
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