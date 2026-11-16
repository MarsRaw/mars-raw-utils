use crate::subs::runnable::RunnableSubcommand;
use anyhow::{Result, anyhow, bail};
use clap::Parser;
use itertools::iproduct;
use sciimg::{drawable::Drawable, image::Image};
use std::path::PathBuf;
pb_create!();

// The first iteration of this will be images of the same dimensions. As I add metadata output to
// mosaics, I'll update this to assemble those as well.
#[derive(Parser)]
#[command(author, version, about = "Overlay images of the same dimensions", long_about = None)]
pub struct Overlay {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<PathBuf>,

    #[arg(long, short, help = "Output image")]
    output: PathBuf,
}
impl RunnableSubcommand for Overlay {
    async fn run(&self) -> Result<()> {
        if self.input_files.is_empty() {
            // I know, given the num_args=1.. clap parameter above, this won't happen.
            // I'm just adding it as extra extra precaution as I'm gonna use .first() below.
            return Err(anyhow!("No input files specified. Please use -i option"));
        }

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        let first_image_path = match in_files.first() {
            Some(first_image_path) => first_image_path,
            None => return Err(anyhow!("Cannot resolve first image path of inputs")),
        };

        let first_image = match Image::open(first_image_path) {
            Ok(img) => img,
            Err(why) => return Err(anyhow!("Error opening image: {:?}", why)),
        };

        let img_width = first_image.width;
        let img_height = first_image.height;

        let mut composite = Image::create(img_width, img_height);

        in_files.iter().try_for_each(|in_file| {
            let img = match Image::open(in_file) {
                Ok(img) => img,
                Err(why) => bail!("Failed to open file: {:?}", why),
            };
            if img.width != composite.width || img.height != composite.height {
                bail!("Image dimensions do not match");
            }

            // This works like using "Lighten Only in photoshop". For each pixel,
            // this will check whether the input image grayscale value is greater
            // than the composite image. If it is, it will set that value into the
            // composite image.
            iproduct!(0..img.height, 0..img.width).for_each(|(y, x)| {
                let img_gray = img.get_pixel_grayscale(x, y);
                let comp_gray = composite.get_pixel_grayscale(x, y);
                if img_gray > comp_gray {
                    for b in 0..img.num_bands() {
                        let img_pixel_value = img.get_band(b).get(x, y);
                        composite.put(x, y, img_pixel_value, b);
                    }
                }
            });

            Ok(())
        })?;

        if let Some(output_path) = self.output.as_os_str().to_str() {
            vprintln!("Saving output file to {}", output_path);
            composite.save(output_path)
        } else {
            Err(anyhow!("Failed to parse output path"))
        }
    }
}
