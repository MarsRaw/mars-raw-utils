use crate::subs::runnable::RunnableSubcommand;
use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::linearize::Boresight;
use mars_raw_utils::linearize::Extent;
use mars_raw_utils::linearize::LevelOptions;
use mars_raw_utils::prelude::*;
use sciimg::prelude::*;
use stump;

pb_create_spinner!();

fn linearize_mars_image(
    in_file: &str,
    rotate_to_level: bool,
    invert_attitude: bool,
    rsm_el: bool,
    max_scale: f64,
    max_width: usize,
    max_height: usize,
    apply_scaling: bool,
) -> Result<()> {
    if !path::file_exists(in_file) {
        return Err(anyhow!("File error: File not found: {}", in_file));
    } else if !path::file_readable(in_file) {
        return Err(anyhow!("File error: File cannot be read: {}", in_file));
    }

    let out_file = util::append_file_name(in_file, "lin");
    if !path::parent_exists_and_writable(&out_file) {
        return Err(anyhow!(
            "File error: destination path not writable: {}",
            out_file
        ));
    }

    let nonlin_image = MarsImage::open(in_file, Instrument::M20MastcamZLeft);

    let rover_metadata = if let Some(md) = nonlin_image.metadata.as_rover() {
        md
    } else {
        return Err(anyhow!(
            "Linearization for nonsurface missions is not supported"
        ));
    };

    let mast_el = if let Some(mast_el) = &rover_metadata.mast_el {
        *mast_el
    } else {
        0.0
    };

    let level_options = if rotate_to_level {
        let boresight = if rsm_el {
            Boresight::Elevation(mast_el.to_radians())
        } else {
            Boresight::Preserve
        };

        Some(&LevelOptions {
            boresight,
            attitude: rover_metadata.attitude,
            invert_attitude,
        })
    } else {
        None
    };

    let extent = if max_width == 0 && max_height == 0 {
        Extent::Expand { max_scale }
    } else if (max_width == 0 && max_height != 0) || (max_width != 0 && max_height == 0) {
        return Err(anyhow!(
            "Canvas width or height cannot be zero when the other is not"
        ));
    } else if max_scale < 1.0 {
        return Err(anyhow!("Canvas scale cannot be less than 1"));
    } else if max_scale > 1.0 && (max_width != 0 || max_height != 0) {
        return Err(anyhow!(
            "Cannot use both canvas scale and width/height parameters"
        ));
    } else {
        Extent::Crop {
            width: max_width,
            height: max_height,
        }
    };

    let out_img = match linearize::linearize_image_expanded(
        &nonlin_image,
        level_options,
        &extent,
        apply_scaling,
    ) {
        Ok(out_img) => out_img,
        Err(why) => return Err(anyhow!("Error linearizing image: {:?}", why)),
    };

    if let Err(why) = out_img.save(&out_file) {
        Err(anyhow!(
            "File error: Failed to write output file to {}: {:?}",
            out_file,
            why
        ))
    } else {
        Ok(())
    }
}

#[derive(Parser)]
#[command(author, version, about = "Linearize lens distortion based on camera model", long_about = None)]
pub struct Linearize {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short = 'a', help = "Invert rover attitude")]
    invert: bool,

    #[arg(long, short = 'r', help = "Rotate image to ground level")]
    rotate: bool,

    #[arg(long, short = 'm', help = "Force RSM elevation over boresight")]
    rsm_el: bool,

    #[arg(long, short = 's', help = "Maximum canvas scale percentage")]
    max_scale: Option<f64>,

    #[arg(long, short = 'w', help = "Maximum canvas width in pixels")]
    max_width: Option<usize>,

    #[arg(long, short = 'H', help = "Maximum canvas height in pixels")]
    max_height: Option<usize>,

    #[arg(
        long,
        short = 'S',
        help = "Apply camera model scaling according to image scale factor"
    )]
    apply_scaling: bool,
}

impl RunnableSubcommand for Linearize {
    async fn run(&self) -> Result<()> {
        pb_set_print!();

        stump::print_experimental();

        for in_file_pb in self.input_files.iter() {
            let in_file = String::from(in_file_pb.as_os_str().to_str().unwrap());
            info!("Linearizing file {}", in_file);
            match linearize_mars_image(
                &in_file,
                self.rotate,
                self.invert,
                self.rsm_el,
                self.max_scale.unwrap_or(1.0),
                self.max_width.unwrap_or(0),
                self.max_height.unwrap_or(0),
                self.apply_scaling,
            ) {
                Ok(_) => info!("File completed"),
                Err(why) => error!("Processing error: {:?}", why),
            };
        }

        pb_done!();
        Ok(())
    }
}
