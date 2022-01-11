use anyhow::{Context, Result};
use cargo_toml::Value;
use fs_extra::dir::CopyOptions;
use std::{fs, process};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

use crate::Args;

pub fn main(args: &Args) -> Result<()> {
    if let Some(icon) = &args.icon_file {
        if std::path::Path::new("./icon.png").exists() {
            eprintln!("Icon cant be named icon.png please rename it!");
            process::exit(-1)
        } else {
            std::fs::copy(icon, "./icon.png")?;
        }
    } else {
        std::fs::write("./icon.png", &[]).context("Failed to generate icon.png")?;
    }

    let meta = cargo_toml::Manifest::<Value>::from_path_with_metadata("./Cargo.toml")
        .context("Cannot find Cargo.toml")?;
    let pkg = meta
        .package
        .context("Cannot load metadata from Cargo.toml")?;
    let assets;
    let mut args = std::env::args().skip(2);
    let mut target = args.find(|f| f.contains("--target="));
    if target.is_some() {
        target = Some(format!(
            "{}/release",
            target.unwrap().split_at(9).1.to_string()
        ));
    }
    let link_deps;

    if let Some(meta) = pkg.metadata.as_ref() {
        match meta {
            Value::Table(t) => match t.get("appimage") {
                Some(Value::Table(t)) => {
                    match t.get("assets") {
                        Some(Value::Array(v)) => {
                            assets = v
                                .to_vec()
                                .into_iter()
                                .filter_map(|v| match v {
                                    Value::String(s) => Some(s),
                                    _ => None,
                                })
                                .collect()
                        }
                        _ => assets = Vec::with_capacity(0),
                    }
                    match t.get("auto_link") {
                        Some(Value::Boolean(v)) => link_deps = v.to_owned(),
                        _ => link_deps = false,
                    }
                }
                _ => {
                    assets = Vec::with_capacity(0);
                    link_deps = false
                }
            },
            _ => {
                assets = Vec::with_capacity(0);
                link_deps = false
            }
        };
    } else {
        assets = Vec::with_capacity(0);
        link_deps = false;
    }

    for currentbin in meta.bin {
        let name = currentbin.name.unwrap_or(pkg.name.clone());
        let appdirpath = std::path::Path::new("target/").join(name.clone() + ".AppDir");
        let target = target.clone().unwrap_or("release".to_string());
        fs_extra::dir::create_all(appdirpath.join("usr"), true)
            .with_context(|| format!("Error creating {}", appdirpath.join("usr").display()))?;

        fs_extra::dir::create_all(appdirpath.join("usr/bin"), true)
            .with_context(|| format!("Error creating {}", appdirpath.join("usr/bin").display()))?;
        if link_deps {
            if !std::path::Path::new("libs").exists() {
                std::fs::create_dir("libs").context("Could not create libs directory")?;
            }
            let awk = std::process::Command::new("awk")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .arg("NF == 4 {print $3}; NF == 2 {print $1}")
                .spawn()
                .context("Could not start awk")?;

            awk.stdin
                .context("Make sure you have awk on your system")?
                .write_all(
                    &std::process::Command::new("ldd")
                        .arg(format!("target/{}/{}", &target, &name))
                        .output()
                        .with_context(|| {
                            format!(
                                "Failed to run ldd on {}",
                                format!("target/{}/{}", &target, &name)
                            )
                        })?
                        .stdout,
                )?;

            let mut linkedlibs = String::new();
            awk.stdout
                .context("Unknown error ocurred while running awk")?
                .read_to_string(&mut linkedlibs)?;

            fs_extra::dir::create("libs", true).context("Failed to create libs dir")?;

            for line in linkedlibs.lines() {
                if line.starts_with('/') && !std::path::Path::new("libs").join(&line[1..]).exists() {
                    std::os::unix::fs::symlink(
                        line,
                        std::path::Path::new("libs").join(
                            std::path::Path::new(line)
                                .file_name()
                                .with_context(|| format!("No filename for {}", line))?,
                        ),
                    )
                    .with_context(|| {
                        format!(
                            "Error symlinking {} to {}",
                            line,
                            std::path::Path::new("libs").join(&line[1..]).display()
                        )
                    })?;
                }
            }
        }

        if std::path::Path::new("libs").exists() {
            for i in std::fs::read_dir("libs").context("Could not read libs dir")? {
                let path = &i?.path();
                let link = std::fs::read_link(path)
                    .with_context(|| format!("Error reading link in libs {}", path.display()))?;

                if fs_extra::dir::create_all(
                    appdirpath.join(
                        &link
                            .parent()
                            .with_context(|| format!("Lib {} has no parent dir", &link.display()))?
                            .to_str()
                            .with_context(|| format!("{} is not valid Unicode", link.display()))?
                            [1..],
                    ),
                    false,
                )
                .is_err()
                {}
                let dest = appdirpath.join(
                    &link
                        .to_str()
                        .with_context(|| format!("{} is not valid Unicode", link.display()))?[1..],
                );
                std::fs::copy(&link, &dest).with_context(|| {
                    format!("Error copying {} to {}", &link.display(), dest.display())
                })?;
            }
        }

        std::fs::copy(
            format!("target/{}/{}", &target, &name),
            appdirpath.join("usr/bin/bin"),
        )
        .with_context(|| format!("Cannot find binary file at target/{}/{}", &target, &name))?;
        std::fs::copy("./icon.png", appdirpath.join("icon.png")).context("Cannot find icon.png")?;
        fs_extra::copy_items(
            &assets,
            appdirpath.as_path(),
            &CopyOptions {
                overwrite: true,
                buffer_size: 0,
                copy_inside: true,
                ..Default::default()
            },
        )
        .context("Error copying assets")?;
        std::fs::write(
            appdirpath.join("cargo-appimage.desktop"),
            format!(
                "[Desktop Entry]\nName={}\nExec=bin\nIcon=icon\nType=Application\nCategories=Utility;", name
                ),
                )
            .with_context(|| {
                format!(
                    "Error writing desktop file {}",
                    appdirpath.join("cargo-appimage.desktop").display()
                    )
            })?;
        std::fs::copy(
            std::path::PathBuf::from(std::env::var("HOME")?)
                .join(".cargo/bin/cargo-appimage-runner"),
            appdirpath.join("AppRun"),
        )
        .with_context(|| {
            format!(
                "Error copying {} to {}",
                std::path::PathBuf::from(std::env::var("HOME").unwrap())
                    .join(".cargo/bin/cargo-appimage-runner")
                    .display(),
                appdirpath.join("AppRun").display()
            )
        })?;

        Command::new("appimagetool")
            .arg(appdirpath)
            .env("ARCH", platforms::target::TARGET_ARCH.as_str())
            .env("VERSION", pkg.version.as_str())
            .status()
            .context("Error occurred: make sure that appimagetool is installed")?;

        fs::rename(
            format!(
                "{}-{}-{}.AppImage",
                name,
                pkg.version.as_str(),
                platforms::target::TARGET_ARCH.as_str(),
            ),
            format!("target/package/{}.AppImage", name),
        )
        .unwrap();
        fs::remove_file("icon.png").unwrap();
    }

    Ok(())
}
