
use mars_raw_utils::{
    prelude::*,
    diffgif,
    focusmerge,
    composite
};
use sciimg::{
    prelude::*,
    inpaint,
    vector::Vector
};

// use std::ffi::OsString;
// use std::path::PathBuf;

use clap::{Parser, Subcommand};

use rayon::prelude::*;

use std::process;

use std::panic;

use backtrace::Backtrace;

/// This allows for a combined definition of arguments required and entry logic for a given subcommand
trait RunnableSubcommand {
    fn run(&self);
}



#[derive(Parser)]
#[clap(name = "mru")]
#[clap(about = "Mars Raw Utils", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Mru,

    #[clap(long, short, help = "Verbose output")]
    verbose: bool,
}


#[derive(Subcommand)]
enum Mru {
    MslFetch(MslFetch),
    M20Fetch(M20Fetch),
    NsytFetch(NsytFetch),
    Calibrate(Calibrate),
    MslDate(MslDate),
    MslLatest(MslLatest),
    M20Date(M20Date),
    M20Latest(M20Latest),
    NsytDate(NsytDate),
    NsytLatest(NsytLatest),
    Anaglyph(Anaglyph),
    Composite(Composite),
    Crop(Crop),
    Debayer(Debayer),
    DiffGif(DiffGif),
    FocusMerge(FocusMerge),
    MeanStack(MeanStack),
    HpcFilter(HpcFilter),
    Inpaint(Inpaint),
    Levels(Levels)
}


#[derive(clap::Args)]
#[clap(author, version, about = "Fetch raw MSL images", long_about = None)]
struct MslFetch {
    #[clap(long, short, help = "MSL Camera Instrument(s)", multiple_values(true))]
    camera: Vec<String>,

    #[clap(long, short = 's', help = "Mission Sol")]
    sol: Option<u32>,

    #[clap(long, short = 'm', help = "Starting Mission Sol")]
    minsol: Option<u32>,

    #[clap(long, short = 'M', help = "Ending Mission Sol")]
    maxsol: Option<u32>,

    #[clap(long, short = 'l', help = "Don't download, only list results")]
    list: bool,

    #[clap(long, short = 't', help = "Download thumbnails in the results")]
    thumbnails: bool,

    #[clap(long, short = 'N', help = "Max number of results")]
    num: Option<u32>,

    #[clap(long, short = 'p', help = "Results page (starts at 1)")]
    page: Option<u8>,

    #[clap(long, short = 'S', help = "Sequence ID")]
    seqid: Option<String>,

    #[clap(long, short = 'I', help = "List instruments")]
    instruments: bool,

    #[clap(long, short, parse(from_os_str), help = "Output directory")]
    output: Option<std::path::PathBuf>,

    #[clap(long, short = 'n', help = "Only new images. Skipped processed images.")]
    new: bool,
}

impl RunnableSubcommand for MslFetch {
    fn run(&self) {

        let instruments = msl::remote::make_instrument_map();
        if self.instruments {
            instruments.print_instruments();
            process::exit(0);
        }

        let sol : i32 = match self.sol {
            Some(s) => s as i32,
            None => -1
        };
        
        let minsol = match self.minsol {
            Some(s) => if sol >= 0 { sol } else { s as i32 },
            None => if sol >= 0 { sol } else { 100000 }
        };

        let maxsol = match self.minsol {
            Some(s) => if sol >= 0 { sol } else { s as i32 },
            None => if sol >= 0 { sol } else { -100000 as i32 }
        };

        let num_per_page = match self.num {
            Some(n) => n as i32,
            None => 100
        };

        let page = match self.page {
            Some(p) => Some(p as i32),
            None => None
        };

        let search = match &self.seqid {
            Some(s) => s.clone(),
            None => "".to_string()
        };

        let output = match &self.output {
            Some(s) => String::from(s.as_os_str().to_str().unwrap()),
            None => path::cwd()
        };

        let camera_ids_res = instruments.find_remote_instrument_names_fromlist(&self.camera);
        let cameras = match camera_ids_res {
            Err(_e) => {
                eprintln!("Invalid camera instrument(s) specified");
                process::exit(1);
            },
            Ok(v) => v,
        };

        msl::remote::print_header();
        match msl::remote::remote_fetch(&cameras, 
                                        num_per_page, 
                                        page, 
                                        minsol, 
                                        maxsol, 
                                        self.thumbnails, 
                                        self.list, 
                                        &search.as_str(), 
                                        self.new, 
                                        &output.as_str()) {
            Ok(c) => println!("{} images found", c),
            Err(e) => eprintln!("Error: {}", e)
        }
    }
}

