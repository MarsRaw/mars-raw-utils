use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::diffgif;
use std::process;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Create differential gif from a navcam movie", long_about = None, name="diffgif")]
pub struct DiffGif {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short, help = "Black level")]
    black: Option<f32>,

    #[arg(long, short, help = "White level")]
    white: Option<f32>,

    #[arg(long, short, help = "Gamma level")]
    gamma: Option<f32>,

    #[arg(long, short, help = "Interframe delay in increments of 10ms")]
    delay: Option<u16>,

    #[arg(long, short, help = "Lowpass window size")]
    lowpass: Option<u8>,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,

    #[arg(long, short, help = "Product type")]
    prodtype: Option<diffgif::ProductType>,

    #[arg(long, short, help = "Convert RGB to mono")]
    mono: bool,

    #[arg(long, short = 'L', help = "Light only, discard dark values")]
    lightonly: bool,
}

#[async_trait::async_trait]
impl RunnableSubcommand for DiffGif {
    async fn run(&self) -> Result<()> {
        pb_set_print!();

        let white_level = self.white.unwrap_or(1.0);

        let black_level = self.black.unwrap_or(0.0);

        let gamma = self.gamma.unwrap_or(1.0);

        let delay = self.delay.unwrap_or(10);

        let lowpass_window_size = self.lowpass.unwrap_or(0);

        let product_type = match self.prodtype {
            Some(p) => p,
            None => diffgif::ProductType::STANDARD,
        };

        info!(
            "Black: {}, White: {}, Gamma: {}, Lowpass Window: {}, Gif Delay: {}",
            black_level, white_level, gamma, lowpass_window_size, delay
        );
        let output = self.output.as_os_str().to_str().unwrap();

        if white_level < 0.0 || black_level < 0.0 {
            eprintln!("Error: Levels cannot be negative");
            pb_done_with_error!();
            process::exit(1);
        }

        if white_level < black_level {
            eprintln!("Error: White level cannot be less than black level");
            pb_done_with_error!();
            process::exit(1);
        }

        if gamma <= 0.0 {
            eprintln!("Error: Gamma cannot be zero or negative");
            pb_done_with_error!();
            process::exit(1);
        }

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        if in_files.is_empty() {
            eprintln!("Error: No images provided!");
            process::exit(1);
        } else if in_files.len() == 1 {
            eprintln!("Error: Requires more than one image");
            process::exit(1);
        }

        debug!("Processing on files: {:?}", in_files);

        diffgif::process(&diffgif::DiffGif {
            input_files: in_files,
            output: String::from(output),
            product_type,
            black_level: black_level / 100.0,
            white_level: white_level / 100.0,
            gamma,
            delay,
            lowpass_window_size,
            convert_to_mono: self.mono,
            light_only: self.lightonly,
        });
        pb_done!();
        Ok(())
    }
}
