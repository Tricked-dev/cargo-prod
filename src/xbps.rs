use std::{env::args, fs::File, io::BufWriter};

use crate::{
    aur::{sha256sum, GitHost, Package},
    Args,
};
use anyhow::{Error, Result};
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize, Debug)]
struct Config {
    package: Package,
}

fn cargo_config() -> Result<Package> {
    let content = std::fs::read_to_string("Cargo.toml")?;
    let proj: Config = toml::from_str(&content)?;
    Ok(proj.package)
}

pub fn main(args: &Args) -> Result<()> {
    let conf = cargo_config()?;
    let mut file = BufWriter::new(File::create("target/package/template")?);
    let source = conf.git_host().unwrap_or(GitHost::Github).source(&conf);
    write!(file, "pkgname={}\n", conf.name)?;
    write!(
        file,
        "version={}\n",
        args.deb_version.as_ref().unwrap_or(&conf.version)
    )?;
    write!(file, "revision={}\n", args.revision)?;
    write!(file, "short_desc={}\n", conf.description)?;
    write!(file, "maintainer={}\n", conf.authors.join(","))?;
    write!(file, "license={}\n", conf.license)?;
    write!(file, "homepage={}\n", conf.homepage)?;
    write!(file, "distfiles={}\n", source)?;
    write!(file, "checksum={}\n", sha256sum(&conf).unwrap())?;
    Ok(())
}
