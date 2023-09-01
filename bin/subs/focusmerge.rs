use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::focusmerge;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Focus merge a series of images of differing focal lengths", long_about = None)]
pub struct FocusMerge {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,

    #[arg(long, short = 'w', help = "Quality determination window size (pixels)")]
    window: Option<usize>,

    #[arg(long, short = 'd', help = "Produce a depth map")]
    depth_map: bool,
}

#[async_trait::async_trait]
impl RunnableSubcommand for FocusMerge {
    async fn run(&self) -> Result<()> {
        pb_set_print!();

        let quality_window_size = self.window.unwrap_or(15);

        let output = self.output.as_os_str().to_str().unwrap();
        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();
        focusmerge::focusmerge(&in_files, quality_window_size, self.depth_map, output);

        pb_done!();
        Ok(())
    }
}