#[derive(clap::Args)]
#[clap(author, version, about = "Fetch raw Mars2020 images", long_about = None)]
struct M20Fetch {
    #[clap(long, short, help = "Mars2020 Camera Instrument(s)", multiple_values(true))]
    camera: Vec<String>,

    #[clap(long, short = 's', help = "Mission Sol")]
    sol: Option<u32>,

    #[clap(long, short = 'm', help = "Starting Mission Sol")]
    minsol: Option<u32>,

    #[clap(long, short = 'M', help = "Ending Mission Sol")]
    maxsol: Option<u32>,

    #[clap(long, short = 'l', help = "Don't download, only list results")]
    list: bool,

    #[clap(long, short = 't', help = "Download thumbnails in the results")]
    thumbnails: bool,

    #[clap(long, short = 'N', help = "Max number of results")]
    num: Option<u32>,

    #[clap(long, short = 'p', help = "Results page (starts at 1)")]
    page: Option<u8>,

    #[clap(long, short = 'S', help = "Sequence ID")]
    seqid: Option<String>,

    #[clap(long, short = 'I', help = "List instruments")]
    instruments: bool,

    #[clap(long, short = 'e', help = "Only movie frames")]
    movie: bool,

    #[clap(long, short, parse(from_os_str), help = "Output directory")]
    output: Option<std::path::PathBuf>,

    #[clap(long, short = 'n', help = "Only new images. Skipped processed images.")]
    new: bool,
}

impl RunnableSubcommand for M20Fetch {
    fn run(&self) {
        let im = m20::remote::make_instrument_map();
        if self.instruments {
            im.print_instruments();
            process::exit(0);
        }
        
        let sol : i32 = match self.sol {
            Some(s) => s as i32,
            None => -1
        };

        let minsol = match self.minsol {
            Some(s) => if sol >= 0 { sol } else { s as i32 },
            None => if sol >= 0 { sol } else { 100000 }
        };

        let maxsol = match self.minsol {
            Some(s) => if sol >= 0 { sol } else { s as i32 },
            None => if sol >= 0 { sol } else { -100000 as i32 }
        };

        let num_per_page = match self.num {
            Some(n) => n as i32,
            None => 100
        };

        let page = match self.page {
            Some(p) => Some(p as i32),
            None => None
        };

        let search = match &self.seqid {
            Some(s) => s.clone(),
            None => "".to_string()
        };

        let output = match &self.output {
            Some(s) => String::from(s.as_os_str().to_str().unwrap()),
            None => path::cwd()
        };

        let camera_ids_res = im.find_remote_instrument_names_fromlist(&self.camera);
        let cameras = match camera_ids_res {
            Err(_e) => {
                eprintln!("Invalid camera instrument(s) specified");
                process::exit(1);
            },
            Ok(v) => v,
        };

        m20::remote::print_header();
        match m20::remote::remote_fetch(&cameras, 
                                        num_per_page, 
                                        page, 
                                        minsol, 
                                        maxsol, 
                                        self.thumbnails, 
                                        self.movie,
                                        self.list, 
                                        &search.as_str(), 
                                        self.new, 
                                        &output.as_str()) {
            Ok(c) => println!("{} images found", c),
            Err(e) => eprintln!("Error: {}", e)
        };
    }
}

#[derive(clap::Args)]
#[clap(author, version, about = "Fetch raw InSight images", long_about = None)]
struct NsytFetch {
    #[clap(long, short, help = "InSight Camera Instrument(s)", multiple_values(true))]
    camera: Vec<String>,

    #[clap(long, short = 's', help = "Mission Sol")]
    sol: Option<u32>,

    #[clap(long, short = 'm', help = "Starting Mission Sol")]
    minsol: Option<u32>,

    #[clap(long, short = 'M', help = "Ending Mission Sol")]
    maxsol: Option<u32>,

    #[clap(long, short = 'l', help = "Don't download, only list results")]
    list: bool,

    #[clap(long, short = 't', help = "Download thumbnails in the results")]
    thumbnails: bool,

    #[clap(long, short = 'N', help = "Max number of results")]
    num: Option<u32>,

    #[clap(long, short = 'p', help = "Results page (starts at 1)")]
    page: Option<u8>,

    #[clap(long, short = 'S', help = "Sequence ID")]
    seqid: Option<String>,

    #[clap(long, short = 'I', help = "List instruments")]
    instruments: bool,

    #[clap(long, short, parse(from_os_str), help = "Output directory")]
    output: Option<std::path::PathBuf>,

    #[clap(long, short = 'n', help = "Only new images. Skipped processed images.")]
    new: bool,

}

