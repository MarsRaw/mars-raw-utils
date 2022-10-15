use mars_raw_utils::prelude::*;



use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Fetch raw MSL images", long_about = None)]
pub struct MslFetch {
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

    #[clap(long, short = 'f', help = "Filter on image id", multiple_values(true))]
    filter: Option<Vec<String>>,

    #[clap(long, short = 'I', help = "List instruments")]
    instruments: bool,

    #[clap(long, short, parse(from_os_str), help = "Output directory")]
    output: Option<std::path::PathBuf>,

    #[clap(long, short = 'n', help = "Only new images. Skipped processed images.")]
    new: bool,
}

impl MslFetch {
    pub async fn run(&self) {
        let instruments = msl::remote::make_instrument_map();
        if self.instruments {
            instruments.print_instruments();
            process::exit(0);
        }

        let sol: i32 = match self.sol {
            Some(s) => s as i32,
            None => -1,
        };

        let minsol = match self.minsol {
            Some(s) => {
                if sol >= 0 {
                    sol
                } else {
                    s as i32
                }
            }
            None => {
                if sol >= 0 {
                    sol
                } else {
                    100000
                }
            }
        };

        let maxsol = match self.maxsol {
            Some(s) => {
                if sol >= 0 {
                    sol
                } else {
                    s as i32
                }
            }
            None => {
                if sol >= 0 {
                    sol
                } else {
                    -100000_i32
                }
            }
        };

        let num_per_page = match self.num {
            Some(n) => n as i32,
            None => 100,
        };

        let page = self.page.map(|p| p as i32);

        let search = match &self.filter {
            Some(s) => s.clone(),
            None => vec![],
        };

        let output = match &self.output {
            Some(s) => String::from(s.as_os_str().to_str().unwrap()),
            None => path::cwd(),
        };

        let camera_ids_res = instruments.find_remote_instrument_names_fromlist(&self.camera);
        let cameras = match camera_ids_res {
            Err(_e) => {
                eprintln!("Invalid camera instrument(s) specified");
                process::exit(1);
            }
            Ok(v) => v,
        };

        msl::remote::print_header();
        match msl::remote::remote_fetch(
            &cameras,
            num_per_page,
            page,
            minsol,
            maxsol,
            self.thumbnails,
            self.list,
            &search,
            self.new,
            output.as_str(),
        )
        .await
        {
            Ok(c) => println!("{} images found", c),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
