use mars_raw_utils::{
    prelude::*,
    vecmath::VecMath
};
use sciimg::{
    prelude::*
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
        // 361, 620
        let f0 = RgbImage::open16(&self.inputs[0]).unwrap();

        // 530, 619
        let f1 = RgbImage::open16(&self.inputs[1]).unwrap();

        let b1 = f1.get_band(0);

        let w0 = f0.get_band(0).isolate_window(kernel_size, 361, 620);

        let mut closest_x = 0;
        let mut closest_y = 0;
        let mut closest_sigma = 0.0;

        let edge = kernel_size / 2 + 1;

        for y in edge..(f1.height - edge) {
            println!("Row {} of {}. {}%", y, f1.height, (y as f32 / f1.height as f32 * 100.0));

            for x in edge..(f1.width - edge) {
                
                let w1 = b1.isolate_window(kernel_size, x, y);

                let c = w0.xcorr(&w1);
                if c.abs() > closest_sigma {
                    closest_sigma = c.abs();
                    closest_x = x;
                    closest_y = y;
                }

            }
        }

        println!("Closest X/Y: {}, {}", closest_x, closest_y);


    }
}