impl RunnableSubcommand for NsytFetch {
    fn run(&self) {

        let instruments = nsyt::remote::make_instrument_map();
        if self.instruments {
            instruments.print_instruments();
            process::exit(0);
        }

        let sol : i32 = match self.sol {
            Some(s) => s as i32,
            None => -1
        };

        let minsol = match self.minsol {
            Some(s) => if sol >= 0 { sol } else { s as i32 },
            None => if sol >= 0 { sol } else { 100000 }
        };

        let maxsol = match self.minsol {
            Some(s) => if sol >= 0 { sol } else { s as i32 },
            None => if sol >= 0 { sol } else { -100000 as i32 }
        };

        let num_per_page = match self.num {
            Some(n) => n as i32,
            None => 100
        };

        let page = match self.page {
            Some(p) => Some(p as i32),
            None => None
        };

        let search = match &self.seqid {
            Some(s) => s.clone(),
            None => "".to_string()
        };

        let output = match &self.output {
            Some(s) => String::from(s.as_os_str().to_str().unwrap()),
            None => path::cwd()
        };

        let camera_ids_res = instruments.find_remote_instrument_names_fromlist(&self.camera);
        let cameras = match camera_ids_res {
            Err(_e) => {
                eprintln!("Invalid camera instrument(s) specified");
                process::exit(1);
            },
            Ok(v) => v,
        };
    
        nsyt::remote::print_header();
        match nsyt::remote::remote_fetch(&cameras, 
                                            num_per_page, 
                                            page, 
                                            minsol, 
                                            maxsol, 
                                            self.thumbnails, 
                                            self.list, 
                                            &search.as_str(), 
                                            self.new, 
                                            &output.as_str()) {
            Ok(c) => println!("{} images found", c),
            Err(e) => eprintln!("Error: {}", e)
        }
    }
}

#[derive(clap::Args)]
#[clap(author, version, about = "Get current MSL mission date information", long_about = None)]
struct MslDate {}

impl RunnableSubcommand for MslDate {
    fn run(&self) {
        match msl::lmst::get_lmst() {
            Ok(mtime) => {
                println!("Mars Sol Date:          {}", mtime.msd);
                println!("Coordinated Mars Time:  {}", mtime.mtc_display);
                println!("Mission Sol:            {}", mtime.sol);
                println!("Mission Time:           {}", mtime.lmst_display);
                println!("Local True Solar Time:  {}", mtime.ltst_display);
                println!("Solar Longitude:        {}", mtime.l_s);
            },
            Err(_e) => {
                eprintln!("Error calculating mission time");
            }
        }
    }
}

#[derive(clap::Args)]
#[clap(author, version, about = "Report sols with new images", long_about = None)]
struct MslLatest {
    #[clap(long, short, help = "List sols with new images only")]
    list: bool,
}

impl RunnableSubcommand for MslLatest {
    fn run(&self) {
        let latest : msl::latest::LatestData = match msl::remote::fetch_latest() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error fetching latest data from MSL remote server: {}", e);
                process::exit(1);
            }
        };
    
        if self.list {
            latest.latest_sols.iter().for_each(|s| {
                println!("{}", s);
            });
        } else {
            println!("Latest data: {}", latest.latest);
            println!("Latest sol: {}", latest.latest_sol);
            println!("Latest sols: {:?}", latest.latest_sols);
            println!("New Count: {}", latest.new_count);
            println!("Sol Count: {}", latest.sol_count);
            println!("Total: {}", latest.total);
        }
    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Get current Mars2020 mission date information", long_about = None)]
struct M20Date {}

impl RunnableSubcommand for M20Date {
    fn run(&self) {
        match m20::lmst::get_lmst() {
            Ok(mtime) => {
                println!("Mars Sol Date:          {}", mtime.msd);
                println!("Coordinated Mars Time:  {}", mtime.mtc_display);
                println!("Mission Sol:            {}", mtime.sol);
                println!("Mission Time:           {}", mtime.lmst_display);
                println!("Local True Solar Time:  {}", mtime.ltst_display);
                println!("Solar Longitude:        {}", mtime.l_s);
            },
            Err(_e) => {
                eprintln!("Error calculating mission time");
            }
        }
    }
}

#[derive(clap::Args)]
#[clap(author, version, about = "Report sols with new images", long_about = None)]
struct M20Latest {
    #[clap(long, short, help = "List sols with new images only")]
    list: bool,
}

