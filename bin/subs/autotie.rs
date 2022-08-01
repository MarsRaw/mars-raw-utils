use mars_raw_utils::{
    prelude::*
};
use sciimg::{
    prelude::*,
    quality,
    imagebuffer,
    lowpass
};

use crate::subs::runnable::RunnableSubcommand;

use std::process;

use colored::*;

#[derive(clap::Args)]
#[clap(author, version, about = "Attempt to auto generate tiepoints for stereo pair", long_about = None)]
pub struct Autotie {
    #[clap(long, short, help = "Input images", multiple_values(true))]
    inputs: Vec<String>,
}

fn make_diff_for_band(buffer:&imagebuffer::ImageBuffer, amount:usize) -> imagebuffer::ImageBuffer {
    let blurred = lowpass::lowpass_imagebuffer(&buffer, amount);
    blurred.subtract(&buffer).unwrap()
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

        let kernel_size = 25;
        // 361, 620
        let f0 = RgbImage::open16(&self.inputs[0]).unwrap();

        // 530, 619
        let f1 = RgbImage::open16(&self.inputs[1]).unwrap();

        let d0 = make_diff_for_band(f0.get_band(0), 10);
        let d1 = make_diff_for_band(f1.get_band(0), 10);

        let s0 = quality::get_point_quality_estimation_on_diff_buffer(&d0, kernel_size, 361, 610);

        let mut closest_x = 0;
        let mut closest_y = 0;
        let mut closest_sigma = 10000000.0;

        for y in 0..f1.height {
            println!("Row {} of {}. {}%", y, f1.height, (y as f32 / f1.height as f32 * 100.0));

            for x in 0..f1.width {
            
                let s1 = quality::get_point_quality_estimation_on_diff_buffer(&d1, kernel_size, x, y);
                if (s1 - s0).abs() < closest_sigma {
                    closest_sigma = (s1 - s0).abs();
                    closest_x = x;
                    closest_y = y;
                }

            }
        }

        println!("Closest X/Y: {}, {}", closest_x, closest_y);


    }
}