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

impl RunnableSubcommand for Info {
    async fn run(&self) -> Result<()> {
        for in_file in self.input_files.iter() {
            if in_file.exists() {
                println!("Image: {:?}", in_file);
                let img = MarsImage::open(in_file.as_os_str().to_str().unwrap(), Instrument::None);

                println!("Sol:                         {}", img.metadata.sol);
                println!("Instrument:                  {}", img.metadata.instrument);
                println!("Image Id:                    {}", img.metadata.imageid);
                println!(
                    "Date Taken (UTC):            {}",
                    img.metadata.date_taken_utc
                );

                if let Some(sclk) = img.metadata.sclk {
                    println!("SCLK:                        {}", sclk);
                }

                if let Some(dt) = img.metadata.date_taken_mars {
                    println!("Data Taken (Mars):           {}", dt);
                }

                if let Some(sf) = img.metadata.subframe_rect {
                    println!("Subframe Rectangle:          {:?}", sf);
                }

                if let Some(cmt) = img.metadata.camera_model_type {
                    println!("Camera Model Type:           {}", cmt);
                }

                if let Some(site) = img.metadata.site {
                    println!("Site:                        {}", site);
                }

                if let Some(drive) = img.metadata.drive {
                    println!("Drive:                       {}", drive);
                }

                println!("Scale Factor:                {}", img.metadata.scale_factor);

                if let Some(f) = img.metadata.filter_name {
                    println!("Filter Name:                 {}", f);
                }

                if let Some(az) = img.metadata.mast_az {
                    println!("Mast Azimuth:                {}", az);
                }

                if let Some(el) = img.metadata.mast_el {
                    println!("Mast Elevation:              {}", el);
                }

                println!(
                    "Date Received:               {}",
                    img.metadata.date_received
                );
                println!("Sample Type:                 {}", img.metadata.sample_type);

                if let Some(d) = img.metadata.dimension {
                    println!("Dimension:                   {:?}", d);
                }

                println!(
                    "Decompanded:                 {}",
                    img.metadata.decompand.yesno()
                );
                println!(
                    "Debayered:                   {}",
                    img.metadata.debayer.yesno()
                );
                println!(
                    "Flatfielded:                 {}",
                    img.metadata.flatfield.yesno()
                );
                println!(
                    "Radiometric Correction:      {}",
                    img.metadata.radiometric.yesno()
                );
                println!(
                    "Inpainted:                   {}",
                    img.metadata.inpaint.yesno()
                );
                println!(
                    "Cropped:                     {}",
                    img.metadata.cropped.yesno()
                );

                //println!("Caption:                     {}", md.caption);
                println!("Credit:                      {}", img.metadata.credit);

                // Consider adding values derived from CAHVOR camera models
                println!();
                println!();
            } else {
                error!("File not found: {:?}", in_file);
            }
        }
        Ok(())
    }
}
