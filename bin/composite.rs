use mars_raw_utils::prelude::*;
use sciimg::{
    prelude::*,
    vector::Vector,
    min,
    max
};

#[macro_use]
extern crate clap;
use clap::{Arg, App};
use std::process;


#[derive(Debug, Clone)]
struct Point {
    pub x:f64,
    pub y:f64,
    pub r:f64,
    pub g:f64,
    pub b:f64
}

impl Point{
    pub fn create(x:f64, y:f64, r:f64, g:f64, b:f64) -> Self {


        Point{
            x:x,
            y:y,
            r:r,
            g:g,
            b:b
        }
    }
}

struct Triangle {
    pub p0:Point,
    pub p1:Point,
    pub p2:Point
}

impl Triangle {

    pub fn contains(&self, x:f64, y:f64) -> bool {
        let p = Point{x:x, y:y, r:0.0, g:0.0, b:0.0};
        let b0 = Triangle::sign(&p, &self.p0, &self.p1) <= 0.0;
        let b1 = Triangle::sign(&p, &self.p1, &self.p2) <= 0.0;
        let b2 = Triangle::sign(&p, &self.p2, &self.p0) <= 0.0;

        (b0 == b1) && (b1 == b2)
    }

    pub fn sign(p0:&Point, p1:&Point, p2:&Point) -> f64 {
        (p0.x - p2.x) * (p1.y - p2.y) - (p1.x - p2.x) * (p0.y - p2.y)
    }

    pub fn x_min(&self) -> f64 {
        min!(self.p0.x, self.p1.x, self.p2.x)
    }

    pub fn x_max(&self) -> f64 {
        max!(self.p0.x, self.p1.x, self.p2.x)
    }

    pub fn y_min(&self) -> f64 {
        min!(self.p0.y, self.p1.y, self.p2.y)
    }

    pub fn y_max(&self) -> f64 {
        max!(self.p0.y, self.p1.y, self.p2.y)
    }

    pub fn interpolate_color_channel(&self, x:f64, y:f64, c0:f64, c1:f64, c2:f64) -> f64 {
        let det = self.p0.x * self.p1.y - self.p1.x * self.p0.y + self.p1.x * self.p2.y - self.p2.x * self.p1.y + self.p2.x * self.p0.y - self.p0.x * self.p2.y;
        let a = ((self.p1.y-self.p2.y)*c0+(self.p2.y-self.p0.y)*c1+(self.p0.y-self.p1.y)*c2) / det;
        let b = ((self.p2.x-self.p1.x)*c0+(self.p0.x-self.p2.x)*c1+(self.p1.x-self.p0.x)*c2) / det;
        let c = ((self.p1.x*self.p2.y-self.p2.x*self.p1.y)*c0+(self.p2.x*self.p0.y-self.p0.x*self.p2.y)*c1+(self.p0.x*self.p1.y-self.p1.x*self.p0.y)*c2) / det;

        let v = a*x+b*y+c;
        v
    }

    pub fn interpolate_color(&self, x:f64, y:f64) -> (f64, f64, f64) {
        let r = self.interpolate_color_channel(x, y, self.p0.r, self.p1.r, self.p2.r);
        let g = self.interpolate_color_channel(x, y, self.p0.g, self.p1.g, self.p2.g);
        let b = self.interpolate_color_channel(x, y, self.p0.b, self.p1.b, self.p2.b);
        (r, g, b)
    }

}

struct Map {
    pub width:usize,
    pub height:usize,
    pub img_r:ImageBuffer,
    pub img_g:ImageBuffer,
    pub img_b:ImageBuffer
}

impl Map {
    pub fn create(width:usize, height:usize) -> Self {
        Map {
            width:width,
            height:height,
            img_r: ImageBuffer::new_with_fill_as_mode(width, height, 0.0, ImageMode::U16BIT).unwrap(),
            img_g: ImageBuffer::new_with_fill_as_mode(width, height, 0.0, ImageMode::U16BIT).unwrap(),
            img_b: ImageBuffer::new_with_fill_as_mode(width, height, 0.0, ImageMode::U16BIT).unwrap()
        }
    }

    pub fn to_rgbimage(&self) -> RgbImage {
        RgbImage::new_from_buffers_rgb(&self.img_r, &self.img_g, &self.img_b, ImageMode::U16BIT).unwrap()
    }