impl RunnableSubcommand for M20Latest {
    fn run(&self) {
        let latest : m20::latest::LatestData = match m20::remote::fetch_latest() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error fetching latest data from M20 remote server: {}", e);
                process::exit(1);
            }
        };
    
        if self.list {
            latest.latest_sols.iter().for_each(|s| {
                println!("{}", s);
            });
        } else {
            println!("Latest data: {}", latest.latest);
            println!("Latest sol: {}", latest.latest_sol);
            println!("Latest sols: {:?}", latest.latest_sols);
            println!("New Count: {}", latest.new_count);
            println!("Sol Count: {}", latest.sol_count);
            println!("Total: {}", latest.total);
        }
    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Get current InSight mission date information", long_about = None)]
struct NsytDate {}

impl RunnableSubcommand for NsytDate {
    fn run(&self) {
        match nsyt::lmst::get_lmst() {
            Ok(mtime) => {
                println!("Mars Sol Date:          {}", mtime.msd);
                println!("Coordinated Mars Time:  {}", mtime.mtc_display);
                println!("Mission Sol:            {}", mtime.sol);
                println!("Mission Time:           {}", mtime.lmst_display);
                println!("Local True Solar Time:  {}", mtime.ltst_display);
                println!("Solar Longitude:        {}", mtime.l_s);
            },
            Err(_e) => {
                eprintln!("Error calculating mission time");
            }
        }
    }
}

#[derive(clap::Args)]
#[clap(author, version, about = "Report sols with new images", long_about = None)]
struct NsytLatest {
    #[clap(long, short, help = "List sols with new images only")]
    list: bool,
}

impl RunnableSubcommand for NsytLatest {
    fn run(&self) {
        let latest : nsyt::latest::LatestData = match nsyt::remote::fetch_latest() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error fetching latest data from InSight remote server: {}", e);
                process::exit(1);
            }
        };
    
        if self.list {
            latest.latest_sols.iter().for_each(|s| {
                println!("{}", s);
            });
        } else {
            println!("Latest data: {}", latest.latest);
            println!("Latest sol: {}", latest.latest_sol);
            println!("Latest sols: {:?}", latest.latest_sols);
            println!("New Count: {}", latest.new_count);
            println!("Sol Count: {}", latest.sol_count);
            println!("Total: {}", latest.total);
        }
    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Batch raw image calibration", long_about = None)]
struct Calibrate {
    #[clap(long, short, parse(from_os_str), help = "Input raw images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short = 'I', help = "Force instrument")]
    instrument: Option<String>,

    #[clap(long, short = 'R', help = "Red weight")]
    red_weight: Option<f32>,

    #[clap(long, short = 'G', help = "Green weight")]
    green_weight: Option<f32>,

    #[clap(long, short = 'B', help = "Blue weight")]
    blue_weight: Option<f32>,

    #[clap(long, short, help = "Raw color, skip ILT")]
    raw: bool,

    #[clap(long, short, help = "Color noise reduction amount")]
    color_noise_reduction_amount: Option<i32>,

    #[clap(long, short = 't', help = "HPC threshold")]
    hpc_threshold: Option<f32>,

    #[clap(long, short = 'w', help = "HPC window size")]
    hpc_window: Option<i32>,

    #[clap(long, short = 'P', help = "Calibration profile", multiple_values(true))]
    profile: Option<Vec<String>>,

}

impl Calibrate {

    fn get_calibrator_for_file(input_file:&str, default_instrument:&Option<String>) -> Option<&'static CalContainer>  {
        let metadata_file = util::replace_image_extension(&input_file, "-metadata.json");
        vprintln!("Checking for metadata file at {}", metadata_file);
        if path::file_exists(metadata_file.as_str()) {
            vprintln!("Metadata file exists for loaded image: {}", metadata_file);
            match metadata::load_image_metadata(&metadata_file) {
                Err(_) => {
                    vprintln!("Could not load metadata file!");
                    None
                }, // Error loading the metadata file
                Ok(md) => {
                    calibrator_for_instrument_from_str(&md.instrument)
                }
            }
        } else { // metadata file is missing
    
            // If a default instrument was passed in, try and use that
            if let Some(instrument) = default_instrument {
                calibrator_for_instrument_from_str(&instrument)
            } else {
                vprintln!("We don't know what instrument was used!");
                None // Otherwise, we don't know the instrument.
            }
        }
    }

}

