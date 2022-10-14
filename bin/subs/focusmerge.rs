use mars_raw_utils::focusmerge;

use crate::subs::runnable::RunnableSubcommand;

#[derive(clap::Args)]
#[clap(author, version, about = "Focus merge a series of images of differing focal lengths", long_about = None)]
pub struct FocusMerge {
    #[clap(
        long,
        short,
        parse(from_os_str),
        help = "Input images",
        multiple_values(true)
    )]
    input_files: Vec<std::path::PathBuf>,

    #[clap(long, short, parse(from_os_str), help = "Output image")]
    output: std::path::PathBuf,

    #[clap(long, short = 'w', help = "Quality determination window size (pixels)")]
    window: Option<usize>,

    #[clap(long, short = 'd', help = "Produce a depth map")]
    depth_map: bool,
}

impl RunnableSubcommand for FocusMerge {
    fn run(&self) {
        let quality_window_size = self.window.unwrap_or(15);

        let output = self.output.as_os_str().to_str().unwrap();
        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();
        focusmerge::focusmerge(&in_files, quality_window_size, self.depth_map, output);
    }
}
