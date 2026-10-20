use std::process;

use anyhow::Result;
use clap::Parser;
use sciimg::path;

use mars_raw_utils::psyche::fetch::PsycheFetch as PsycheFetchClient;

use crate::subs::runnable::RunnableSubcommand;
use mars_raw_utils::remotequery::FetchError;
use mars_raw_utils::{remotequery, util};

use chrono::offset::Utc;
use chrono::{DateTime, Duration, NaiveTime};
use dateparser::parse_with;

use dateparser::DateTimeUtc;

crate::pb_create!();

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Fetch raw Psyche images", long_about = None)]
pub struct PsycheFetch {
    #[arg(long, short, help = "Psyche Camera Instrument(s)", num_args = 1..)]
    camera: Vec<String>,

    #[arg(long, short = 'd', help = "Image Date")]
    date: Option<String>,

    #[arg(long, short = 's', help = "Starting Image Date")]
    startdate: Option<String>,

    #[arg(long, short = 'e', help = "Ending Image Date")]
    enddate: Option<String>,

    #[arg(long, short = 'l', help = "Don't download, only list results")]
    list: bool,

    #[arg(long, short = 'N', help = "Max number of results")]
    num: Option<usize>,

    #[arg(long, short = 'p', help = "Results page (starts at 1)")]
    page: Option<u8>,

    #[arg(long, short = 'f', help = "Camera filter number", num_args = 1..)]
    filter_num: Option<Vec<u32>>,

    #[arg(long, short = 'F', help = "Camera filter name", num_args = 1..)]
    filter_name: Option<Vec<String>>,

    #[arg(long, short = 'w', help = "Camera filter wavelength", num_args = 1..)]
    filter_wavelength: Option<Vec<String>>,

    #[arg(long, short = 'i', help = "filter on image id", num_args = 1..)]
    filter: Option<Vec<String>>,

    #[arg(long, short = 'I', help = "List instruments")]
    instruments: bool,

    #[arg(long, short, help = "Output directory")]
    output: Option<std::path::PathBuf>,

    #[arg(long, short = 'n', help = "Only new images. Skipped processed images.")]
    new: bool,
}

fn parse_date_parameter(on_date: &str) -> Result<DateTime<Utc>> {
    parse_with(on_date, &Utc, NaiveTime::from_hms_opt(0, 0, 0).unwrap())
}

impl RunnableSubcommand for PsycheFetch {
    async fn run(&self) -> Result<()> {
        crate::pb_set_print!();

        let client = PsycheFetchClient::new();

        let im: util::InstrumentMap = remotequery::get_instrument_map(&client).unwrap();
        if self.instruments {
            im.print_instruments();
            process::exit(0);
        }

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

        let camera_ids_res = im.find_remote_instrument_names_fromlist(&self.camera);
        let cameras = match camera_ids_res {
            Err(_e) => {
                error!("Invalid camera instrument(s) specified");
                process::exit(1);
            }
            Ok(v) => v,
        };

        let (start_date, end_date) = if let Some(on_date) = &self.date {
            let dt = parse_date_parameter(on_date)?;
            (dt, dt + Duration::days(1))
        } else {
            (
                match &self.startdate {
                    Some(s) => parse_date_parameter(s)?,
                    None => "2000-01-01".parse::<DateTimeUtc>()?.0,
                },
                match &self.enddate {
                    Some(s) => parse_date_parameter(s)? + Duration::days(1),
                    None => "2100-01-01".parse::<DateTimeUtc>()?.0,
                },
            )
        };

        info!("Query Dates: {:?} - {:?}", start_date, end_date);

        match remotequery::perform_fetch(
            &client,
            &remotequery::RemoteQuery {
                cameras,
                num_per_page,
                page,
                start_date: Some(start_date),
                end_date: Some(end_date),
                list_only: self.list,
                search,
                only_new: self.new,
                filter_num: self.filter_num.clone(),
                filter: self.filter_name.clone(),
                filter_wavelenth: self.filter_wavelength.clone(),
                output_path: output,
                minsol: 0,
                maxsol: 0,
                thumbnails: false,
                movie_only: false,
                product_types: vec![],
            },
            |total| crate::pb_set_length!(total),
            |_| crate::pb_inc!(),
            self.num,
        )
        .await
        {
            Ok(_) => info!("Done"),
            Err(FetchError::SkippingFile) => info!("Not downloading images. Done"),
            Err(why) => error!("Error: {}", why),
        };

        Ok(())
    }
}