impl RunnableSubcommand for Calibrate {
    fn run(&self) {

        let cal_context = CalProfile{
            apply_ilt: !self.raw,
            red_scalar: match self.red_weight {
                Some(s) => s,
                None => 1.0
            },
            green_scalar: match self.green_weight {
                Some(s) => s,
                None => 1.0
            },
            blue_scalar: match self.blue_weight {
                Some(s) => s,
                None => 1.0
            },
            color_noise_reduction : match self.color_noise_reduction_amount {
                Some(_) => true,
                None => false
            },
            color_noise_reduction_amount : match self.color_noise_reduction_amount {
                Some(s) => s,
                None => 0
            },
            hot_pixel_detection_threshold : match self.hpc_threshold {
                Some(s) => s,
                None => 0.0
            },
            hot_pixel_window_size : match self.hpc_window {
                Some(s) => s,
                None => 3
            },
            filename_suffix: String::from(constants::OUTPUT_FILENAME_APPEND)
        };

        let profiles: Vec<String> = match &self.profile {
            Some(p) => p.clone(),
            None => vec!()
        };

        panic::set_hook(Box::new(|_info| {
            if print::is_verbose() {
                println!("{:?}", Backtrace::new());  
            }
            print_fail(&format!("Internal Error!"));
            
            // If the user has exported MRU_EXIT_ON_PANIC=1, then we should exit here. 
            // This will prevent situations where errors fly by on the screen and
            // aren't noticed when testing.
            match option_env!("MRU_EXIT_ON_PANIC") {
                Some(v) => {
                    if v == "1" {
                        process::exit(1);
                    }
                }
                None => {}
            };   
        }));

        let in_files : Vec<String> = self.input_files.iter().map(|s| String::from(s.as_os_str().to_str().unwrap())).collect();


        in_files.par_iter().for_each(|input_file| {
            let calibrator = Calibrate::get_calibrator_for_file(&input_file, &self.instrument);
            match calibrator {
                Some(cal) => {
    
                    if profiles.len() > 0 {
                        process_with_profiles(&cal, input_file, false, &profiles, |result| {
                            match result {
                                Ok(cc) => print_complete(&format!("{} ({})", path::basename(input_file), cc.cal_context.filename_suffix), cc.status),
                                Err(why) => {
                                    eprintln!("Error: {}", why);
                                    print_fail(&input_file.to_string());
                                }
                            }
                        });
                    } else {
                        
                        
                        match cal.calibrator.process_file(input_file, &cal_context, false) {
                            Ok(cc) => print_complete(&format!("{} ({})", path::basename(input_file), cc.cal_context.filename_suffix), cc.status),
                            Err(why) => {
                                eprintln!("Error: {}", why);
                                print_fail(&input_file.to_string());
                            }
                        }
                    }
                },
                None => {
                    print_fail(&format!("{} - Error: Instrument Unknown!", path::basename(input_file)));
                }
            }
            
        });
    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Generate anaglyph from stereo pair", long_about = None)]
struct Anaglyph {
    #[clap(long, short, parse(from_os_str), help = "Left image")]
    left: std::path::PathBuf,

    #[clap(long, short, parse(from_os_str), help = "Right image")]
    right: std::path::PathBuf,

    #[clap(long, short, parse(from_os_str), help = "Output image")]
    output: std::path::PathBuf,

    #[clap(long, short, help = "Monochrome color (before converting to red/blue)")]
    mono: bool,
}

impl RunnableSubcommand for Anaglyph {
    fn run(&self) {

        let mut left_img = MarsImage::open(String::from(self.left.as_os_str().to_str().unwrap()), Instrument::M20MastcamZLeft);
        let mut right_img = MarsImage::open(String::from(self.right.as_os_str().to_str().unwrap()), Instrument::M20MastcamZRight);
    
        if self.mono {
            vprintln!("Converting input images to monochrome...");
            left_img.to_mono();
            right_img.to_mono();
        }
    
        let left_cahv = if let Some(left_md) = &left_img.metadata {
            if left_md.camera_model_component_list.is_valid() {
                left_md.camera_model_component_list.clone()
            } else {
                process::exit(2);
            }
        } else {
            process::exit(1);
        };
    
        let right_cahv = if let Some(right_md) = &right_img.metadata {
            if right_md.camera_model_component_list.is_valid() {
                right_md.camera_model_component_list.clone()
            } else {
                process::exit(2);
            }
        } else {
            process::exit(1);
        };

        let ground = Vector::new(0.0, 0.0, 1.84566);

        let mut map = RgbImage::create(left_img.image.width, left_img.image.height);
        let output_model = left_cahv.linearize(left_img.image.width, left_img.image.height, left_img.image.width, left_img.image.height).unwrap();
    
        anaglyph::process_image(&right_img, &mut map, &right_cahv, &output_model, &ground, Eye::Right);
        anaglyph::process_image(&left_img, &mut map, &left_cahv, &output_model, &ground, Eye::Left);
    
        map.normalize_to_16bit_with_max(255.0);
        map.save(self.output.as_os_str().to_str().unwrap());
    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Create composite mosaic", long_about = None)]
struct Composite {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, parse(from_os_str), help = "Output image")]
    output: std::path::PathBuf,

    #[clap(long, short, help = "Anaglyph mode")]
    anaglyph: bool,

    #[clap(long, short, help = "Azimuth rotation")]
    azimuth: Option<f64>,

}

impl RunnableSubcommand for Composite {
    fn run(&self) {

        let in_files : Vec<String> = self.input_files.iter().map(|s| String::from(s.as_os_str().to_str().unwrap())).collect();

        let output = self.output.as_os_str().to_str().unwrap();

        let azimuth_rotation:f64 = match self.azimuth {
            Some(a) => a,
            None => 0.0
        };

        let map_context = composite::determine_map_context(&in_files);
        vprintln!("Map Context: {:?}", map_context);
        vprintln!("FOV Vertical: {}", map_context.top_lat - map_context.bottom_lat);
        vprintln!("FOV Horizontal: {}", map_context.right_lon - map_context.left_lon);

        if map_context.width == 0 {
            eprintln!("Output expected to have zero width. Cannot continue with that. Exiting...");
            process::exit(1);
        } else if map_context.height == 0 {
            eprintln!("Output expected to have zero height. Cannot continue with that. Exiting...");
            process::exit(1);
        }

        let mut map = RgbImage::create(map_context.width, map_context.height);

        let first_image = MarsImage::open(in_files[0].to_owned(), Instrument::M20MastcamZLeft);
        let initial_origin = if let Some(model) = composite::get_cahvor(&first_image) {
            model.c()
        } else {
            eprintln!("Cannot determine initial camera origin");
            process::exit(2);
        };

        for in_file in in_files.iter() {
            if path::file_exists(in_file) {
                vprintln!("Processing File: {}", in_file);
                composite::process_file(in_file, &map_context, &mut map, self.anaglyph, azimuth_rotation, &initial_origin);
            } else {
                eprintln!("File not found: {}", in_file);
                process::exit(1);
            }
        }
    
        map.normalize_to_16bit_with_max(255.0);
        map.save(output);
    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Batch image crop", long_about = None)]
struct Crop {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, help = "Crop as x,y,width,height")]
    crop: String,
}

impl RunnableSubcommand for Crop {
    fn run(&self) {
        
        //https://stackoverflow.com/questions/26536871/how-can-i-convert-a-string-of-numbers-to-an-array-or-vector-of-integers-in-rust
        let crop_numbers: Vec<usize> = self.crop.split(",")
                                            .map(|s| s.parse().expect("parse error"))
                                            .collect();

        if crop_numbers.len() != 4 {
            eprintln!("Invalid number of crop parameters specified.");
            process::exit(1);
        }

        let x = crop_numbers[0];
        let y = crop_numbers[1];
        let width = crop_numbers[2];
        let height = crop_numbers[3];

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);
                
                let mut raw = RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                if x >= raw.width {
                    eprintln!("X parameter is out of bounds: {}. Must be between 0 and {}", x, raw.width - 1);
                    process::exit(2);
                }
            
                if y >= raw.height {
                    eprintln!("Y parameter is out of bounds: {}. Must be between 0 and {}", x, raw.height - 1);
                    process::exit(2);
                }
            
                if width > raw.width - x {
                    eprintln!("Specified width exceeds maximum allowable value");
                    process::exit(2);
                }
            
                if height > raw.height - y {
                    eprintln!("Specified height exceeds maximum allowable value");
                    process::exit(2);
                }
            
                let out_file = util::append_file_name(in_file.as_os_str().to_str().unwrap(), "crop");
            
            
                vprintln!("Cropping with x={}, y={}, width={}, height={}", x, y, width, height);
                raw.crop(x, y, width, height);
            
                vprintln!("Saving output to {}", out_file);
            
                raw.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }
    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Batch image debayering", long_about = None)]
struct Debayer {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,
}   

impl RunnableSubcommand for Debayer {
    fn run(&self) {
        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);

                let mut raw = RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                let out_file = util::append_file_name(in_file.as_os_str().to_str().unwrap(), "debayer");

                if !raw.is_grayscale() {
                    vprintln!("WARNING: Image doesn't appear to be grayscale as would be expected.");
                    vprintln!("Results may be inaccurate");
                }

                vprintln!("Debayering image...");
                raw.debayer();

                vprintln!("Writing to disk...");
                raw.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }
    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Create differential gif from a navcam movie", long_about = None)]
struct DiffGif {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, help = "Black level")]
    black: Option<f32>,

