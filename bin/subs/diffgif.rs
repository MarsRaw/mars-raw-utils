use mars_raw_utils::diffgif;

use crate::subs::runnable::RunnableSubcommand;

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Create differential gif from a navcam movie", long_about = None, name="diffgif")]
pub struct DiffGif {
    #[clap(
        long,
        short,
        parse(from_os_str),
        help = "Input images",
        multiple_values(true)
    )]
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
        let white_level = self.white.unwrap_or(1.0);
        let black_level = self.black.unwrap_or(0.0);
        let gamma = self.gamma.unwrap_or(1.0);
        let delay = self.delay.unwrap_or(10);
        let lowpass_window_size = self.lowpass.unwrap_or(0);
        let product_type = self.prodtype.unwrap_or(diffgif::ProductType::STANDARD);

        println!(
            "{}, {}, {}, {}, {}",
            black_level, white_level, gamma, lowpass_window_size, delay
        );
        let output = self.output.as_os_str().to_str().unwrap();

        if white_level < 0.0 || black_level < 0.0 {
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

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        diffgif::process(&diffgif::DiffGif {
            input_files: in_files,
            output: String::from(output),
            product_type,
            black_level: black_level / 100.0,
            white_level: white_level / 100.0,
            gamma,
            delay,
            lowpass_window_size,
        });
    }
}
