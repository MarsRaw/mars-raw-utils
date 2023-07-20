use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::prelude::*;

#[derive(Parser)]
#[command(author, version, about = "Image information", long_about = None)]
pub struct Info {
    #[arg(long, short, help = "Input images", required(true), num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,
}

pub trait YesNo {
    fn yesno(&self) -> String;
}

impl YesNo for bool {
    fn yesno(&self) -> String {
        if *self {
            "Yes".to_string()
        } else {
            "No".to_string()
        }
    }
}

#[async_trait::async_trait]
impl RunnableSubcommand for Info {
    async fn run(&self) -> Result<()> {
        for in_file in self.input_files.iter() {
            if in_file.exists() {
                println!("Image: {:?}", in_file);
                let img = MarsImage::open(in_file.as_os_str().to_str().unwrap(), Instrument::None);
                if let Some(md) = img.metadata {
                    println!("Sol:                         {}", md.sol);
                    println!("Instrument:                  {}", md.instrument);
                    println!("Image Id:                    {}", md.imageid);
                    println!("Date Taken (UTC):            {}", md.date_taken_utc);

                    if let Some(sclk) = md.sclk {
                        println!("SCLK:                        {}", sclk);
                    }

                    if let Some(dt) = md.date_taken_mars {
                        println!("Data Taken (Mars):           {}", dt);
                    }

                    if let Some(sf) = md.subframe_rect {
                        println!("Subframe Rectangle:          {:?}", sf);
                    }

                    if let Some(cmt) = md.camera_model_type {
                        println!("Camera Model Type:           {}", cmt);
                    }

                    if let Some(site) = md.site {
                        println!("Site:                        {}", site);
                    }

                    if let Some(drive) = md.drive {
                        println!("Drive:                       {}", drive);
                    }

                    println!("Scale Factor:                {}", md.scale_factor);

                    if let Some(f) = md.filter_name {
                        println!("Filter Name:                 {}", f);
                    }

                    if let Some(az) = md.mast_az {
                        println!("Mast Azimuth:                {}", az);
                    }

                    if let Some(el) = md.mast_el {
                        println!("Mast Elevation:              {}", el);
                    }

                    println!("Date Received:               {}", md.date_received);
                    println!("Sample Type:                 {}", md.sample_type);

                    if let Some(d) = md.dimension {
                        println!("Dimension:                   {:?}", d);
                    }

                    println!("Decompanded:                 {}", md.decompand.yesno());
                    println!("Debayered:                   {}", md.debayer.yesno());
                    println!("Flatfielded:                 {}", md.flatfield.yesno());
                    println!("Radiometric Correction:      {}", md.radiometric.yesno());
                    println!("Inpainted:                   {}", md.inpaint.yesno());
                    println!("Cropped:                     {}", md.cropped.yesno());

                    //println!("Caption:                     {}", md.caption);
                    println!("Credit:                      {}", md.credit);

                    // Consider adding values derived from CAHVOR camera models
                } else {
                    eprintln!("Image {:?} lacks metadata", in_file);
                }

                println!();
                println!();
            } else {
                error!("File not found: {:?}", in_file);
            }
        }
        Ok(())
    }
}
