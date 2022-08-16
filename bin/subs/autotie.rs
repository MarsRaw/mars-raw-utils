use mars_raw_utils::{
    prelude::*
};
use sciimg::{
    prelude::*
};
use sciimg::vector::Vector;
use sciimg::VecMath;
use crate::subs::runnable::RunnableSubcommand;

use std::process;

use colored::*;


#[derive(clap::Args)]
#[clap(author, version, about = "Attempt to auto generate tiepoints for stereo pair", long_about = None)]
pub struct Autotie {
    #[clap(long, short, help = "Input images", multiple_values(true))]
    inputs: Vec<String>,
}


pub fn lcross(b1:&Vector, v1:&Vector, b2:&Vector, v2:&Vector) -> (Vector, Vector, bool) {

    let b = b1.subtract(b2);
    let v1b = v1.dot_product(&b);
    let v2b = v2.dot_product(&b);
    let v1v1 = v1.dot_product(&v1);
    let v2v2 = v2.dot_product(&v2);
    let v1v2 = v1.dot_product(&v2);

    let epsilon = 1.0e-20;
    let denom = v1v2 * v1v2 - v1v1 * v2v2;
    if denom < epsilon && denom > -epsilon {
        (Vector::default(), Vector::default(), true)
    } else {
        let k2 = (v1b * v1v2 - v2b * v1v1) / denom;
        let k1 = (k2 * v1v2 - v1b) / v1v1;

        let p1 = v1.scale(k1);
        let p2 = v2.scale(k2);

        let p1f = b1.add(&p1);
        let p2f = b2.add(&p2);

        (p1f, p2f, false)
    }
}


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

impl RunnableSubcommand for Autotie {
    fn run(&self) {
        print::print_experimental();

        if self.inputs.len() != 2 {
            eprintln!("{} Please provide only two files (for now)", "Error:".red());
            process::exit(1);
        }

        self.inputs.iter().for_each(|f| {
            if ! path::file_exists(&f) {
                eprintln!("{} File not found: {}", "Error:".red(), f);
                process::exit(2);
            }
        });

        
        let kernel_size = 16;
        let edge = kernel_size / 2 + 1;
        // 361, 620

        let m0 = MarsImage::open(self.inputs[0].to_owned(), Instrument::MslNavCamLeft);
        let m1 = MarsImage::open(self.inputs[1].to_owned(), Instrument::MslNavCamRight);

        //let f0 = m0.image;
        //let f1 = m1.image;

        let cav0 = match get_cahvor(&m0) {
            Some(c) => c,
            None => {
                eprintln!("Camera model not found for image 1. Cannot continue");
                process::exit(1);
            }
        };

        let cav1 = match get_cahvor(&m1) {
            Some(c) => c,
            None => {
                eprintln!("Camera model not found for image 2. Cannot continue");
                process::exit(1);
            }
        };

        let b1 = m1.image.get_band(0);

        for x0 in (300..700).step_by(10) {
            for y0 in (300..700).step_by(10) {
                let w0 = m0.image.get_band(0).isolate_window(kernel_size, x0, y0);

                let mut closest_x = 0;
                let mut closest_y = 0;
                let mut closest_sigma = 0.0;

                let min_y = y0 - 100;
                let max_y = y0 + 100;
                let min_x = x0 - 250;
                let max_x = x0 + 250;
                for y in edge..(m1.image.height - edge) {
                    //println!("Row {} of {}. {}%", y, m1.image.height, (y as f32 / m1.image.height as f32 * 100.0));
                    
                    for x in edge..(m1.image.width - edge) {
                        
                        let w1 = b1.isolate_window(kernel_size, x, y);
        
                        let c = w0.xcorr(&w1);
                        if c.abs() > closest_sigma {
                            closest_sigma = c.abs();
                            closest_x = x;
                            closest_y = y;
                        }
        
                    }
                }

                //println!("Closest X/Y: {}, {}", closest_x, closest_y);
                let lv0 = cav0.ls_to_look_vector(&ImageCoordinate{ line:y0 as f64, sample:x0 as f64 as f64 }).unwrap();
                let lv1 = cav1.ls_to_look_vector(&ImageCoordinate{ line:closest_y as f64, sample:closest_x as f64 }).unwrap();
                let lv0_pt = lv0.look_direction.scale(1000.0).add(&lv0.origin);
                let lv1_pt = lv1.look_direction.scale(1000.0).add(&lv1.origin);
                let  (p1, p2, colinear) = lcross(&lv0.origin, &lv0_pt, &lv1.origin, &lv1_pt);
                if p1.distance_to(&p2) > 0.05 {
                    continue;
                }

                let intersection = p1.add(&p2).scale(0.5);
                println!("{} {} {}", intersection.x, intersection.y, intersection.z);
            }
        }


       

        

        

        
        



    }
}