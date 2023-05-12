pub mod runnable;

/// Creates the default progress bar as a static ref in global space. Uses lazy_static
#[macro_export]
macro_rules! pb_create {
    () => {
        use indicatif::ProgressBar;
        use lazy_static::lazy_static;

        lazy_static! {
            static ref PB: ProgressBar = ProgressBar::new(0);
        }
    };
}

/// Creates the default spinner as a static ref in global space. Uses lazy_static
#[macro_export]
macro_rules! pb_create_spinner {
    () => {
        use indicatif::{ProgressBar, ProgressStyle};
        use lazy_static::lazy_static;
        use std::time::Duration;

        lazy_static! {
            static ref PB: ProgressBar = {
                // This is directly ripped from the indicatif examples.
                let pb = ProgressBar::new_spinner();
                pb.enable_steady_tick(Duration::from_millis(80));
                pb.set_style(
                    ProgressStyle::with_template("{spinner:.blue} {msg}")
                        .unwrap()
                        .tick_strings(&[
                            "▹▹▹▹▹▹▹▹▹▹",
                            "▸▹▹▹▹▹▹▹▹▹",
                            "▹▸▹▹▹▹▹▹▹▹",
                            "▹▹▸▹▹▹▹▹▹▹",
                            "▹▹▹▸▹▹▹▹▹▹",
                            "▹▹▹▹▸▹▹▹▹▹",
                            "▹▹▹▹▹▸▹▹▹▹",
                            "▹▹▹▹▹▹▸▹▹▹",
                            "▹▹▹▹▹▹▹▸▹▹",
                            "▹▹▹▹▹▹▹▹▸▹",
                            "▹▹▹▹▹▹▹▹▹▸",
                            "▪▪▪▪▪▪▪▪▪▪",
                        ]),
                );
                pb.set_message("Processing...");
                pb
            };
        }
    };
}

/// Injects the progress bar into the mru::print component to route verbose printing
#[macro_export]
macro_rules! pb_set_print {
    () => {
        $crate::print::set_print(|s| {
            PB.println(s);
        });
    };
}

/// Sets the item length of the progress bar
#[macro_export]
macro_rules! pb_set_length {
    ($x: expr) => {
        PB.set_length($x as u64);
    };
}

/// Combined method as proxy to both pb_set_print! and pb_set_length
#[macro_export]
macro_rules! pb_set_print_and_length {
    ($x: expr) => {
        pb_set_print!();
        pb_set_length!($x);
    };
}

/// Increment the progress bar by a specified amount
#[macro_export]
macro_rules! pb_inc_by {
    ($x: expr) => {
        PB.inc($x);
    };
}

/// Increment the progress bar by one
#[macro_export]
macro_rules! pb_inc {
    () => {
        PB.inc(1)
    };
}

/// Print to the console via the progress bar's println method.
#[macro_export]
macro_rules! pb_println {
    ($x: expr) => {
        PB.println($x);
    };
}

/// Finishes the spinner with a 'Done' message
#[macro_export]
macro_rules! pb_done {
    () => {
        PB.finish_with_message("Done");
    };
}

/// Finishes the spinner with a 'Done with error' message
#[macro_export]
macro_rules! pb_done_with_error {
    () => {
        PB.finish_with_message("Done with error");
    };
}

// Mission specific subcommands:
pub mod m20;
pub mod mer;
pub mod msl;
pub mod nsyt;

// Multimission subcommands:
pub mod anaglyph;
pub mod caldata;
pub mod calibrate;
pub mod composite;
pub mod crop;
pub mod debayer;
pub mod decorr;
pub mod diffgif;
pub mod focusmerge;
pub mod hpcfilter;
pub mod info;
pub mod inpaint;
pub mod levels;
pub mod meanstack;
pub mod profile;
pub mod xeye;
