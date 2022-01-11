pub mod appimage;
pub mod aur;
pub mod deb;
#[cfg(feature = "xbps")]
pub mod xbps;

pub mod error;

use clap::Parser;
use colored::{ColoredString, Colorize};

#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
pub struct Args {
    #[clap(short, long)]
    pub targets: Option<Vec<String>>,

    #[clap(short, long)]
    pub musl: bool,

    #[clap(long)]
    no_build: bool,
    #[clap(long)]
    no_strip: bool,
    #[clap(long)]
    separate_debug_symbols: bool,
    #[clap(long)]
    quiet: bool,
    #[clap(long)]
    verbose: bool,
    #[clap(long)]
    install: bool,
    #[clap(long)]
    fast: bool,

    #[clap(long, default_value_t = 1)]
    revision: i32,

    #[clap(long)]
    variant: Option<String>,
    #[clap(long)]
    target: Option<String>,
    #[clap(long)]
    package_name: Option<String>,
    #[clap(long)]
    manifest_path: Option<String>,
    #[clap(long)]
    deb_version: Option<String>,

    //Location  of the icon file this will be used for the appimage
    #[clap(long)]
    icon_file: Option<String>,
}

pub fn p(msg: ColoredString) {
    println!("{} {}", "::".bold(), msg)
}
