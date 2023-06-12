use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use cli_table::{Cell, Style, Table};
use itertools::iproduct;
use mars_raw_utils::prelude::*;
use sciimg::prelude::*;
use std::path::{Path, PathBuf};
use stump::do_println;
use vicar::*;

pb_create!();

#[derive(Parser)]
#[command(author, version, about = "Convert PDS Vicar images to PNG", long_about = None, name="pds2png")]
pub struct Pds2Png {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short = 'm', help = "Minimum value")]
    min: Option<f32>,

    #[arg(long, short = 'M', help = "Maximum value")]
    max: Option<f32>,

    #[arg(
        long,
        short = 'x',
        help = "Prints minimum and maximum values then exit"
    )]
    minmax: bool,
}

struct FileMinMax {
    file_name: PathBuf,
    min: f32,
    max: f32,
}

#[async_trait::async_trait]
impl RunnableSubcommand for Pds2Png {
    async fn run(&self) -> Result<()> {
        pb_set_print_and_length!(self.input_files.len());

        let mut ranges: Vec<FileMinMax> = vec![];

        self.input_files.iter().for_each(|input_file| {
            let vr = VicarReader::new_from_detached_label(input_file).unwrap();
            let mut image =
                Image::new_with_bands(vr.samples, vr.lines, vr.bands, ImageMode::U16BIT).unwrap();

            iproduct!(0..vr.lines, 0..vr.samples, 0..vr.bands).for_each(|(y, x, b)| {
                let pixel_value = vr.get_pixel_value(y, x, b).unwrap();
                image.put(x, y, pixel_value, b);
            });

            let (mn, mx) = image.get_min_max_all_channel();
            let use_min = if let Some(m) = self.min { m } else { mn };
            let use_max = if let Some(m) = self.max { m } else { mx };

            ranges.push(FileMinMax {
                file_name: input_file.to_owned(),
                min: use_min,
                max: use_max,
            });

            if !self.minmax {
                info!(
                    "Normalizing with starting min/max: {}, {}",
                    use_min, use_max
                );

                image.normalize_to_with_min_max(0.0, 65535.0, use_min, use_max);

                let output_file = util::replace_extension(&input_file, "png").unwrap();
                info!("Saving to output: {}", output_file);
                image.save(&output_file).expect("Failed to save image");
            }
            pb_inc!();
        });

        let table = ranges
            .iter()
            .map(|rg| {
                vec![
                    Path::new(&rg.file_name).to_str().unwrap().cell(),
                    rg.min.cell(),
                    rg.max.cell(),
                ]
            })
            .collect::<Vec<_>>()
            .table()
            .title(vec![
                "File".cell().bold(true),
                "Minimum".cell().bold(true),
                "Maximum".cell().bold(true),
            ]);

        do_println(&format!("{}", &table.display().unwrap()));
        Ok(())
    }
}