    #[clap(long, short, help = "White level")]
    white: Option<f32>,

    #[clap(long, short, help = "Gamma level")]
    gamma: Option<f32>,

    #[clap(long, short, help = "Interframe delay in increments of 10ms")]
    delay: Option<u16>,

    #[clap(long, short, help = "Lowpass window size")]
    lowpass: Option<u8>,

    #[clap(long, short, parse(from_os_str), help = "Output image")]
    output: std::path::PathBuf,

    #[clap(long, short, help = "Product type")]
    prodtype: Option<diffgif::ProductType>,
}

impl RunnableSubcommand for DiffGif {
    fn run(&self) {
        let white_level = match self.white {
            Some(w) => w,
            None => 1.0
        };

        let black_level = match self.black {
            Some(b) => b,
            None => 0.0
        };

        let gamma = match self.gamma {
            Some(g) => g,
            None => 1.0
        };

        let delay = match self.delay {
            Some(d) => d,
            None => 10
        };

        let lowpass_window_size = match self.lowpass {
            Some(l) => l,
            None => 0
        };

        let product_type = match self.prodtype {
            Some(p) => p,
            None => diffgif::ProductType::STANDARD
        };

        let output = self.output.as_os_str().to_str().unwrap();

        if white_level < 0.0 || black_level < 0.0{
            eprintln!("Levels cannot be negative");
            process::exit(1);
        }
    
        if white_level < black_level {
            eprintln!("White level cannot be less than black level");
            process::exit(1);
        }

        if gamma <= 0.0 {
            eprintln!("Gamma cannot be zero or negative");
            process::exit(1);
        }

        let in_files : Vec<String> = self.input_files.iter().map(|s| String::from(s.as_os_str().to_str().unwrap())).collect();

        diffgif::process(&diffgif::DiffGif{
            input_files: in_files,
            output: String::from(output),
            product_type: product_type,
            black_level: black_level,
            white_level: white_level,
            gamma: gamma,
            delay: delay,
            lowpass_window_size: lowpass_window_size
        });

    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Focus merge a series of images of differing focal lengths", long_about = None)]
struct FocusMerge {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, parse(from_os_str), help = "Output image")]
    output: std::path::PathBuf,