    pub fn paint_tri(&mut self, tri:&Triangle, avg_pixels:bool) {

        let min_x = tri.x_min().floor() as usize;
        let max_x = tri.x_max().ceil() as usize;
        let min_y = tri.y_min().floor() as usize;
        let max_y = tri.y_max().ceil() as usize;

        // Gonna limit the max dimension of a poly to just 100x100 
        // to prevent those that wrap the entire image. 
        // Until I plan out a better control to handle polygons that
        // wrap the cut-off azimuth
        if max_x - min_x < 100 && max_y - min_y <  100 {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if x < self.width && y < self.height && tri.contains(x as f64, y as f64) {
                        let (mut r, mut g, mut b) = tri.interpolate_color(x as f64,y as f64);
                        if r > 0.0 && g > 0.0 && b > 0.0 {

                            let r0 = self.img_r.get(x, y).unwrap() as f64;
                            let g0 = self.img_g.get(x, y).unwrap() as f64;
                            let b0 = self.img_b.get(x, y).unwrap() as f64;

                            if avg_pixels && (r0 > 0.0 || g0 > 0.0 || b0 > 0.0) {
                                r = (r + r0) / 2.0;
                                g = (g + g0) / 2.0;
                                b = (b + b0) / 2.0;
                            }

                            self.img_r.put(x, y, r as f32);
                            self.img_g.put(x, y, g as f32);
                            self.img_b.put(x, y, b as f32);
                        }
                        
                    }
                }
            }
        }

        

    }

    pub fn paint_square(&mut self, tl:&Point, bl:&Point, br:&Point, tr:&Point, avg_pixels:bool) {
        self.paint_tri(&Triangle {
            p0: tl.clone(),
            p1: bl.clone(),
            p2: tr.clone()
        }, avg_pixels);
        self.paint_tri(&Triangle {
            p0: tr.clone(),
            p1: bl.clone(),
            p2: br.clone()
        }, avg_pixels);
    }
}


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

fn get_ls_from_map_xy(model:&CameraModel, map_context:&MapContext, x:usize, y:usize) -> (f64, f64) {
    let img_x = x as f64;
    let img_y = y as f64;


    let lv = match model.ls_to_look_vector(&ImageCoordinate{ line:img_y, sample: img_x }) {
        Ok(lv) => lv,
        Err(_) => panic!("Unable to convert ls to look vector")
    };

    let ll = lookvector_to_cylindrical(&lv);
    let lat = ll.lat;
    let lon = ll.lon;
    
    let out_y_f = (lat - map_context.bottom_lat) / (map_context.top_lat - map_context.bottom_lat) * map_context.height as f64;
    let out_x_f = (lon - map_context.left_lon) / (map_context.right_lon - map_context.left_lon) * map_context.width as f64;

    (out_x_f, out_y_f)
}

