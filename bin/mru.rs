mod subs;
use anyhow::Result;
use colored::Colorize;
use subs::runnable::RunnableSubcommand;
use subs::*;

#[macro_use]
extern crate stump;

extern crate wild;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "mru")]
#[clap(about = "Mars Raw Utils", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Mru,

    #[clap(long, short, help = "Verbose output")]
    verbose: bool,
}

macro_rules! commands {
    ($( $(#[$meta:meta])* $variant:ident($ty:ty) ),* $(,)?) => {
        #[derive(Subcommand)]
        pub enum Mru {
            $( $(#[$meta])* $variant($ty), )*
        }

        impl Mru {
            pub async fn run(self) -> anyhow::Result<()> {
                match self {
                    $( Mru::$variant(args) => args.run().await, )*
                }
            }
        }
    };
}

commands! {
    MslFetch(msl::mslfetch::MslFetch),
    MslDate(msl::msldate::MslDate),
    MslLatest(msl::msllatest::MslLatest),
    MslLocation(msl::msllocation::MslLocation),
    MslRunOn(msl::mslrunon::MslRunOn),
    MslWeather(msl::mslweather::MslWeather),

    M20Fetch(m20::m20fetch::M20Fetch),
    M20Date(m20::m20date::M20Date),
    M20Latest(m20::m20latest::M20Latest),
    M20EcamAssemble(m20::ecamassemble::M20EcamAssemble),
    M20SherlocColorizer(m20::sherloccolorizer::M20SherlocColorizer),
    M20Location(m20::m20location::M20Location),
    M20RunOn(m20::m20runon::M20RunOn),
    M20Weather(m20::m20weather::M20Weather),

    NsytFetch(nsyt::nsytfetch::NsytFetch),
    NsytDate(nsyt::nsytdate::NsytDate),
    NsytLatest(nsyt::nsytlatest::NsytLatest),

    PsycheFetch(psyche::psychefetch::PsycheFetch),

    MerDate(mer::merdate::MerDate),

    Calibrate(calibrate::Calibrate),
    Compose(compose::Compose),
    Anaglyph(anaglyph::Anaglyph),
    Range(range::Range),
    Composite(composite::Composite),
    Linearize(linearize::Linearize),
    Crop(crop::Crop),
    Debayer(debayer::Debayer),

    #[clap(name = "diffgif")]
    DiffGif(diffgif::DiffGif),
    FocusMerge(focusmerge::FocusMerge),
    MeanStack(meanstack::MeanStack),
    HpcFilter(hpcfilter::HpcFilter),
    Inpaint(inpaint::Inpaint),
    Levels(levels::Levels),
    Info(info::Info),
    Xeye(xeye::CrossEye),
    Profile(profile::Profile),
    Decorr(decorr::DecorrelationStretch),
    UpdateCalData(caldata::UpdateCalData),

    #[clap(name = "pds2png")]
    Pds2Png(pds2png::Pds2Png),

    Passes(passes::Passes),
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let t1 = std::time::Instant::now();

    stump::set_min_log_level(stump::LogEntryLevel::WARN);
    info!("Initialized logging"); // INFO, which means that this won't be seen
                                  // unless the user overrides via environment
                                  // variable.

    let args = Cli::parse_from(wild::args());

    if args.verbose {
        stump::set_verbose(true);
    }

    if let Err(why) = args.command.run().await {
        error!("{}", "Unhandled program error:".red());
        error!("{}", why);
    }

    info!("Runtime: {}s", t1.elapsed().as_secs_f64());
    Ok(())
}