    #[clap(long, short = 'w', help = "Quality determination window size (pixels)")]
    window: Option<usize>,
}

impl RunnableSubcommand for FocusMerge {
    fn run(&self) {
        let quality_window_size = match self.window {
            Some(w) => w,
            None => 15
        };

        let output = self.output.as_os_str().to_str().unwrap();
        let in_files : Vec<String> = self.input_files.iter().map(|s| String::from(s.as_os_str().to_str().unwrap())).collect();
        focusmerge::focusmerge(&in_files, quality_window_size, &output);

    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Compute the mean of a series of images", long_about = None)]
struct MeanStack {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, parse(from_os_str), help = "Output image")]
    output: std::path::PathBuf,
}

impl RunnableSubcommand for MeanStack {
    fn run(&self) {

        let output = self.output.as_os_str().to_str().unwrap();

        
        let mut mean : RgbImage = RgbImage::new_empty().unwrap();
        let mut count : ImageBuffer = ImageBuffer::new_empty().unwrap();
        let mut ones : ImageBuffer = ImageBuffer::new_empty().unwrap();

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);
                
                let raw = RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                if mean.is_empty() {
                    mean = raw;
                    count = ImageBuffer::new(mean.width, mean.height).unwrap();
                    ones = ImageBuffer::new_with_fill(mean.width, mean.height, 1.0).unwrap();
                } else {

                    if raw.width != mean.width || raw.height != mean.height {
                        eprintln!("Input image has differing dimensions, cannot continue");
                        process::exit(1);
                    }

                    mean.add(&raw);
                }

                count = count.add(&ones).unwrap();
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }

        if !mean.is_empty() {
            mean.divide_from_each(&count);

            if path::parent_exists_and_writable(output) {
                vprintln!("Writing image to {}", output);
                mean.save(output);
            } else {
                eprintln!("Unable to write output image, parent doesn't exist or is not writable");
            }

        } else {
            println!("No images processed, cannot create output");
        }



    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Perform hot pixel detection and correction", long_about = None)]
struct HpcFilter {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short = 't', help = "HPC threshold")]
    threshold: Option<f32>,

    #[clap(long, short = 'w', help = "HPC window size")]
    window: Option<i32>,
}

impl RunnableSubcommand for HpcFilter {
    fn run(&self) {
        
        let window_size = match self.window {
            Some(w) => w,
            None => 3
        };

        let threshold = match self.threshold {
            Some(t) => t,
            None => 0.0
        };
        
        if threshold < 0.0 {
            eprintln!("Threshold cannot be less than zero!");
            process::exit(1);
        }

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);
                let mut raw = RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                vprintln!("Hot pixel correction with variance threshold {}...", threshold);
                raw.hot_pixel_correction(window_size, threshold);
                
