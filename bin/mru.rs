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

#[derive(Subcommand)]
enum Mru {
    MslFetch(msl::mslfetch::MslFetch),
    MslDate(msl::msldate::MslDate),
    MslLatest(msl::msllatest::MslLatest),
    MslLocation(msl::msllocation::MslLocation),
    MslRunOn(msl::mslrunon::MslRunOn),

    M20Fetch(m20::m20fetch::M20Fetch),
    M20Date(m20::m20date::M20Date),
    M20Latest(m20::m20latest::M20Latest),
    M20EcamAssemble(m20::ecamassemble::M20EcamAssemble),
    M20SherlocColorizer(m20::sherloccolorizer::M20SherlocColorizer),
    M20Location(m20::m20location::M20Location),
    M20RunOn(m20::m20runon::M20RunOn),

    NsytFetch(nsyt::nsytfetch::NsytFetch),
    NsytDate(nsyt::nsytdate::NsytDate),
    NsytLatest(nsyt::nsytlatest::NsytLatest),

    MerDate(mer::merdate::MerDate),

    Calibrate(calibrate::Calibrate),
    Anaglyph(anaglyph::Anaglyph),
    Composite(composite::Composite),
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

    if let Err(why) = match args.command {
        Mru::MslFetch(args) => args.run().await,
        Mru::M20Fetch(args) => args.run().await,
        Mru::NsytFetch(args) => args.run().await,
        Mru::Calibrate(args) => args.run().await,
        Mru::MslDate(args) => args.run().await,
        Mru::MslLatest(args) => args.run().await,
        Mru::MslLocation(args) => args.run().await,
        Mru::MslRunOn(args) => args.run().await,
        Mru::M20Date(args) => args.run().await,
        Mru::M20Latest(args) => args.run().await,
        Mru::M20RunOn(args) => args.run().await,
        Mru::NsytDate(args) => args.run().await,
        Mru::M20EcamAssemble(args) => args.run().await,
        Mru::M20SherlocColorizer(args) => args.run().await,
        Mru::M20Location(args) => args.run().await,
        Mru::NsytLatest(args) => args.run().await,
        Mru::MerDate(args) => args.run().await,
        Mru::Anaglyph(args) => args.run().await,
        Mru::Composite(args) => args.run().await,
        Mru::Crop(args) => args.run().await,
        Mru::Debayer(args) => args.run().await,
        Mru::DiffGif(args) => args.run().await,
        Mru::FocusMerge(args) => args.run().await,
        Mru::MeanStack(args) => args.run().await,
        Mru::HpcFilter(args) => args.run().await,
        Mru::Inpaint(args) => args.run().await,
        Mru::Levels(args) => args.run().await,
        Mru::Info(args) => args.run().await,
        Mru::Xeye(args) => args.run().await,
        Mru::Profile(args) => args.run().await,
        Mru::Decorr(args) => args.run().await,
        Mru::UpdateCalData(args) => args.run().await,
        Mru::Pds2Png(args) => args.run().await,
        Mru::Passes(args) => args.run().await,
    } {
        error!("{}", "Unhandled program error:".red());
        error!("{}", why);
    };
    info!("Runtime: {}s", t1.elapsed().as_secs_f64());
    Ok(())
}
