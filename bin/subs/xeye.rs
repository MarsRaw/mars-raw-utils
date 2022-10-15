use image::load_from_memory;
use mars_raw_utils::prelude::*;
use sciimg::{prelude::*, vector::Vector};

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
    fn open_from_bytes(bytes: &Vec<u8>) -> RgbImage;
}

impl OpenFromBytes for RgbImage {
    fn open_from_bytes(bytes: &Vec<u8>) -> RgbImage {
        let image_data = load_from_memory(bytes).unwrap().into_rgba8();
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        let mut rgbimage =
            RgbImage::new_with_bands_masked(width, height, 3, ImageMode::U8BIT, true).unwrap();

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

trait GetCameraModel {
    fn get_camera_model(&self) -> Option<CameraModel>;
    fn has_camera_model(&self) -> bool;
    fn implements_linearized(&self) -> bool;
}

impl GetCameraModel for MarsImage {
    fn get_camera_model(&self) -> Option<CameraModel> {
        if let Some(right_md) = &self.metadata {
            if right_md.camera_model_component_list.is_valid() {
                Some(right_md.camera_model_component_list.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn has_camera_model(&self) -> bool {
        match self.get_camera_model() {
            Some(_) => true,
            None => false,
        }
    }

    fn implements_linearized(&self) -> bool {
        match self.get_camera_model() {
            Some(c) => c
                .linearize(
                    self.image.width,
                    self.image.height,
                    self.image.width,
                    self.image.height,
                )
                .is_ok(),
            None => false,
        }
    }
}

fn simple_create(left_img: &MarsImage, right_img: &MarsImage, map: &mut RgbImage) {
    vprintln!("Adding images");
    map.paste(&right_img.image, 0, 0);
    map.paste(&left_img.image, left_img.image.width, 0);
    map.paste(&right_img.image, left_img.image.width * 2, 0);
}

fn project_line_sample(
    img: &RgbImage,
    input_model: &CameraModel,
    output_model: &Cahv,
    line: usize,
    sample: usize,
    map: &mut RgbImage,
    x_offset: usize,
) {
    let ground = Vector::new(0.0, 0.0, -1.84566);

    if let Ok(lv) = input_model.ls_to_look_vector(&ImageCoordinate {
        line: line as f64,
        sample: sample as f64,
    }) {
        let ls_in = match lv.intersect_to_plane(&ground) {
            Some(ray) => output_model.xyz_to_ls(&ray, false),
            None => output_model.xyz_to_ls(&lv.look_direction, true),
        };

        let in_x = ls_in.sample.round() as usize;
        let in_y = ls_in.line.round() as usize;

        if ls_in.sample >= 0.0 && ls_in.line >= 0.0 && in_x < img.width - 1 && in_y < img.height - 1
        {
            let tl = Point::create(
                sample as f64 + x_offset as f64,
                line as f64,
                img.get_band(0).get(in_x, in_y).unwrap() as f64,
                img.get_band(1).get(in_x, in_y).unwrap() as f64,
                img.get_band(2).get(in_x, in_y).unwrap() as f64,
            );

            let bl = Point::create(
                sample as f64 + x_offset as f64,
                (line + 1) as f64,
                img.get_band(0).get(in_x, in_y).unwrap() as f64,
                img.get_band(1).get(in_x, in_y).unwrap() as f64,
                img.get_band(2).get(in_x, in_y).unwrap() as f64,
            );

            let tr = Point::create(
                (sample + 1) as f64 + x_offset as f64,
                line as f64,
                img.get_band(0).get(in_x, in_y).unwrap() as f64,
                img.get_band(1).get(in_x, in_y).unwrap() as f64,
                img.get_band(2).get(in_x, in_y).unwrap() as f64,
            );

            let br = Point::create(
                (sample + 1) as f64 + x_offset as f64,
                (line + 1) as f64,
                img.get_band(0).get(in_x, in_y).unwrap() as f64,
                img.get_band(1).get(in_x, in_y).unwrap() as f64,
                img.get_band(2).get(in_x, in_y).unwrap() as f64,
            );

            map.paint_square(&tl, &bl, &br, &tr, false, Eye::DontCare);
        }
    }
}

fn linearize_create(left_img: &MarsImage, right_img: &MarsImage, map: &mut RgbImage) {
    // TODO: This.

    let left_input_cahv = left_img
        .get_camera_model()
        .expect("No left CAHV after all...");
    let left_output_cahv = left_input_cahv
        .linearize(
            left_img.image.width,
            left_img.image.height,
            left_img.image.width,
            left_img.image.height,
        )
        .expect("Failed to linearize left CAHV");

    let right_input_cahv = right_img
        .get_camera_model()
        .expect("No right CAHV after all...");
    let right_output_cahv = right_input_cahv
        .linearize(
            right_img.image.width,
            right_img.image.height,
            right_img.image.width,
            right_img.image.height,
        )
        .expect("Failed to linearize right CAHV");

    for y in 0..left_img.image.height {
        for x in 0..left_img.image.width {
            project_line_sample(
                &right_img.image,
                &right_input_cahv,
                &right_output_cahv,
                y,
                x,
                map,
                0,
            );
            project_line_sample(
                &left_img.image,
                &left_input_cahv,
                &left_output_cahv,
                y,
                x,
                map,
                left_img.image.width,
            );
            project_line_sample(
                &right_img.image,
                &right_input_cahv,
                &right_output_cahv,
                y,
                x,
                map,
                left_img.image.width * 2,
            );
        }
    }

    //simple_create(left_img, right_img, map);
}

#[async_trait::async_trait]
impl RunnableSubcommand for CrossEye {
    async fn run(&self) {
        print::print_experimental();

        let left_image_path = String::from(self.left.as_os_str().to_str().unwrap());
        let right_image_path = String::from(self.right.as_os_str().to_str().unwrap());
        let out_file_path = self.output.as_os_str().to_str().unwrap();

        if !path::file_exists(&left_image_path) {
            eprintln!("Error: File not found (left eye): {}", left_image_path);
            process::exit(1);
        }

        if !path::file_exists(&right_image_path) {
            eprintln!("Error: File not found (right eye): {}", right_image_path);
            process::exit(1);
        }

        if !path::parent_exists_and_writable(out_file_path) {
            eprintln!(
                "Error: Output file directory not found or is not writable: {}",
                out_file_path
            );
            process::exit(1);
        }

        vprintln!("Left image: {}", left_image_path);
        let left_img = MarsImage::open(left_image_path, Instrument::M20MastcamZLeft);

        vprintln!("Right image: {}", right_image_path);
        let right_img = MarsImage::open(right_image_path, Instrument::M20MastcamZRight);

        if left_img.image.width != right_img.image.width
            || left_img.image.height != right_img.image.height
        {
            eprintln!("Error: Left and right images have different dimensions");
            process::exit(1);
        }

        let out_width = left_img.image.width * 3;
        let out_height = left_img.image.height + 56;
        let mut map = RgbImage::create(out_width, out_height);

        if left_img.implements_linearized() && right_img.implements_linearized() {
            vprintln!("Both images support CAHV linearization. Taking that path");
            linearize_create(&left_img, &right_img, &mut map);
        } else {
            vprintln!("One or both images support CAHV linearization. Doing simple assembly");
            simple_create(&left_img, &right_img, &mut map);
        }

        vprintln!("Adding X icon");
        let x_icon = RgbImage::open_from_bytes(&include_bytes!("icons/Xicon.png").to_vec());
        map.paste(
            &x_icon,
            left_img.image.width - x_icon.width / 2,
            left_img.image.height + 3,
        );

        vprintln!("Adding verteq icon");
        let eq_icon = RgbImage::open_from_bytes(&include_bytes!("icons/VertEqIcon.png").to_vec());
        map.paste(
            &eq_icon,
            left_img.image.width * 2 - eq_icon.width / 2,
            left_img.image.height + 3,
        );

        map.normalize_to_16bit_with_max(255.0);

        vprintln!("Output to {}", out_file_path);
        map.save(out_file_path);
    }
}
