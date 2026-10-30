use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::{
    metadata::{FlightMetadata, MissionMetadata, SurfaceMetadata},
    prelude::*,
};

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

fn print_surface_properties(metadata: &SurfaceMetadata) {
    println!("Sol:                         {}", metadata.sol);
    if let Some(sclk) = metadata.sclk {
        println!("SCLK:                        {}", sclk);
    }

    if let Some(dt) = &metadata.date_taken_mars {
        println!("Data Taken (Mars):           {}", dt);
    }

    if let Some(sf) = &metadata.subframe_rect {
        println!("Subframe Rectangle:          {:?}", sf);
    }

    if let Some(cmt) = &metadata.camera_model_type {
        println!("Camera Model Type:           {}", cmt);
    }

    if let Some(site) = metadata.site {
        println!("Site:                        {}", site);
    }

    if let Some(drive) = metadata.drive {
        println!("Drive:                       {}", drive);
    }

    println!("Scale Factor:                {}", metadata.scale_factor);

    if let Some(f) = &metadata.filter_name {
        println!("Filter Name:                 {}", f);
    }

    if let Some(az) = metadata.mast_az {
        println!("Mast Azimuth:                {}", az);
    }

    if let Some(el) = metadata.mast_el {
        println!("Mast Elevation:              {}", el);
    }
    if let Some(d) = &metadata.dimension {
        println!("Dimension:                   {:?}", d);
    }
    println!("Sample Type:                 {}", metadata.sample_type);
}

fn print_nonsurface_properties(metadata: &FlightMetadata) {
    println!("Camera Name:                 {}", metadata.camera_name);
    println!("Camera Title:                {}", metadata.camera_title);

    if let Some(sclk) = metadata.spacecraft_clock {
        println!("Spcacecraft Clock:           {}", sclk);
    }

    println!("Filter #:                    {}", metadata.filter);

    println!("Filter Name:                 {}", metadata.filter_name);

    println!(
        "Filter Wavelength:           {}",
        metadata.filter_wavelength
    );

    if let Some(t) = &metadata.target {
        println!("Filter Name:                 {}", t);
    }

    if let Some(d) = metadata.distance {
        println!("Distance:                {}", d);
    }

    if let Some(on) = metadata.orbit_number {
        println!("Orbit Number:              {}", on);
    }

    println!("Image Width:                 {}", metadata.width);
    println!("Image Height:                {}", metadata.height);
}

impl RunnableSubcommand for Info {
    async fn run(&self) -> Result<()> {
        for in_file in self.input_files.iter() {
            if in_file.exists() {
                println!("Image: {:?}", in_file);
                let img = MarsImage::open(in_file.as_os_str().to_str().unwrap())?;

                println!(
                    "Mission:                     {}",
                    img.metadata.flight_mission
                );
                println!(
                    "Instrument:                  {:?}",
                    img.metadata.instrument()
                );
                println!("Image Id:                    {}", img.metadata.imageid());

                match &img.metadata.mission {
                    MissionMetadata::Surface(m) => print_surface_properties(m),
                    MissionMetadata::Flight(m) => print_nonsurface_properties(m),
                };

                println!(
                    "Date Taken (UTC):            {}",
                    img.metadata.image.date_taken_utc
                );

                println!(
                    "Date Received:               {}",
                    img.metadata.image.date_received
                );

                println!(
                    "Decompanded:                 {}",
                    img.metadata.processing.decompand.yesno()
                );
                println!(
                    "Debayered:                   {}",
                    img.metadata.processing.debayer.yesno()
                );
                println!(
                    "Flatfielded:                 {}",
                    img.metadata.processing.flatfield.yesno()
                );
                println!(
                    "Radiometric Correction:      {}",
                    img.metadata.processing.radiometric.yesno()
                );
                println!(
                    "Inpainted:                   {}",
                    img.metadata.processing.inpaint.yesno()
                );
                println!(
                    "Cropped:                     {}",
                    img.metadata.processing.cropped.yesno()
                );

                //println!("Caption:                     {}", md.caption);
                println!("Credit:                      {}", img.metadata.image.credit);

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