fn process_file(input_file:&str, map_context:&MapContext, map:&mut Map) {

    let img = MarsImage::open(String::from(input_file), Instrument::M20MastcamZLeft);

    match get_cahvor(&img) {
        Some(input_model) => {

            println!("");
            vprintln!("Input Model C: {:?}", input_model.c());
            vprintln!("Input Model A: {:?}", input_model.a());
            vprintln!("Input Model H: {:?}", input_model.h());
            vprintln!("Input Model V: {:?}", input_model.v());
            vprintln!("Input Model O: {:?}", input_model.o());
            vprintln!("Input Model R: {:?}", input_model.r());
            vprintln!("Input Model E: {:?}", input_model.e());
            println!("");

            for x in 0..(img.image.width - 1) {
                for y in 0..(img.image.height - 1) {
                    
                    let (tl_x, tl_y) = get_ls_from_map_xy(&input_model, &map_context, x, y);
                    let (tr_x, tr_y) = get_ls_from_map_xy(&input_model, &map_context, x+1, y);
                    let (bl_x, bl_y) = get_ls_from_map_xy(&input_model, &map_context, x, y+1);
                    let (br_x, br_y) = get_ls_from_map_xy(&input_model, &map_context, x+1, y+1);

                    let tl = Point::create(
                        tl_x,
                        tl_y,
                        img.image.get_band(0).get(x, y).unwrap() as f64,
                        img.image.get_band(1).get(x, y).unwrap() as f64,
                        img.image.get_band(2).get(x, y).unwrap() as f64
                    );

                    let tr = Point::create(
                        tr_x,
                        tr_y,
                        img.image.get_band(0).get(x+1, y).unwrap() as f64,
                        img.image.get_band(1).get(x+1, y).unwrap() as f64,
                        img.image.get_band(2).get(x+1, y).unwrap() as f64
                    );

                    let bl = Point::create(
                        bl_x,
                        bl_y,
                        img.image.get_band(0).get(x, y+1).unwrap() as f64,
                        img.image.get_band(1).get(x, y+1).unwrap() as f64,
                        img.image.get_band(2).get(x, y+1).unwrap() as f64
                    );

                    let br = Point::create(
                        br_x,
                        br_y,
                        img.image.get_band(0).get(x+1, y+1).unwrap() as f64,
                        img.image.get_band(1).get(x+1, y+1).unwrap() as f64,
                        img.image.get_band(2).get(x+1, y+1).unwrap() as f64
                    );


                    map.paint_square(&tl, &bl, &br, &tr, false);
                }

            }

            //let output_model = input_model.linearize(img.image.width, img.image.height, img.image.width, img.image.height).unwrap();
            // vprintln!("output model: {:?}", output_model);
            // println!("");

            // let ground = Vector::new(0.0,0.0,1.84566);
            // let z = Vector::new(0.0, 0.0, -1.0);
            // let mut min_angle = 1000000.0;
            // let mut max_angle = -1000000.0;

            // for y in 0..map_context.height {
            //     for x in 0..map_context.width {

            //         if let Ok(lv) = output_model.ls_to_look_vector(&ImageCoordinate{line: y as f64, sample: x as f64}) {
                        
            //             //vprintln!("lv -> {:?}", lv.look_direction);
            //             let ray = intersect_to_plane(&lv, &ground);
            //             //vprintln!("ray -> {:?} -- {}", ray, ray.len());
            //             min_angle = min!(z.angle(&ray).to_degrees(), min_angle);
            //             max_angle = max!(z.angle(&ray).to_degrees(), max_angle);
            //             let ls_in = input_model.xyz_to_ls(&ray, false);
                        

            //             let in_x = ls_in.sample.round() as usize;
            //             let in_y = ls_in.line.round() as usize;
            //             //vprintln!("{}, {} -> Line: {}, Sample: {}", y, x, ls_in.line, ls_in.sample);


            //             if in_x < img.image.width && in_y < img.image.height {
            //                 map_r.put(x, y, img.image.get_band(0).get(in_x, in_y).unwrap());
            //                 map_g.put(x, y, img.image.get_band(1).get(in_x, in_y).unwrap());
            //                 map_b.put(x, y, img.image.get_band(2).get(in_x, in_y).unwrap());
            //             }
            //         }

            //     }
            // }
            
            // vprintln!("Min/Max angles: {}, {}", min_angle, max_angle);
            
                   
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

    if map_context.width == 0 {
        eprintln!("Output expected to have zero width. Cannot continue with that. Exiting...");
        process::exit(1);
    } else if map_context.height == 0 {
        eprintln!("Output expected to have zero height. Cannot continue with that. Exiting...");
        process::exit(1);
    }
    // let map_context = MapContext{
    //     top_lat : -90.0,
    //     bottom_lat : 90.0,
    //     left_lon: 360.0,
    //     right_lon: -360.0,
    //     width: 1024,
    //     height: 1024,
    //     degrees_per_pixel: 0.0
    // };
    // let mut map_r = ImageBuffer::new_with_fill_as_mode(map_context.width, map_context.height, 100.0, ImageMode::U16BIT).unwrap();
    // let mut map_g = ImageBuffer::new_with_fill_as_mode(map_context.width, map_context.height, 0.0, ImageMode::U16BIT).unwrap();
    // let mut map_b = ImageBuffer::new_with_fill_as_mode(map_context.width, map_context.height, 0.0, ImageMode::U16BIT).unwrap();

    let mut map = Map::create(map_context.width, map_context.height);

    for in_file in input_files.iter() {
        if path::file_exists(in_file) {
            vprintln!("Processing File: {}", in_file);
            process_file(in_file, &map_context, &mut map);
        } else {
            eprintln!("File not found: {}", in_file);
            process::exit(1);
        }
    }


    //let mut out_img = RgbImage::new_from_buffers_rgb(&map_r, &map_g, &map_b, ImageMode::U16BIT).unwrap();
    let mut out_img = map.to_rgbimage();
    out_img.normalize_to_16bit_with_max(255.0);
    out_img.save(output);
    // map_r.normalize_mut(0.0, 65535.0);
    // map_r.save("test.png", ImageMode::U16BIT);
}