
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

    #[clap(name="diffgif")]
    DiffGif(diffgif::DiffGif),
    FocusMerge(focusmerge::FocusMerge),
    MeanStack(meanstack::MeanStack),
    HpcFilter(hpcfilter::HpcFilter),
    Inpaint(inpaint::Inpaint),
    Levels(levels::Levels),
    Info(info::Info),
    Xeye(xeye::CrossEye)
}

fn main() {
    let args = Cli::parse_from(wild::args());

    if args.verbose {
        print::set_verbose(true);
    }

    match args.command {
        Mru::MslFetch(args) => {
            args.run();
        },
        Mru::M20Fetch(args) => {
            args.run();
        },
        Mru::NsytFetch(args) => {
            args.run();
        },
        Mru::Calibrate(args) => {
            args.run();
        },
        Mru::MslDate(args) => {
            args.run();
        },
        Mru::MslLatest(args) => {
            args.run();
        },
        Mru::M20Date(args) => {
            args.run();
        },
        Mru::M20Latest(args) => {
            args.run();
        },
        Mru::NsytDate(args) => {
            args.run();
        },
        Mru::M20EcamAssemble(args) => {
            args.run();
        },
        Mru::NsytLatest(args) => {
            args.run();
        },
        Mru::MerDate(args) => {
            args.run();
        },
        Mru::Anaglyph(args) => {
            args.run();
        },
        Mru::Composite(args) => {
            args.run();
        },
        Mru::Crop(args) => {
            args.run();
        },
        Mru::Debayer(args) => {
            args.run();
        },
        Mru::DiffGif(args) => {
            args.run();
        },
        Mru::FocusMerge(args) => {
            args.run();
        },
        Mru::MeanStack(args) => {
            args.run();
        },
        Mru::HpcFilter(args) => {
            args.run();
        },
        Mru::Inpaint(args) => {
            args.run();
        },
        Mru::Levels(args) => {
            args.run();
        },
        Mru::Info(args) => {
            args.run();
        },
        Mru::Xeye(args) => {
            args.run();
        }
    };
}