                // DON'T ASSUME THIS!
                let data_max = 255.0;
            
                vprintln!("Normalizing...");
                raw.normalize_to_16bit_with_max(data_max);
            
                vprintln!("Writing to disk...");
            
                let out_file = util::append_file_name(in_file.as_os_str().to_str().unwrap(), "hpc");
                raw.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }    
    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Perform an image inpaint repair", long_about = None)]
struct Inpaint {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,

}

impl RunnableSubcommand for Inpaint {
    fn run(&self) {

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);

                let raw = RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();
                
                vprintln!("Generating mask from red pixels...");
                let mask = inpaint::make_mask_from_red(&raw).unwrap();
                //mask.save("/data/MSL/inpaint_test/test-mask.png", enums::ImageMode::U8BIT).unwrap();

                vprintln!("Inpainting based on generated mask...");
                let filled = match inpaint::apply_inpaint_to_buffer_with_mask(&raw, &mask) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Error in inpainting process: {}", e);
                        process::exit(1);
                    }
                };

                let out_file = util::append_file_name(in_file.as_os_str().to_str().unwrap(), "inpaint");

                vprintln!("Saving output to {}", out_file);

                filled.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }

    }
}


#[derive(clap::Args)]
#[clap(author, version, about = "Adjust image levels", long_about = None)]
struct Levels {
    #[clap(long, short, parse(from_os_str), help = "Input images", multiple_values(true))]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, help = "Black level")]
    black: Option<f32>,

    #[clap(long, short, help = "White level")]
    white: Option<f32>,

    #[clap(long, short, help = "Gamma level")]
    gamma: Option<f32>,
}

impl RunnableSubcommand for Levels {
    fn run(&self) {
        
        let white_level = match self.white {
            Some(w) => w,
            None => 1.0
        };

        let black_level = match self.black {
            Some(b) => b,
            None => 0.0
        };

        let gamma = match self.gamma {
            Some(g) => g,
            None => 1.0
        };
        
        // Some rules on the parameters
        // TODO: Keep an eye on floating point errors
        if white_level < 0.0 || black_level < 0.0{
            eprintln!("Levels cannot be negative");
            process::exit(1);
        }

        if white_level < black_level {
            eprintln!("White level cannot be less than black level");
            process::exit(1);
        }

        if white_level > 1.0 || black_level > 1.0 {
            eprintln!("Levels cannot exceed 1.0");
            process::exit(1);
        }

        if gamma <= 0.0 {
            eprintln!("Gamma cannot be zero or negative");
            process::exit(1);
        }

        for in_file in self.input_files.iter() {
            if in_file.exists() {
                vprintln!("Processing File: {:?}", in_file);

                let mut raw = RgbImage::open(&String::from(in_file.as_os_str().to_str().unwrap())).unwrap();

                vprintln!("Black: {}, White: {}, Gamma: {}, {:?}", black_level, white_level, gamma, in_file);
                raw.levels(black_level, white_level, gamma);

                let out_file = util::append_file_name(in_file.as_os_str().to_str().unwrap(), "lvls");
                raw.save(&out_file);
            } else {
                eprintln!("File not found: {:?}", in_file);
            }
        }
    }
}

fn main() {
    let args = Cli::parse();

    if args.verbose {
        print::set_verbose(true);
    }

    match args.command {
        Mru::MslFetch(args) => {
            args.run();
        },
        Mru::M20Fetch(args) => {
            args.run();
        },
        Mru::NsytFetch(args) => {
            args.run();
        },
        Mru::Calibrate(args) => {
            args.run();
        },
        Mru::MslDate(args) => {
            args.run();
        },
        Mru::MslLatest(args) => {
            args.run();
        },
        Mru::M20Date(args) => {
            args.run();
        },
        Mru::M20Latest(args) => {
            args.run();
        },
        Mru::NsytDate(args) => {
            args.run();
        },
        Mru::NsytLatest(args) => {
            args.run();
        },
        Mru::Anaglyph(args) => {
            args.run();
        },
        Mru::Composite(args) => {
            args.run();
        },
        Mru::Crop(args) => {
            args.run();
        },
        Mru::Debayer(args) => {
            args.run();
        },
        Mru::DiffGif(args) => {
            args.run();
        },
        Mru::FocusMerge(args) => {
            args.run();
        },
        Mru::MeanStack(args) => {
            args.run();
        },
        Mru::HpcFilter(args) => {
            args.run();
        },
        Mru::Inpaint(args) => {
            args.run();
        },
        Mru::Levels(args) => {
            args.run();
        }
    };
}

