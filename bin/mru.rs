use mars_raw_utils::print;
mod subs;
use subs::runnable::RunnableSubcommand;
use subs::*;

// use std::ffi::OsString;
// use std::path::PathBuf;

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
}

#[tokio::main]
async fn main() {
    let args = Cli::parse_from(wild::args());

    if args.verbose {
        print::set_verbose(true);
    }

    match args.command {
        Mru::MslFetch(args) => {
            _ = args.run().await;
        }
        Mru::M20Fetch(args) => {
            _ = args.run().await;
        }
        Mru::NsytFetch(args) => {
            _ = args.run().await;
        }
        Mru::Calibrate(args) => {
            _ = args.run().await;
        }
        Mru::MslDate(args) => {
            _ = args.run().await;
        }
        Mru::MslLatest(args) => {
            _ = args.run().await;
        }
        Mru::M20Date(args) => {
            _ = args.run().await;
        }
        Mru::M20Latest(args) => {
            _ = args.run().await;
        }
        Mru::NsytDate(args) => {
            _ = args.run().await;
        }
        Mru::M20EcamAssemble(args) => {
            _ = args.run().await;
        }
        Mru::NsytLatest(args) => {
            _ = args.run().await;
        }
        Mru::MerDate(args) => {
            _ = args.run().await;
        }
        Mru::Anaglyph(args) => {
            _ = args.run().await;
        }
        Mru::Composite(args) => {
            _ = args.run().await;
        }
        Mru::Crop(args) => {
            _ = args.run().await;
        }
        Mru::Debayer(args) => {
            _ = args.run().await;
        }
        Mru::DiffGif(args) => {
            _ = args.run().await;
        }
        Mru::FocusMerge(args) => {
            _ = args.run().await;
        }
        Mru::MeanStack(args) => {
            _ = args.run().await;
        }
        Mru::HpcFilter(args) => {
            _ = args.run().await;
        }
        Mru::Inpaint(args) => {
            _ = args.run().await;
        }
        Mru::Levels(args) => {
            _ = args.run().await;
        }
        Mru::Info(args) => {
            _ = args.run().await;
        }
        Mru::Xeye(args) => {
            _ = args.run().await;
        }
    };
}
