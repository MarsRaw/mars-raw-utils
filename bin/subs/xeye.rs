
use mars_raw_utils::{
    prelude::*
};
use sciimg::{
    prelude::*
};
use image::{
    load_from_memory
};

use crate::subs::runnable::RunnableSubcommand;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Generate cross-eye from stereo pair", long_about = None)]
pub struct CrossEye {
    #[clap(long, short, parse(from_os_str), help = "Left image")]
    left: std::path::PathBuf,

    #[clap(long, short, parse(from_os_str), help = "Right image")]
    right: std::path::PathBuf,

    #[clap(long, short, parse(from_os_str), help = "Output image")]
    output: std::path::PathBuf,

}


trait OpenFromBytes {
    fn open_from_bytes(bytes:&Vec<u8>) -> RgbImage;
}

impl OpenFromBytes for RgbImage {

    fn open_from_bytes(bytes:&Vec<u8>) -> RgbImage {
        let image_data = load_from_memory(bytes).unwrap().into_rgba8();
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        let mut rgbimage = RgbImage::new_with_bands_masked(width, height, 3, ImageMode::U8BIT, true).unwrap();

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let red = pixel[0] as f32;
                let green = pixel[1] as f32;
                let blue = pixel[2] as f32;
                let alpha: f32 = pixel[3] as f32;

                rgbimage.put(x, y, red, 0);
                rgbimage.put(x, y, green, 1);
                rgbimage.put(x, y, blue, 2);
                
                rgbimage.put_alpha(x, y, alpha > 0.0);
            }
        }

        rgbimage
    }
}

impl RunnableSubcommand for CrossEye {
    fn run(&self) {
        print::print_experimental();

        let left_image_path = String::from(self.left.as_os_str().to_str().unwrap());
        let right_image_path = String::from(self.right.as_os_str().to_str().unwrap());
        let out_file_path = self.output.as_os_str().to_str().unwrap();

        if ! path::file_exists(&left_image_path) {
            eprintln!("Error: File not found (left eye): {}", left_image_path);
            process::exit(1);
        }

        if ! path::file_exists(&right_image_path) {
            eprintln!("Error: File not found (right eye): {}", right_image_path);
            process::exit(1);
        }

        if ! path::parent_exists_and_writable(&out_file_path) {
            eprintln!("Error: Output file directory not found or is not writable: {}", out_file_path);
            process::exit(1);
        }

        vprintln!("Left image: {}", left_image_path);
        let left_img = MarsImage::open(left_image_path, Instrument::M20MastcamZLeft);

        vprintln!("Right image: {}", right_image_path);
        let right_img = MarsImage::open(right_image_path, Instrument::M20MastcamZRight);

        if left_img.image.width != right_img.image.width || left_img.image.height != right_img.image.height {
            eprintln!("Error: Left and right images have different dimensions");
            process::exit(1);
        }

        let out_width = left_img.image.width * 3;
        let out_height = left_img.image.height + 56;
        let mut map = RgbImage::create(out_width, out_height);

        vprintln!("Adding images");
        map.paste(&right_img.image, 0, 0);
        map.paste(&left_img.image, left_img.image.width, 0);
        map.paste(&right_img.image, left_img.image.width * 2, 0);


        vprintln!("Adding X icon");
        let x_icon = RgbImage::open_from_bytes(&include_bytes!("icons/Xicon.png").to_vec());
        map.paste(&x_icon, left_img.image.width - x_icon.width / 2, left_img.image.height + 3);

        vprintln!("Adding verteq icon");
        let eq_icon = RgbImage::open_from_bytes(&include_bytes!("icons/VertEqIcon.png").to_vec());
        map.paste(&eq_icon, left_img.image.width * 2 - eq_icon.width / 2, left_img.image.height + 3);

        map.normalize_to_16bit_with_max(255.0);

        vprintln!("Output to {}", out_file_path);
        map.save(out_file_path);
    }
}