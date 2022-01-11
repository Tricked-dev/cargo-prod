use std::process::Command;

use anyhow::{Context, Result};
use clap::StructOpt;
use colored::{ColoredString, Colorize};
use std::fs;

#[cfg(feature = "xbps")]
use cargo_prod::xbps;
use cargo_prod::{appimage, aur, deb, p, Args};

fn main() -> Result<()> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .args(std::env::args().nth(2))
        .status()
        .context("Failed to build package")?;

    fs::create_dir_all("./target/package/")?;

    let args = Args::parse();

    if let Some(ref targets) = args.targets {
        for target in targets {
            match target.as_str() {
                #[cfg(target_os = "linux")]
                "appimage" => {
                    appimage::main(&args)?;
                }
                #[cfg(target_os = "linux")]
                "deb" => {
                    deb::main(&args);
                }
                #[cfg(target_os = "linux")]
                "aur" => {
                    aur::main(&args);
                }
                #[cfg(feature = "xbps")]
                #[cfg(target_os = "linux")]
                "xbps" => {
                    xbps::main(&args)?;
                }

                _ => {
                    eprintln!("{} is not a valid target", target);
                }
            };
        }
    } else {
        deb::main(&args);
        aur::main(&args);
        #[cfg(feature = "xbps")]
        xbps::main(&args)?;
        appimage::main(&args)?;
    }
    p(format!(
        "Done building check your {} folder",
        "target/package".green()
    )
    .white()
    .bold());
    Ok(())
}
