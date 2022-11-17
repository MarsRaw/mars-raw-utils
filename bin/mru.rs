use mars_raw_utils::print;
mod subs;
use subs::runnable::RunnableSubcommand;
use subs::*;

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

    M20Fetch(m20::m20fetch::M20Fetch),
    M20Date(m20::m20date::M20Date),
    M20Latest(m20::m20latest::M20Latest),
    M20EcamAssemble(m20::ecamassemble::M20EcamAssemble),

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
}

#[tokio::main]
async fn main() {
    let t1 = std::time::Instant::now();
    let args = Cli::parse_from(wild::args());

    if args.verbose {
        print::set_verbose(true);
    }

    match args.command {
        Mru::MslFetch(args) => {
            args.run().await;
        }
        Mru::M20Fetch(args) => {
            args.run().await;
        }
        Mru::NsytFetch(args) => {
            args.run().await;
        }
        Mru::Calibrate(args) => {
            args.run().await;
        }
        Mru::MslDate(args) => {
            args.run().await;
        }
        Mru::MslLatest(args) => {
            args.run().await;
        }
        Mru::M20Date(args) => {
            args.run().await;
        }
        Mru::M20Latest(args) => {
            args.run().await;
        }
        Mru::NsytDate(args) => {
            args.run().await;
        }
        Mru::M20EcamAssemble(args) => {
            args.run().await;
        }
        Mru::NsytLatest(args) => {
            args.run().await;
        }
        Mru::MerDate(args) => {
            args.run().await;
        }
        Mru::Anaglyph(args) => {
            args.run().await;
        }
        Mru::Composite(args) => {
            args.run().await;
        }
        Mru::Crop(args) => {
            args.run().await;
        }
        Mru::Debayer(args) => {
            args.run().await;
        }
        Mru::DiffGif(args) => {
            args.run().await;
        }
        Mru::FocusMerge(args) => {
            args.run().await;
        }
        Mru::MeanStack(args) => {
            args.run().await;
        }
        Mru::HpcFilter(args) => {
            args.run().await;
        }
        Mru::Inpaint(args) => {
            args.run().await;
        }
        Mru::Levels(args) => {
            args.run().await;
        }
        Mru::Info(args) => {
            args.run().await;
        }
        Mru::Xeye(args) => {
            args.run().await;
        }
        Mru::Profile(args) => {
            args.run().await;
        }
    };
    println!("Runtime: {}s", t1.elapsed().as_secs_f64());
}
