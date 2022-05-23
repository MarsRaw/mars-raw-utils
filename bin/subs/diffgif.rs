use mars_raw_utils::{
    diffgif
};

use crate::subs::runnable::RunnableSubcommand;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Create differential gif from a navcam movie", long_about = None, name="diffgif")]
pub struct DiffGif {
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

        println!("{}, {}, {}, {}, {}", black_level, white_level, gamma, lowpass_window_size, delay);
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
            black_level: black_level / 100.0,
            white_level: white_level / 100.0,
            gamma: gamma,
            delay: delay,
            lowpass_window_size: lowpass_window_size
        });

    }
}